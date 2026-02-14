## ADDED Requirements

### Requirement: Native bootstrap usage documentation
The documentation set MUST describe how agents invoke native Flashgrep bootstrap to receive skill injection without manual external skill loading.

#### Scenario: Bootstrap trigger guidance is documented
- **WHEN** users read Flashgrep agent documentation
- **THEN** documentation MUST include bootstrap trigger examples for `flashgrep-init` and `fgrep-boot`

#### Scenario: Bootstrap response behavior is documented
- **WHEN** users read bootstrap documentation
- **THEN** documentation MUST describe first-call injection and repeated-call idempotent behavior

### Requirement: Flashgrep-first tool selection guidance
The documentation set MUST explicitly guide agents to prioritize Flashgrep-native tools over generic grep/glob workflows for matching tasks.

#### Scenario: Guidance includes preferred tool order
- **WHEN** users read the skill or bootstrap guidance
- **THEN** it MUST recommend using `query`, `files`, `symbol`, `read_code`, and `write_code` as primary operations

#### Scenario: Efficient read/write guidance is present
- **WHEN** users review read/write recommendations
- **THEN** documentation MUST describe budgeted `read_code` usage and targeted `write_code` usage for token-efficient agent operation
