---

description: "Task list for precise editing and batch patch reliability"
---

# Tasks: Precise Editing and Patch Reliability

**Input**: Design documents from `/specs/002-improve-edit-precision/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/

**Tests**: Tests are required for this feature because it changes edit behavior and deterministic guarantees.

**Organization**: Tasks are grouped by user story so each story can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Task can run in parallel (different files, no dependency on incomplete tasks)
- **[Story]**: User story label (`[US1]`, `[US2]`, `[US3]`)
- Every task includes an explicit file path

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare fixtures and baseline scaffolding for reliable edit behavior work.

- [X] T001 Create deterministic edit fixture files in `tests/fixtures/editing_precision/`
- [X] T002 [P] Add precise-edit test scaffold cases in `tests/mcp_integration_tests.rs`
- [X] T003 [P] Add docs/skill parity test scaffold in `tests/integration_tests.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build shared validation and contract plumbing required by all user stories.

**⚠️ CRITICAL**: No user story implementation starts until this phase is complete.

- [X] T004 Implement shared edit preflight validation helpers in `src/mcp/code_io.rs`
- [X] T005 [P] Add typed edit/batch reason code constants in `src/mcp/safety.rs`
- [X] T006 [P] Add deterministic operation ordering and duplicate-ID checks in `src/mcp/code_io.rs`
- [X] T007 Implement batch request/response structs from data model in `src/mcp/code_io.rs`
- [X] T008 Wire `batch_write_code` tool schema and metadata in `src/mcp/tools.rs`
- [X] T009 Wire `batch_write_code` dispatch handlers in `src/mcp/stdio.rs` and `src/mcp/mod.rs`

**Checkpoint**: Foundation complete; user stories can proceed.

---

## Phase 3: User Story 1 - Prevent Wrong Edits (Priority: P1) 🎯 MVP

**Goal**: Ensure single-edit operations are exact, conflict-safe, and deterministic.

**Independent Test**: Apply single edits against repeated-content fixtures and verify only target lines change, stale targets fail with typed conflicts, and no duplicate lines are produced.

### Tests for User Story 1

- [X] T010 [P] [US1] Add exact-range replacement and no-duplication unit tests in `src/mcp/code_io.rs`
- [X] T011 [P] [US1] Add stale-precondition rejection unit tests in `src/mcp/code_io.rs`
- [X] T012 [US1] Add single-edit end-to-end MCP test cases in `tests/mcp_integration_tests.rs`

### Implementation for User Story 1

- [X] T013 [US1] Harden `write_code` range mutation to preserve newline invariants in `src/mcp/code_io.rs`
- [X] T014 [US1] Enforce optimistic preconditions before any write apply in `src/mcp/code_io.rs`
- [X] T015 [US1] Return deterministic typed conflict/success payload fields in `src/mcp/code_io.rs`
- [X] T016 [US1] Normalize single-edit tool error mapping in `src/mcp/stdio.rs` and `src/mcp/mod.rs`

**Checkpoint**: US1 is independently functional and safe.

---

## Phase 4: User Story 2 - Apply Batch Edits Safely (Priority: P1)

**Goal**: Add deterministic batch editing with explicit `atomic` and `best_effort` modes.

**Independent Test**: Run mixed-validity batches and verify mode-specific behavior, overlap detection, per-operation status reporting, and deterministic ordering.

### Tests for User Story 2

- [X] T017 [P] [US2] Add `atomic` rollback behavior tests for multi-op batches in `src/mcp/code_io.rs`
- [X] T018 [P] [US2] Add `best_effort` per-operation status tests in `src/mcp/code_io.rs`
- [X] T019 [US2] Add overlap and duplicate-ID preflight integration tests in `tests/mcp_integration_tests.rs`

### Implementation for User Story 2

- [X] T020 [US2] Implement `batch_write_code` execution pipeline in `src/mcp/code_io.rs`
- [X] T021 [US2] Implement overlapping-range and duplicate-target rejection in `src/mcp/code_io.rs`
- [X] T022 [US2] Add batch mode (`atomic`/`best_effort`) handling and summary counters in `src/mcp/code_io.rs`
- [X] T023 [US2] Expose batch tool call path and output wiring in `src/mcp/stdio.rs` and `src/mcp/mod.rs`
- [X] T024 [US2] Add batch dry-run preview response behavior in `src/mcp/code_io.rs`

