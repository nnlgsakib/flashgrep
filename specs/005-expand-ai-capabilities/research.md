# Research: AI Capability Expansion and Efficiency

## Decision 1: Focus AI on high-value workflows first

- Decision: Expand AI in three primary scopes: intent-driven discovery assistance,
  result synthesis over retrieved context, and guided planning workflows.
- Rationale: These deliver the highest user value while remaining measurable and
  operationally controlled.
- Alternatives considered:
  - Broad assistant behavior across all commands (rejected: scope bloat)
  - Autonomous edit-by-default workflows (rejected: safety and determinism risk)

## Decision 2: Keep deterministic retrieval as primary path

- Decision: Use deterministic index-backed retrieval as first stage; AI augments
  ranking/synthesis after candidate selection unless explicit explore mode is enabled.
- Rationale: Preserves trust, repeatability, and script compatibility.
- Alternatives considered:
  - Semantic-only routing by default (rejected: lower predictability)
  - AI-only retrieval and answer generation (rejected: weak debuggability)

## Decision 3: Enforce explicit fallback ladder with typed reasons

- Decision: Fallback chain is deterministic: lexical/symbol retrieval -> optional
  semantic rerank -> AI synthesis; any AI failure yields typed fallback output.
- Rationale: Guarantees useful results during provider failures or policy denial.
- Alternatives considered:
  - Hard-fail on AI errors (rejected: poor reliability)
  - Silent fallback without reason metadata (rejected: opaque behavior)

## Decision 4: Adopt strict token-budget architecture

- Decision: Partition token budgets by role (policy/system, context, memory,
  response headroom) and enforce deterministic context reduction.
- Rationale: Controls cost/latency while preventing overflow and unstable prompts.
- Alternatives considered:
  - Single soft budget (rejected: unpredictable truncation)
  - Large context by default (rejected: cost/latency blowup)

## Decision 5: Use semantic-unit context packing with continuation metadata

- Decision: Pack context by symbols/functions/modules with bounded overlap and
  explicit continuation markers for trimmed sections.
- Rationale: Improves relevance and reduces duplicate tokens.
- Alternatives considered:
  - Fixed-size chunk windows (rejected: semantic fragmentation)
  - Full-file payloads (rejected: token inefficiency)

## Decision 6: Govern prompts as versioned, auditable artifacts

- Decision: Maintain versioned system-prompt templates with stable IDs/hashes and
  include prompt metadata in route/audit output.
- Rationale: Enables reproducibility, rollback, and policy incident analysis.
- Alternatives considered:
  - Mutable prompt text without versioning (rejected: drift and poor traceability)
  - Runtime-only undocumented prompt changes (rejected: governance gap)

## Decision 7: Add pre-execution policy engine checks

- Decision: Validate AI action requests against deterministic policy rules before
  model invocation; deny with typed reason codes and remediation hints.
- Rationale: Reduces unsafe prompt/tool execution and supports automation controls.
- Alternatives considered:
  - Post-hoc filtering only (rejected: too late)
  - Prompt-text-only policy controls (rejected: non-deterministic)

## Decision 8: Gate runtime/docs/skills parity in release flow

- Decision: Enforce parity checks for AI behavior fields, fallback semantics,
  and governance instructions across runtime metadata, README, and skills.
- Rationale: Prevents stale guidance from causing policy drift.
- Alternatives considered:
  - Manual parity review (rejected: non-enforceable)
  - Runtime tests only (rejected: documentation drift risk)
