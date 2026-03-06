---

description: "Task list for enforcing native policy routing"
---

# Tasks: Enforce Native Policy Routing

**Input**: Design documents from `/specs/001-enforce-native-policy/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/

**Tests**: Include test tasks because this feature changes routing behavior, typed errors, and recovery semantics.

**Organization**: Tasks are grouped by user story so each story can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Task can run in parallel (different files, no dependency on incomplete tasks)
- **[Story]**: User story label (`[US1]`, `[US2]`, `[US3]`)
- Every task includes an explicit file path

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare policy-enforcement fixtures and baseline validation scaffolding.

- [X] T001 Create policy enforcement test fixtures in `tests/fixtures/policy_enforcement/`
- [X] T002 [P] Add bootstrap payload fixture assertions scaffold in `tests/mcp_integration_tests.rs`
- [X] T003 [P] Add docs/skills parity scaffold tests in `tests/integration_tests.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build core policy decision primitives used by all user stories.

**⚠️ CRITICAL**: No user story implementation starts until this phase is complete.

- [X] T004 Add policy decision and violation reason-code constants in `src/mcp/safety.rs`
- [X] T005 [P] Implement shared route-decision data model fields in `src/mcp/bootstrap.rs`
- [X] T006 [P] Implement fallback-gate validation helpers in `src/mcp/bootstrap.rs`
- [X] T007 Wire deterministic policy decision mapping for MCP tools in `src/mcp/mod.rs`
- [X] T008 Wire stdio policy decision plumbing and typed denials in `src/mcp/stdio.rs`
- [X] T009 Add policy metadata field parity helpers in `src/mcp/skill.rs`

**Checkpoint**: Foundation ready for story-level implementation.

---

## Phase 3: User Story 1 - Enforce Native Tool Usage (Priority: P1) 🎯 MVP

**Goal**: Enforce native-first routing and reject ungated non-native routes with typed diagnostics.

**Independent Test**: Start a strict session, attempt an ungated non-native route, and verify deterministic denial with typed reason code and recovery hint.

### Tests for User Story 1

- [X] T010 [P] [US1] Add strict-route deny tests for ungated non-native paths in `src/mcp/bootstrap.rs`
- [X] T011 [P] [US1] Add typed denial payload tests for MCP routing in `src/mcp/mod.rs`
- [X] T012 [US1] Add end-to-end strict routing tests in `tests/mcp_integration_tests.rs`

### Implementation for User Story 1

- [X] T013 [US1] Enforce deny-by-default routing policy in `src/mcp/bootstrap.rs`
- [X] T014 [US1] Return deterministic `policy_denied` payload shape in `src/mcp/mod.rs`
- [X] T015 [US1] Align stdio tool-call denial behavior with typed reason codes in `src/mcp/stdio.rs`
- [X] T016 [US1] Extend policy metadata for native route families in `src/mcp/skill.rs`

**Checkpoint**: US1 is independently functional and testable.

---

## Phase 4: User Story 2 - Detect and Recover from Policy Drift (Priority: P1)

**Goal**: Detect policy drift deterministically and restore strict state through forced reinjection.

**Independent Test**: Simulate session drift and verify diagnostics expose drift cause, then force reinjection and confirm strict routing is restored.

### Tests for User Story 2

- [X] T017 [P] [US2] Add policy-drift detection tests for metadata mismatch in `src/mcp/bootstrap.rs`
- [X] T018 [P] [US2] Add fallback reason-code validation tests in `src/mcp/bootstrap.rs`
- [X] T019 [US2] Add force-reinjection recovery integration tests in `tests/mcp_integration_tests.rs`

### Implementation for User Story 2

- [X] T020 [US2] Implement drift detection checks and typed drift reasons in `src/mcp/bootstrap.rs`
- [X] T021 [US2] Implement force-refresh policy reinjection behavior in `src/mcp/stdio.rs`
- [X] T022 [US2] Propagate restored policy state fields in `src/mcp/skill.rs` and `src/mcp/mod.rs`
- [X] T023 [US2] Add auditable fallback gate and reason-code emission in `src/mcp/bootstrap.rs`

**Checkpoint**: US2 is independently functional and testable.

---

## Phase 5: User Story 3 - Keep Guidance and Behavior Aligned (Priority: P2)

