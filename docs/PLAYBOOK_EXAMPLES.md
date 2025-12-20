# PAGI-Core Hive Playbook Examples (ACE + Ethics)

This repo supports an **Evolving Playbook** that can be stored in the Hive repo as `playbook.toml` and pulled at runtime.

The schema is implemented in [`Playbook`](../common/pagi-common/src/swarm.rs:18).

## 1) Example `playbook.toml` (repo-safe)

This example is safe to commit (no constitution text). Ethics and AI-principles should be injected via environment variables.

```toml
[meta]
version = 2
hive_version = "abc123" # optional (can be populated by merge tooling)
last_updated = "2025-12-20T12:00:00Z"

# playbook revision
version = 1

[instructions]
system_prompt = """
You are a self-improving PAGI agent. Prioritize safety, accuracy, and efficiency.
Reflect on every task: what succeeded, what failed, and propose refinements.
"""
reflection_rules = [
  "Analyze outcomes using success metrics.",
  "Generalize edge cases to cross-domain improvements.",
]
meta_learning = "Dynamically select sub-agents based on task type; optimize for minimal steps."

[context_engineering]
max_context_tokens = 128000
chunking_strategy = "semantic"
retrieval_top_k = 10

[context_engineering.layers]
system = """You are a self-improving, aligned PAGI agent in the PAGI swarm."""
reflection = """After every task: evaluate against metrics+ethics, identify root cause, propose a concrete playbook improvement."""
tools = "Available tools are listed below. Use them precisely."
memory = "Recall relevant long-term knowledge." 
goal = "Current user goal: {{goal}}"

[context_engineering.order]
priority = ["system", "ethics", "ai_principles", "reflection", "tools", "memory", "goal"]

[context_engineering.filters]
pre_tool_use = ["ethics_check", "privacy_scan"]
post_execution = ["reflection_trigger", "artifact_generation"]

[ai_principles]
core_values = ["beneficence", "non-maleficence", "autonomy", "justice", "explicability"]
alignment_checkpoints = ["pre_execution", "post_reflection"]

[ace.generation]
candidate_count = 3
strategy = "Generate concrete, testable improvements; avoid overwriting existing knowledge."

[ace.reflection]
improvement_threshold = 0.05
checkpoints = ["post_reflection"]

[ace.curation]
mode = "append"
max_playbook_bytes = 4096
```

## 2) Example env-gated ethics + principles

These values are loaded at runtime by:
- [`EthicsPolicy::from_env()`](../services/pagi-executive-engine/src/main.rs:38)
- [`EthicsLayer::from_env()`](../services/pagi-context-builder/src/main.rs:29)
- [`PrinciplesLayer::from_env()`](../services/pagi-context-builder/src/main.rs:83)

```bash
export ETHICS_ALIGNMENT_CHECK=true
export ETHICS_CONSTITUTION='1. Do no harm\n2. Respect privacy and consent\n3. Be truthful and transparent'
export ETHICS_RED_LINES='weapons;non-consensual surveillance;manipulate elections'
export ETHICS_REFUSAL_RESPONSE='I cannot assist with that request as it conflicts with my ethical guidelines.'

export AI_PRINCIPLES_CORE_VALUES='beneficence,non-maleficence,autonomy,justice,explicability'
export AI_PRINCIPLES_ALIGNMENT_CHECKPOINTS='pre_execution,post_reflection'
```

## 3) Example RefinementArtifact (what the Hive sync plugin commits)

Artifacts are serialized from [`RefinementArtifact`](../common/pagi-common/src/swarm.rs:383).

```toml
twin_id = "00000000-0000-0000-0000-000000000000"
critique = "Response was logical but lacked empathy; add validation-first guidance."

[updated_playbook]
version = 2

[updated_playbook.instructions]
system_prompt = """(existing prompt preserved)"""
reflection_rules = [
  "When user expresses negative emotion, prioritize validation before solutions.",
  "Mirror the user's emotional tone in the first response.",
]
meta_learning = """(existing)"""
```