**Checkpoint**: US2 is independently functional with deterministic batch semantics.

---

## Phase 5: User Story 3 - Keep Docs and Skills Aligned (Priority: P2)

**Goal**: Ensure docs and skills accurately describe precise and batch editing behavior.

**Independent Test**: Follow updated docs and skill workflows to perform single and batch edits and confirm outcomes match contract fields and error semantics.

### Tests for User Story 3

- [X] T025 [P] [US3] Add docs contract-field parity checks in `tests/integration_tests.rs`
- [X] T026 [P] [US3] Add skill workflow parity checks for precondition-safe edits in `tests/integration_tests.rs`

### Implementation for User Story 3

- [X] T027 [US3] Update precise and batch editing usage guidance in `README.md`
- [X] T028 [US3] Update edit and batch-safe workflow directives in `skills/SKILL.md`
- [X] T029 [US3] Update release and troubleshooting guidance for edit reliability in `docs/release-readiness-checklist.md` and `docs/bootstrap-policy-troubleshooting.md`

**Checkpoint**: US3 guidance is complete and independently verifiable.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final validation across user stories and performance constraints.

- [X] T030 [P] Run full editing regression tests in `src/mcp/code_io.rs` and `tests/mcp_integration_tests.rs`
- [X] T031 [P] Add/adjust batch performance regression assertions in `tests/mcp_integration_tests.rs`
- [X] T032 Run quickstart validation and record outcomes in `specs/002-improve-edit-precision/quickstart.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: Starts immediately.
- **Phase 2 (Foundational)**: Depends on Phase 1; blocks all user stories.
- **Phase 3 (US1)**: Depends on Phase 2; recommended MVP start.
- **Phase 4 (US2)**: Depends on Phase 2; can run in parallel with US1 if staffed.
- **Phase 5 (US3)**: Depends on Phase 4 contract stability for final docs/skills wording.
- **Phase 6 (Polish)**: Depends on completion of selected stories.

### User Story Dependencies

- **US1 (P1)**: Independent after foundational completion.
- **US2 (P1)**: Independent after foundational completion, but benefits from US1 precondition logic landing first.
- **US3 (P2)**: Depends on final single-edit and batch behavior from US1 and US2.

### Within Each User Story

- Write tests first and confirm they fail before implementation.
- Implement validators/models before execution paths.
- Implement execution paths before docs integration.
- Complete each story and validate independently before moving on.

### Dependency Graph

- `Phase1 -> Phase2 -> (US1 || US2) -> US3 -> Phase6`

---

## Parallel Execution Examples

## Parallel Example: User Story 1

```bash
# Parallel US1 tests
Task: "T010 [US1] Add exact-range replacement and no-duplication unit tests in src/mcp/code_io.rs"
Task: "T011 [US1] Add stale-precondition rejection unit tests in src/mcp/code_io.rs"
```

## Parallel Example: User Story 2

```bash
# Parallel US2 tests
Task: "T017 [US2] Add atomic rollback behavior tests for multi-op batches in src/mcp/code_io.rs"
Task: "T018 [US2] Add best_effort per-operation status tests in src/mcp/code_io.rs"
```

## Parallel Example: User Story 3

```bash
# Parallel docs/skill parity checks
Task: "T025 [US3] Add docs contract-field parity checks in tests/integration_tests.rs"
Task: "T026 [US3] Add skill workflow parity checks for precondition-safe edits in tests/integration_tests.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1 and Phase 2.
2. Complete Phase 3 (US1).
3. Validate US1 independently with quickstart single-edit checks.
4. Pause for review/demo before expanding scope.

### Incremental Delivery

1. Deliver US1 for single-edit safety.
2. Deliver US2 for batch execution semantics.
3. Deliver US3 for docs and skill parity.
4. Finish Phase 6 cross-cutting validation.

### Parallel Team Strategy

1. Team completes setup/foundational work together.
2. After Phase 2:
   - Engineer A: US1 implementation and tests.
   - Engineer B: US2 implementation and tests.
3. Engineer C (or A/B after merge): US3 docs/skills + parity tests.

---

## Notes

- `[P]` tasks touch independent files or can execute without waiting for sibling tasks.
- Every user-story task includes a `[US#]` label and explicit file path.
- Suggested MVP scope: **Phase 3 (US1) only** after foundational completion.
