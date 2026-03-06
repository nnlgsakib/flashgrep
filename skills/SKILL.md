# Flashgrep MCP Skill (Compact DSL)

FORMAT v1
GOAL token-efficient, deterministic, Flashgrep-first routing

BOOTSTRAP
CALL bootstrap_skill|flashgrep-init|flashgrep_init|fgrep-boot|fgrep_boot
RECOMMENDED {"name":"flashgrep_init","arguments":{"compact":true}}
EXPECT first=status:injected repeat=status:already_injected invalid=error:invalid_trigger
SOURCE default=embedded override=allow_repo_override:true fallback=repo_override_unavailable

POLICY_METADATA
REQUIRE policy_strength enforcement_mode payload_source bootstrap_state preferred_tools fallback_rules compliance_checks
SEARCH_ROUTING default=neural_first fallback=lexical_deterministic
AI_GOVERNANCE require=ai_mode|budget_profile|prompt_version expose=prompt_hash|policy_rule_hits

TOOL_ORDER
DISCOVERY query(retrieval_mode=neural) -> query(retrieval_mode=lexical) -> get_symbol -> read_code -> get_slice
NL_DISCOVERY ask(retrieval_mode=neural) -> review_evidence -> optional ask(retrieval_mode=lexical)
FILES glob|files|list_files
EDIT write_code|batch_write_code (after read_code/get_slice validation)
HEALTH stats
FS fs_create|fs_read|fs_write|fs_list|fs_stat|fs_copy|fs_move|fs_remove

FALLBACK_GATES
ALLOW neural_mode_disabled
ALLOW neural_provider_failure
ALLOW neural_no_relevant_matches
ALLOW exact_match_required
ALLOW query_parse_constraints
ALLOW flashgrep_index_unavailable
ALLOW flashgrep_operation_not_supported
ALLOW flashgrep_tool_runtime_failure
ALLOW repo_override_unavailable
REQUIRE fallback_gate + fallback_reason_code for non-native fallback tool usage

NATIVE_TOOL_POLICY
BAN grep rg find cat sed shell_glob Read Write Glob Grep
EXCEPTION only_when_fallback_gate_active

QUERY
ARGS text(required) mode(smart|literal|regex) retrieval_mode(neural|lexical) ai_mode(discovery|synthesis|planning|off) budget_profile(fast|balanced|deep) prompt_version prompt_hash case_sensitive regex_flags include exclude context limit
RULE neural-first for discovery intents
RULE if neural fails/unavailable/non-relevant then lexical fallback
RULE exact-match workloads use mode=literal|regex with gate=exact_match_required
RULE prompt policy hits must be typed (`policy_rule_hits`: allow|deny|escalate) and denials return `policy_denied`

ASK
ARGS question(required) retrieval_mode(neural|lexical) include exclude context limit output(text|json)
RULE use ask for natural-language codebase questions
RULE answer must cite evidence paths/lines from returned snippets
RULE if neural path has no confident evidence, retry ask with retrieval_mode=lexical

READ
USE read_code for bounded reads
BOUNDS max_lines max_bytes max_tokens
CONTINUE continuation_start_line until continuation.completed=true

WRITE
USE write_code minimal-range replacement
USE batch_write_code for deterministic multi-edit jobs
PRECONDITION expected_file_hash|expected_start_line_text|expected_end_line_text
ON_ERROR precondition_failed => re-read and retry

WORKFLOW discovery
STEP query text="<intent>" retrieval_mode=neural limit=20
STEP if no_results_or_failure then query text="<intent>" retrieval_mode=lexical limit=20
STEP optional get_symbol symbol_name="<name>"
STEP read_code file_path="<path>" start_line=<n> max_lines=80

WORKFLOW ask_nl
STEP ask question="<natural language question>" retrieval_mode=neural limit=8 include="src/**/*.rs"
STEP if no_confident_evidence then ask question="<same question>" retrieval_mode=lexical limit=12
STEP verify top evidence with read_code/get_slice before edits

WORKFLOW exact_lookup
STEP query text="<literal_or_regex>" mode=literal|regex limit=50
STEP get_slice file_path="<path>" start_line=<n> end_line=<m>

WORKFLOW edit_file
TASK edit_file
FILE src/auth.rs
FIND fn login
REPLACE add rate_limit check
STEP read_code file_path="src/auth.rs" symbol_name="login"
STEP write_code file_path="src/auth.rs" start_line=<n> end_line=<m> replacement="..." 
STEP read_code file_path="src/auth.rs" start_line=<n> max_lines=80

WORKFLOW batch_edit
STEP read_code file_path="<path>" start_line=<n> max_lines=120
STEP batch_write_code mode="atomic|best_effort" operations=[...]
STEP read_code file_path="<path>" start_line=<n> max_lines=120

FS_WORKFLOW
STEP fs_create path="notes/todo.txt" parents=true
STEP fs_write path="notes/todo.txt" overwrite=true content="..."
STEP fs_stat path="notes/todo.txt"

COMPLIANCE_RECOVERY
STEP bootstrap_skill arguments={"force":true,"compact":true}
STEP verify policy_metadata.search_routing + fallback_gates
STEP verify denied fallback requests return error=policy_denied with typed reason_code
STEP resume with query neural-first then lexical fallback

GUARDRAILS
RULE keep outputs bounded (limit/offset/budgets)
RULE deterministic sorting for automation (path asc)
RULE do not invent results; empty set is valid
RULE keep neural candidate window bounded
RULE emit prompt governance (`prompt_hash`, `policy_rule_hits`) and budget telemetry (`tokens_used`, `reduction_applied`)
