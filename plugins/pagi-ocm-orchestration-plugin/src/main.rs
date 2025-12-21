use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use kube::{
    api::{ListParams, Patch, PatchParams},
    core::{ApiResource, DynamicObject, GroupVersionKind},
    Api, Client,
};
use pagi_common::{PagiError, TwinId};
use pagi_http::errors::PagiAxumError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use tracing::{error, info, warn};

#[derive(Clone)]
struct AppState {
    http: reqwest::Client,
    external_gateway_url: String,
    plugin_url: String,
    kube: Option<Client>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GatewayRegisterPayload {
    twin_id: Option<TwinId>,
    tool: GatewayToolSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GatewayToolSchema {
    name: String,
    description: String,
    plugin_url: String,
    endpoint: String,
    parameters: serde_json::Value,
}

type ApiError = PagiAxumError;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pagi_http::tracing::init("pagi-ocm-orchestration-plugin");

    let external_gateway_url = std::env::var("EXTERNAL_GATEWAY_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8010".to_string());
    let plugin_url = std::env::var("PLUGIN_URL").unwrap_or_else(|_| "http://127.0.0.1:8095".to_string());

    let kube = match Client::try_default().await {
        Ok(c) => Some(c),
        Err(e) => {
            warn!(error = %e, "kubernetes client not available (no kubeconfig/in-cluster config?)");
            None
        }
    };

    let state = AppState {
        http: reqwest::Client::new(),
        external_gateway_url,
        plugin_url,
        kube,
    };

    // Best-effort: register tools with ExternalGateway on startup.
    let st = state.clone();
    tokio::spawn(async move {
        if let Err(err) = register_tools_with_gateway(&st).await {
            error!(error = %err, "failed to register OCM tools with ExternalGateway");
        }
    });

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/healthz", get(|| async { "ok" }))
        // ExternalGateway tools:
        .route("/list_clusters", post(list_clusters))
        .route("/deploy_playbook", post(deploy_playbook))
        .route("/scale_cluster", post(scale_cluster))
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 8095).into());
    info!(%addr, "pagi-ocm-orchestration-plugin listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn register_tools_with_gateway(state: &AppState) -> Result<(), String> {
    let gateway = state.external_gateway_url.trim_end_matches('/');
    let register_url = format!("{gateway}/register_tool");

    let tools = vec![
        GatewayToolSchema {
            name: "list_clusters".to_string(),
            description: "List managed OCM spoke clusters (ManagedCluster resources)".to_string(),
            plugin_url: state.plugin_url.clone(),
            endpoint: "/list_clusters".to_string(),
            parameters: json!({"type": "object"}),
        },
        GatewayToolSchema {
            name: "deploy_playbook".to_string(),
            description: "Deploy a playbook (CID/commit) to a target cluster via ManifestWork".to_string(),
            plugin_url: state.plugin_url.clone(),
            endpoint: "/deploy_playbook".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "cluster_name": {"type": "string"},
                    "playbook_cid": {"type": "string"}
                },
                "required": ["cluster_name", "playbook_cid"]
            }),
        },
        GatewayToolSchema {
            name: "scale_cluster".to_string(),
            description: "Request a cluster scale operation (MVP: records desired_nodes via ManifestWork)".to_string(),
            plugin_url: state.plugin_url.clone(),
            endpoint: "/scale_cluster".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "cluster_name": {"type": "string"},
                    "desired_nodes": {"type": "integer", "minimum": 0}
                },
                "required": ["cluster_name", "desired_nodes"]
            }),
        },
    ];

    for tool in tools {
        let payload = GatewayRegisterPayload { twin_id: None, tool };
        state
            .http
            .post(&register_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
    }

    info!("registered OCM orchestration tools with ExternalGateway");
    Ok(())
}

#[derive(Debug, Serialize)]
struct ListClustersResponse {
    clusters: Vec<String>,
}

