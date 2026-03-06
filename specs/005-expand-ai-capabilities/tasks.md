---

description: "Task list for AI capability expansion and efficiency"
---

# Tasks: AI Capability Expansion and Efficiency

**Input**: Design documents from `/specs/005-expand-ai-capabilities/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/

**Tests**: Tests are required because this feature changes routing behavior, token-budget handling, and policy governance outcomes.

**Organization**: Tasks are grouped by user story so each story can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Task can run in parallel (different files, no dependency on incomplete tasks)
- **[Story]**: User story label (`[US1]`, `[US2]`, `[US3]`)
- Every task includes an explicit file path

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare fixtures, baseline AI metadata scaffolding, and parity test scaffolds.

- [X] T001 Create AI routing and token-budget fixtures in `tests/fixtures/ai_capabilities/`
- [X] T002 [P] Add AI behavior test scaffold module in `tests/mcp_integration_tests.rs`
- [X] T003 [P] Add AI docs/skills parity test scaffold in `tests/integration_tests.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build shared AI routing, budget, and policy primitives required by all user stories.

**⚠️ CRITICAL**: No user story implementation starts until this phase is complete.

- [X] T004 Add typed AI governance reason-code constants in `src/mcp/safety.rs`
- [X] T005 [P] Add AI route state and decision trace primitives in `src/mcp/bootstrap.rs`
- [X] T006 [P] Add prompt policy rule evaluation helpers in `src/mcp/bootstrap.rs`
- [X] T007 Add token-budget record schema and validation helpers in `src/mcp/bootstrap.rs`
- [X] T008 Wire shared AI route metadata envelope fields in `src/mcp/mod.rs`
- [X] T009 Wire shared AI route metadata envelope fields in `src/mcp/stdio.rs`

**Checkpoint**: Shared AI governance foundation complete.

---

## Phase 3: User Story 1 - Smarter AI-First Discovery (Priority: P1) 🎯 MVP

**Goal**: Improve discovery quality using AI-assisted routing while preserving deterministic fallback semantics.

**Independent Test**: Execute discovery intents and verify AI-assisted route metadata, plus deterministic typed fallback when AI is unavailable/low-confidence.

### Tests for User Story 1

- [X] T010 [P] [US1] Add AI-first discovery route tests in `src/mcp/bootstrap.rs`
- [X] T011 [P] [US1] Add deterministic fallback reason-code tests in `src/mcp/mod.rs`
- [X] T012 [US1] Add end-to-end AI discovery behavior tests in `tests/mcp_integration_tests.rs`

### Implementation for User Story 1

- [X] T013 [US1] Implement AI scope profile resolution for discovery in `src/mcp/bootstrap.rs`
- [X] T014 [US1] Implement AI-assisted discovery route selection in `src/search/mod.rs`
- [X] T015 [US1] Implement low-confidence and unavailable-AI deterministic fallback path in `src/mcp/mod.rs`
- [X] T016 [US1] Expose route-state and reason metadata for discovery in `src/mcp/stdio.rs`

**Checkpoint**: US1 discovery behavior is independently functional.

---

## Phase 4: User Story 2 - Token-Efficient Prompt and Context Management (Priority: P1)

**Goal**: Enforce token budgets and deterministic context reduction for AI-assisted workflows.

**Independent Test**: Run requests across budget profiles and confirm budget enforcement, reduction signals, and bounded latency behavior.

### Tests for User Story 2

- [X] T017 [P] [US2] Add budget partition and token-cap validation tests in `src/mcp/bootstrap.rs`
- [X] T018 [P] [US2] Add deterministic context reduction tests in `src/search/mod.rs`
- [X] T019 [US2] Add budget metadata and continuation integration tests in `tests/mcp_integration_tests.rs`

### Implementation for User Story 2

- [X] T020 [US2] Implement budget profiles (`fast|balanced|deep`) handling in `src/mcp/bootstrap.rs`
- [X] T021 [US2] Implement semantic-unit context packing with deterministic ordering in `src/search/mod.rs`
- [X] T022 [US2] Implement deterministic truncation and continuation metadata in `src/mcp/mod.rs`
- [X] T023 [US2] Expose token-usage and reduction metadata through stdio responses in `src/mcp/stdio.rs`
- [X] T024 [US2] Add request-level prompt budget telemetry fields in `src/mcp/skill.rs`

**Checkpoint**: US2 budget and context behavior is independently functional.

---

## Phase 5: User Story 3 - Governed AI Behaviors and Prompt Quality (Priority: P2)

**Goal**: Add prompt governance checks, typed denials, and parity controls for stable AI behavior.

