use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

use crate::TwinId;

/// Evolving playbook schema.
///
/// Design goals:
/// - serializable (TOML/JSON)
/// - human-readable
/// - backward-compatible with early/legacy playbooks
///
/// Backward compatibility notes:
/// - `instructions` supports either `String` (legacy) or `[instructions]` table (expanded).
/// - `tools` supports either `Vec<ToolSchema>` (legacy) or `[tools] [[tools.item]] ...` (expanded).
/// - `metrics` supports either `HashMap<String,f64>` (legacy) or `[metrics] ...` (expanded).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Playbook {
    /// Metadata/provenance (schema versioning, hive commit hash, etc.).
    #[serde(default)]
    pub meta: PlaybookMeta,

    /// Monotonic version counter for *playbook revisions* (not schema version).
    #[serde(default)]
    pub version: u32,

    /// Ethical alignment policy (optional).
    ///
    /// IMPORTANT: production deployments should load/override this from env and
    /// avoid committing sensitive policy text into the Hive repo.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ethics: Option<PlaybookEthics>,

    /// Agentic Context Engineering (ACE) configuration (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_engineering: Option<PlaybookContextEngineering>,

    /// High-level ACE (Generation/Reflection/Curation) settings (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ace: Option<AceConfig>,

    /// High-level AI ethical principles (optional). In hardened deployments this can
    /// be injected from env (similar to `ethics`) to avoid PR-based tampering.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_principles: Option<PlaybookAiPrinciples>,

    /// Instruction set (legacy string or expanded table).
    #[serde(default)]
    pub instructions: InstructionsField,

    /// Tool-use logic (legacy tool registry schema or expanded items).
    #[serde(default)]
    pub tools: ToolsField,

    /// Metrics/governance (legacy numeric map or expanded table).
    #[serde(default)]
    pub metrics: MetricsField,

    /// Memory configuration (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<PlaybookMemory>,

    /// Specialized sub-agents (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sub_agents: Option<PlaybookSubAgents>,

    /// Hive-level optimization hooks (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub optimization: Option<PlaybookOptimization>,
}

impl Playbook {
    /// Returns the best available "system prompt" / top-level instruction text.
    pub fn system_prompt(&self) -> &str {
        match &self.instructions {
            InstructionsField::Legacy(s) => s.as_str(),
            InstructionsField::Structured(s) => s.system_prompt.as_str(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaybookMeta {
    /// Playbook schema version (bump on breaking schema changes).
    #[serde(default, rename = "version")]
    pub schema_version: u32,

    /// Git commit hash (or any content address) for traceability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hive_version: Option<String>,

    /// ISO timestamp string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,

    /// Verifiable contributor identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contributor_did: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaybookEthics {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constitution: Option<String>,

    #[serde(default)]
    pub harm_categories: Vec<String>,

    #[serde(default)]
    pub alignment_check: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal_response: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_reputation_for_override: Option<u32>,

    #[serde(default)]
    pub red_lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaybookContextEngineering {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_context_tokens: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunking_strategy: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieval_top_k: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rerank_model: Option<String>,

    #[serde(default)]
    pub layers: PlaybookContextLayers,

    #[serde(default)]
    pub order: PlaybookContextOrder,

    #[serde(default)]
    pub filters: PlaybookContextFilters,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaybookContextLayers {
    #[serde(default)]
    pub system: String,
    #[serde(default)]
    pub reflection: String,
    #[serde(default)]
    pub tools: String,
    #[serde(default)]
    pub memory: String,
    #[serde(default)]
    pub goal: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaybookContextOrder {
    #[serde(default)]
    pub priority: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaybookContextFilters {
    #[serde(default)]
    pub pre_tool_use: Vec<String>,

    #[serde(default)]
    pub post_execution: Vec<String>,
}

/// Agentic Context Engineering (ACE) configuration:
/// Generator → Reflector → Curator cycle parameters.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AceConfig {
    #[serde(default)]
    pub generation: AceGeneration,

    #[serde(default)]
    pub reflection: AceReflection,

    #[serde(default)]
    pub curation: AceCuration,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AceGeneration {
    /// How many candidate updates to propose (offline/online).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_count: Option<u32>,

    /// Free-form guidance for how to generate candidates.
    #[serde(default)]
    pub strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AceReflection {
    /// Minimum improvement to emit an artifact.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub improvement_threshold: Option<f64>,

    /// Where to apply alignment checks (e.g., pre_execution, post_reflection).
    #[serde(default)]
    pub checkpoints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AceCuration {
    /// Curation mode (append, categorize, prune, etc.).
    #[serde(default)]
    pub mode: String,

    /// Soft limit to avoid context bloat.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_playbook_bytes: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaybookAiPrinciples {
    #[serde(default)]
    pub core_values: Vec<String>,

    #[serde(default)]
    pub alignment_checkpoints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaybookInstructions {
    #[serde(default)]
    pub system_prompt: String,

    #[serde(default)]
    pub reflection_rules: Vec<String>,

    #[serde(default)]
    pub meta_learning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InstructionsField {
    Legacy(String),
    Structured(PlaybookInstructions),
}

impl Default for InstructionsField {
    fn default() -> Self {
        InstructionsField::Legacy(String::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaybookTools {
    #[serde(default, rename = "item")]
    pub items: Vec<PlaybookToolItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaybookToolItem {
    pub name: String,

    #[serde(default)]
    pub description: String,

    /// Optional inline logic snippet (Rust/Python/WIT/etc.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logic: Option<String>,

    /// Simple param typing map (human-readable). For JSON-schema-style parameters,
    /// prefer `ToolSchema` legacy entries.
    #[serde(default)]
    pub parameters: BTreeMap<String, String>,

    /// Optional ExternalGateway routing fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolsField {
    Legacy(Vec<ToolSchema>),
    Structured(PlaybookTools),
}

impl Default for ToolsField {
    fn default() -> Self {
        ToolsField::Legacy(Vec::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaybookMetrics {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub success_threshold: Option<f64>,

    #[serde(default)]
    pub failure_modes: Vec<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reflection_weight: Option<f64>,

    /// Forward-compatible additional fields.
    #[serde(default, flatten)]
    pub extra: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetricsField {
    Legacy(HashMap<String, f64>),
    Structured(PlaybookMetrics),
}

impl Default for MetricsField {
    fn default() -> Self {
        MetricsField::Legacy(HashMap::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaybookMemory {
    #[serde(default)]
    pub schema: HashMap<String, toml::Value>,

    #[serde(default)]
    pub retrieval_strategy: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub long_term_storage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaybookSubAgents {
    #[serde(default, rename = "item")]
    pub items: Vec<PlaybookSubAgent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaybookSubAgent {
    pub name: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub playbook_ref: Option<String>,

    #[serde(default)]
    pub specialization: String,

    #[serde(default)]
    pub improvement_focus: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaybookOptimization {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rlhf_data: Option<String>,

    #[serde(default)]
    pub meta_orchestration: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
}

/// Portable, registry-friendly tool schema (mirrors ExternalGateway's fields).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub plugin_url: String,
    pub endpoint: String,
    pub parameters: serde_json::Value,
}

/// Artifact emitted by reflection/governance that can be synchronized by the swarm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinementArtifact {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub twin_id: Option<TwinId>,

    pub critique: String,
    pub updated_playbook: Playbook,
}