async fn list_clusters(
    State(state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> Result<Json<ListClustersResponse>, ApiError> {
    let Some(client) = state.kube.clone() else {
        return Err(PagiAxumError::with_status(
            PagiError::plugin_exec("kubernetes client not configured"),
            StatusCode::BAD_GATEWAY,
        ));
    };

    let clusters = list_managed_clusters(client).await.map_err(PagiAxumError::from)?;
    Ok(Json(ListClustersResponse { clusters }))
}

async fn list_managed_clusters(client: Client) -> Result<Vec<String>, PagiError> {
    let gvk = GroupVersionKind::gvk("cluster.open-cluster-management.io", "v1", "ManagedCluster");
    let mut ar = ApiResource::from_gvk(&gvk);
    ar.plural = "managedclusters".to_string();
    let api: Api<DynamicObject> = Api::all_with(client, &ar);
    let lp = ListParams::default();
    let list = api
        .list(&lp)
        .await
        .map_err(|e| PagiError::plugin_exec(format!("kube list ManagedCluster failed: {e}")))?;

    let mut clusters: Vec<String> = list
        .items
        .into_iter()
        .filter_map(|o| o.metadata.name)
        .collect();
    clusters.sort();
    Ok(clusters)
}

#[derive(Debug, Deserialize)]
struct DeployPlaybookRequest {
    cluster_name: String,
    playbook_cid: String,
}

#[derive(Debug, Serialize)]
struct DeployPlaybookResponse {
    ok: bool,
    manifestwork_name: String,
}

async fn deploy_playbook(
    State(state): State<AppState>,
    Json(req): Json<DeployPlaybookRequest>,
) -> Result<Json<DeployPlaybookResponse>, ApiError> {
    let Some(client) = state.kube.clone() else {
        return Err(PagiAxumError::with_status(
            PagiError::plugin_exec("kubernetes client not configured"),
            StatusCode::BAD_GATEWAY,
        ));
    };

    let name = format!(
        "pagi-playbook-{}",
        sanitize_k8s_name(&req.playbook_cid.chars().take(12).collect::<String>())
    );

    let cm_name = format!("pagi-playbook-{}", sanitize_k8s_name(&req.playbook_cid.chars().take(8).collect::<String>()));

    let obj = json!({
        "apiVersion": "work.open-cluster-management.io/v1",
        "kind": "ManifestWork",
        "metadata": {
            "name": name,
            "namespace": req.cluster_name,
            "labels": {
                "app.kubernetes.io/managed-by": "pagi-ocm-orchestration-plugin"
            }
        },
        "spec": {
            "workload": {
                "manifests": [
                    {
                        "apiVersion": "v1",
                        "kind": "ConfigMap",
                        "metadata": {
                            "name": cm_name,
                            "namespace": "default",
                            "labels": {
                                "app.kubernetes.io/part-of": "pagi"
                            }
                        },
                        "data": {
                            "playbook_cid": req.playbook_cid
                        }
                    }
                ]
            }
        }
    });

    apply_manifestwork(client, &req.cluster_name, &obj)
        .await
        .map_err(PagiAxumError::from)?;
    Ok(Json(DeployPlaybookResponse {
        ok: true,
        manifestwork_name: obj["metadata"]["name"].as_str().unwrap_or_default().to_string(),
    }))
}

#[derive(Debug, Deserialize)]
struct ScaleClusterRequest {
    cluster_name: String,
    desired_nodes: u32,
}

#[derive(Debug, Serialize)]
struct ScaleClusterResponse {
    ok: bool,
    manifestwork_name: String,
}

async fn scale_cluster(
    State(state): State<AppState>,
    Json(req): Json<ScaleClusterRequest>,
) -> Result<Json<ScaleClusterResponse>, ApiError> {
    let Some(client) = state.kube.clone() else {
        return Err(PagiAxumError::with_status(
            PagiError::plugin_exec("kubernetes client not configured"),
            StatusCode::BAD_GATEWAY,
        ));
    };

    let name = format!("pagi-scale-{}", req.desired_nodes);
    let cm_name = "pagi-scale-request";
    let requested_at = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();

    // MVP:
    // - "scale nodes" is environment-specific (K3s+Submariner+provider).
    // - We emit a ManifestWork carrying the requested desired_nodes so an agent
    //   in the spoke can reconcile to the appropriate infra mechanism.
    let obj = json!({
        "apiVersion": "work.open-cluster-management.io/v1",
        "kind": "ManifestWork",
        "metadata": {
            "name": name,
            "namespace": req.cluster_name,
            "labels": {
                "app.kubernetes.io/managed-by": "pagi-ocm-orchestration-plugin",
                "pagi.ai/intent": "scale_cluster"
            }
        },
        "spec": {
            "workload": {
                "manifests": [
                    {
                        "apiVersion": "v1",
                        "kind": "ConfigMap",
                        "metadata": {
                            "name": cm_name,
                            "namespace": "default"
                        },
                        "data": {
                            "cluster_name": req.cluster_name,
                            "desired_nodes": req.desired_nodes.to_string(),
                            "requested_at": requested_at
                        }
                    }
                ]
            }
        }
    });

    apply_manifestwork(client, &req.cluster_name, &obj)
        .await
        .map_err(PagiAxumError::from)?;
    Ok(Json(ScaleClusterResponse {
        ok: true,
        manifestwork_name: obj["metadata"]["name"].as_str().unwrap_or_default().to_string(),
    }))
}

async fn apply_manifestwork(client: Client, cluster_namespace: &str, obj: &serde_json::Value) -> Result<(), PagiError> {
    let gvk = GroupVersionKind::gvk("work.open-cluster-management.io", "v1", "ManifestWork");
    let mut ar = ApiResource::from_gvk(&gvk);
    ar.plural = "manifestworks".to_string();
    let api: Api<DynamicObject> = Api::namespaced_with(client, cluster_namespace, &ar);

    let name = obj
        .get("metadata")
        .and_then(|m| m.get("name"))
        .and_then(|n| n.as_str())
        .ok_or_else(|| PagiError::plugin_exec("manifestwork metadata.name missing"))?;

    let pp = PatchParams::apply("pagi-ocm-orchestration-plugin").force();
    api.patch(name, &pp, &Patch::Apply(obj))
        .await
        .map_err(|e| PagiError::plugin_exec(format!("kube apply ManifestWork failed: {e}")))?;
    Ok(())
}

fn sanitize_k8s_name(s: &str) -> String {
    // DNS-1123 label: [a-z0-9]([-a-z0-9]*[a-z0-9])?
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        let c = c.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else {
            out.push('-');
        }
    }

    while out.starts_with('-') {
        out.remove(0);
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "pagi".to_string()
    } else {
        out
    }
}