**Goal**: Keep README, skill directives, and troubleshooting/release docs synchronized with enforced policy behavior.

**Independent Test**: Validate docs/skills parity checks and confirm guidance matches runtime enforcement and fallback semantics.

### Tests for User Story 3

- [X] T024 [P] [US3] Add README/runtime policy-field parity tests in `tests/integration_tests.rs`
- [X] T025 [P] [US3] Add skills/runtime route-order parity tests in `tests/integration_tests.rs`
- [X] T026 [US3] Add release-doc parity checks in `tests/integration_tests.rs`

### Implementation for User Story 3

- [X] T027 [US3] Update native-routing and fallback-gate guidance in `README.md`
- [X] T028 [US3] Update strict routing and recovery directives in `skills/SKILL.md`
- [X] T029 [US3] Update drift troubleshooting guidance in `docs/bootstrap-policy-troubleshooting.md`
- [X] T030 [US3] Update release parity gate checklist in `docs/release-readiness-checklist.md`

**Checkpoint**: US3 guidance parity is independently verifiable.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final verification across stories, performance goals, and quickstart validation.

- [X] T031 [P] Run full policy enforcement regression tests in `src/mcp/bootstrap.rs` and `tests/mcp_integration_tests.rs`
- [X] T032 [P] Add/adjust enforcement overhead regression assertions in `tests/mcp_integration_tests.rs`
- [X] T033 Run quickstart validation and record outcomes in `specs/001-enforce-native-policy/quickstart.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: Starts immediately.
- **Phase 2 (Foundational)**: Depends on Phase 1 and blocks user-story work.
- **Phase 3 (US1)**: Depends on Phase 2 and defines MVP behavior.
- **Phase 4 (US2)**: Depends on Phase 2 and can begin after US1 contracts are stable.
- **Phase 5 (US3)**: Depends on US1 + US2 behavior stabilization for accurate parity updates.
- **Phase 6 (Polish)**: Depends on all user stories selected for delivery.

### User Story Dependencies

- **US1 (P1)**: Independent after foundational phase.
- **US2 (P1)**: Independent after foundational phase; consumes same policy primitives as US1.
- **US3 (P2)**: Depends on final enforcement behavior from US1 and US2.

### Within Each User Story

- Tests MUST be written and fail before implementation for behavior changes.
- Implement shared/contract behavior before transport wiring updates.
- Finish and validate each story independently before moving to lower priority.

### Dependency Graph

- `Phase1 -> Phase2 -> US1 -> US2 -> US3 -> Phase6`

---

## Parallel Execution Examples

## Parallel Example: User Story 1

```bash
Task: "T010 [US1] Add strict-route deny tests for ungated non-native paths in src/mcp/bootstrap.rs"
Task: "T011 [US1] Add typed denial payload tests for MCP routing in src/mcp/mod.rs"
```

## Parallel Example: User Story 2

```bash
Task: "T017 [US2] Add policy-drift detection tests for metadata mismatch in src/mcp/bootstrap.rs"
Task: "T018 [US2] Add fallback reason-code validation tests in src/mcp/bootstrap.rs"
```

## Parallel Example: User Story 3

```bash
Task: "T024 [US3] Add README/runtime policy-field parity tests in tests/integration_tests.rs"
Task: "T025 [US3] Add skills/runtime route-order parity tests in tests/integration_tests.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1 and Phase 2.
2. Complete Phase 3 (US1).
3. Validate strict native routing behavior independently.
4. Pause for review before drift recovery and doc parity scope.

### Incremental Delivery

1. Deliver US1 (native enforcement + typed denials).
2. Deliver US2 (drift detection + force recovery).
3. Deliver US3 (docs/skills parity gates).
4. Complete Phase 6 final validation.

### Parallel Team Strategy

1. Team completes setup and foundational work together.
2. After Phase 2:
   - Engineer A: US1 behavior + tests.
   - Engineer B: US2 drift/recovery + tests.
3. Engineer C (or follow-up) updates US3 guidance and parity checks.

---

## Notes

- `[P]` tasks are parallelizable when they touch different files or independent assertions.
- Every user-story task includes `[US#]` and an explicit file path.
- Suggested MVP scope: **Phase 3 (US1)** after foundational completion.
