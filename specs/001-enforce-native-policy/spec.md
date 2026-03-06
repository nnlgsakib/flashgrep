# Feature Specification: Enforce Native Policy Routing

**Feature Branch**: `001-enforce-native-policy`  
**Created**: 2026-03-06  
**Status**: Draft  
**Input**: User description: "the current system policy is not able to enforce the codeagents to use it nativekly ... make it more advanced ... Tool execution aborted"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Enforce Native Tool Usage (Priority: P1)

As a maintainer, I want policy enforcement to require code agents to use Flashgrep
native tools by default so sessions do not drift into unrelated host-native reads,
writes, or broad exploration.

**Why this priority**: Without strict enforcement, agents can ignore policy and
produce irrelevant or unsafe behavior.

**Independent Test**: Start a session with bootstrap metadata, attempt a non-native
tool route without a fallback gate, and verify the request is blocked with a typed
policy violation response.

**Acceptance Scenarios**:

1. **Given** a session with strict policy enabled, **When** an agent attempts a
   non-native route without an allowed gate, **Then** the system rejects it and
   returns a typed reason.
2. **Given** a valid fallback gate condition, **When** an agent uses a fallback
   route, **Then** the system permits it and records the declared reason code.

---

### User Story 2 - Detect and Recover from Policy Drift (Priority: P1)

As an operator, I want deterministic drift detection and recovery steps so I can
quickly restore compliant behavior when agents go off-policy.

**Why this priority**: Detection and recovery are required to keep automation
reliable in long-running or interrupted sessions.

**Independent Test**: Simulate policy drift in a session and verify the system emits
actionable diagnostics, exposes policy state, and supports forced reinjection.

**Acceptance Scenarios**:

1. **Given** an agent request that violates routing rules, **When** enforcement is
   evaluated, **Then** response metadata identifies the violation and recovery path.
2. **Given** a drifted session, **When** force bootstrap is executed, **Then**
   compliant policy state is restored and subsequent routing is enforced.

---

### User Story 3 - Keep Guidance and Behavior Aligned (Priority: P2)

As a power user, I want docs and skills to match the enforced policy behavior so
agent prompts and operational guidance produce predictable outcomes.

**Why this priority**: Mismatched guidance causes repeated off-policy behavior even
when enforcement exists.

**Independent Test**: Follow published docs/skills for native-first routing and
confirm observed behavior matches documented enforcement and fallback semantics.

**Acceptance Scenarios**:

1. **Given** updated policy docs and skills, **When** users follow them, **Then**
   agent routing remains native-first and compliant.
2. **Given** policy behavior changes, **When** release checks run, **Then** stale
   or contradictory guidance is detected before release.

---

### Edge Cases

- What happens when bootstrap metadata is missing or partially malformed?
- How does enforcement behave when index access is temporarily unavailable?
- What happens when an agent declares an unsupported fallback reason code?
- How is repeated violation behavior handled within a single session?
- What happens when strict policy metadata conflicts with legacy fields?

## Assumptions

- Native Flashgrep tool routes remain the default compliant path.
- Fallback usage is allowed only under explicit, typed gate conditions.
- Sessions can be force-reinjected with policy metadata when drift is detected.
- Existing clients consume structured policy metadata and reason codes.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST enforce native Flashgrep tool routing by default for
  discovery, read, and write workflows.
- **FR-002**: System MUST reject non-native routing attempts unless an allowed,
  explicit fallback gate is active.
- **FR-003**: System MUST return typed violation diagnostics for rejected requests,
  including reason code and recovery guidance.
- **FR-004**: System MUST support deterministic fallback admission when a declared
  fallback gate and reason code are valid.
- **FR-005**: System MUST expose policy metadata required to verify enforcement
  state and routing expectations in each session.
- **FR-006**: System MUST provide a force-refresh operation that re-injects policy
  and restores strict enforcement behavior.
- **FR-007**: System MUST preserve deterministic behavior for identical requests and
  session state, including consistent violation outcomes.
- **FR-008**: System MUST prevent unrelated codebase expansion behavior when user
  intent is policy enforcement rather than feature expansion.
- **FR-009**: System MUST provide auditable indicators when fallback is used,
  including gate identity and reason code.
- **FR-010**: System MUST keep docs and skills synchronized with enforcement
  behavior and fallback rules before release.

### Constitution Alignment Requirements *(mandatory)*

- **CA-001**: Feature MUST define deterministic output and typed error behavior for
  enforcement and fallback decisions.
- **CA-002**: Feature MUST use explicit controls for strict mode, fallback gates,
  and policy reinjection behavior.
- **CA-003**: Feature MUST define measurable reliability outcomes for policy
  enforcement under normal and drifted sessions.
- **CA-004**: Feature MUST keep Flashgrep-native routing as the primary path and
  tightly constrain host-native fallback usage.
- **CA-005**: Feature MUST include docs and skills updates for user-visible policy
  and routing behavior changes.

### Key Entities *(include if feature involves data)*

- **Policy Session State**: Runtime state describing whether strict policy is
  injected, active, and enforced.
- **Routing Decision**: Structured result indicating native route accepted, fallback
  accepted, or request rejected.
- **Fallback Gate Record**: Declared gate identifier and typed reason code for
  allowed fallback behavior.
- **Policy Violation Event**: Typed violation output with rejection reason and
  suggested recovery steps.
- **Guidance Artifact**: Skill and documentation content describing compliant
  routing behavior and troubleshooting flow.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In policy conformance testing, 100% of non-native requests without an
  allowed gate are rejected with typed diagnostics.
- **SC-002**: In fallback conformance testing, 100% of requests with valid gate and
  reason code are admitted and correctly labeled as fallback behavior.
- **SC-003**: In drift recovery testing, at least 95% of drifted sessions return to
  compliant routing within one forced reinjection cycle.
- **SC-004**: In release validation, 100% of policy-related docs and skills checks
  pass with no conflicting routing instructions.