**Independent Test**: Trigger compliant and non-compliant AI actions and verify typed denials, prompt version metadata, and docs/skills parity checks.

### Tests for User Story 3

- [X] T025 [P] [US3] Add prompt policy allow/deny/escalate tests in `src/mcp/bootstrap.rs`
- [X] T026 [P] [US3] Add prompt version/hash metadata tests in `tests/mcp_integration_tests.rs`
- [X] T027 [US3] Add runtime/docs/skills parity tests for AI governance fields in `tests/integration_tests.rs`

### Implementation for User Story 3

- [X] T028 [US3] Implement prompt governance pre-execution checks in `src/mcp/bootstrap.rs`
- [X] T029 [US3] Implement typed `policy_denied` payloads for AI governance failures in `src/mcp/mod.rs`
- [X] T030 [US3] Implement prompt ID/version/hash propagation in `src/mcp/stdio.rs`
- [X] T031 [US3] Update AI governance directives and route rules in `skills/SKILL.md`
- [X] T032 [US3] Update AI mode, fallback, and denial semantics in `README.md`
- [X] T033 [US3] Update AI parity and troubleshooting guidance in `docs/release-readiness-checklist.md` and `docs/bootstrap-policy-troubleshooting.md`

**Checkpoint**: US3 governance and guidance parity is independently verifiable.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final performance checks, full regressions, and quickstart verification.

- [X] T034 [P] Run full AI-capability regression suite in `src/mcp/bootstrap.rs`, `src/mcp/mod.rs`, `src/mcp/stdio.rs`, and `tests/mcp_integration_tests.rs`
- [X] T035 [P] Add/adjust AI routing overhead regression assertions in `tests/mcp_integration_tests.rs`
- [X] T036 [P] Add/adjust token-budget compliance regression assertions in `tests/mcp_integration_tests.rs`
- [X] T037 Run quickstart validation and record outcomes in `specs/005-expand-ai-capabilities/quickstart.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: Starts immediately.
- **Phase 2 (Foundational)**: Depends on Phase 1; blocks all user stories.
- **Phase 3 (US1)**: Depends on Phase 2; recommended MVP delivery.
- **Phase 4 (US2)**: Depends on Phase 2 and US1 routing metadata stability.
- **Phase 5 (US3)**: Depends on US1 + US2 behavior stabilization for governance parity.
- **Phase 6 (Polish)**: Depends on completed user stories in scope.

### User Story Dependencies

- **US1 (P1)**: Independent after foundational completion.
- **US2 (P1)**: Depends on shared route metadata established by US1.
- **US3 (P2)**: Depends on final AI route/budget behavior from US1 and US2.

### Within Each User Story

- Tests MUST be written and fail before implementation.
- Shared state/validation helpers precede route handler wiring.
- Runtime behavior updates precede docs/skills parity updates.
- Each story must pass independent validation before next story.

### Dependency Graph

- `Phase1 -> Phase2 -> US1 -> US2 -> US3 -> Phase6`

---

## Parallel Execution Examples

## Parallel Example: User Story 1

```bash
Task: "T010 [US1] Add AI-first discovery route tests in src/mcp/bootstrap.rs"
Task: "T011 [US1] Add deterministic fallback reason-code tests in src/mcp/mod.rs"
```

## Parallel Example: User Story 2

```bash
Task: "T017 [US2] Add budget partition and token-cap validation tests in src/mcp/bootstrap.rs"
Task: "T018 [US2] Add deterministic context reduction tests in src/search/mod.rs"
```

## Parallel Example: User Story 3

```bash
Task: "T025 [US3] Add prompt policy allow/deny/escalate tests in src/mcp/bootstrap.rs"
Task: "T026 [US3] Add prompt version/hash metadata tests in tests/mcp_integration_tests.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1 and Phase 2.
2. Complete Phase 3 (US1).
3. Validate discovery value uplift and deterministic fallback behavior.
4. Pause for review/demo before budget/governance expansion.

### Incremental Delivery

1. Deliver US1: AI-first discovery with typed deterministic fallback.
2. Deliver US2: token-budget and context efficiency controls.
3. Deliver US3: prompt governance and parity gates.
4. Complete Phase 6 cross-cutting validations.

### Parallel Team Strategy

1. Team completes setup and foundational tasks together.
2. After Phase 2:
   - Engineer A: US1 routing behavior and tests.
   - Engineer B: US2 budget/context controls and tests.
3. Engineer C: US3 governance and docs/skills parity updates.

---

## Notes

- `[P]` tasks are parallelizable when file ownership and dependencies do not conflict.
- Every user-story task includes `[US#]` and an explicit file path.
- Suggested MVP scope: **Phase 3 (US1)** after foundational completion.
