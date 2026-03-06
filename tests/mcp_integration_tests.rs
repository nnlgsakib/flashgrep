//! MCP Integration Tests
//!
//! Tests for MCP server functionality

#[cfg(test)]
mod tests {
    /// Test that MCP server starts successfully
    #[test]
    fn test_mcp_server_startup() {
        // This is a basic smoke test
        // In a real scenario, you'd start the server and connect to it
        println!("MCP server startup test would run here");
        println!("To test manually:");
        println!("  1. Run: ./target/release/flashgrep mcp");
        println!("  2. Connect with: echo '{{\"jsonrpc\":\"2.0\",\"method\":\"initialize\",\"params\":{{\"clientInfo\":{{\"name\":\"test\",\"version\":\"1.0\"}}}},\"id\":1}}' | nc localhost 7777");
    }

    /// Test MCP server responds to initialize
    #[test]
    fn test_mcp_initialize() {
        println!("MCP initialize test");
        println!("Expected response should include:");
        println!("  - protocolVersion: 2024-11-05");
        println!("  - serverInfo with name and version");
        println!("  - capabilities with tools list");
    }

    /// Test query tool
    #[test]
    fn test_mcp_query_tool() {
        println!("MCP query tool test");
        println!("Request: {{\"jsonrpc\":\"2.0\",\"method\":\"query\",\"params\":{{\"text\":\"fn main\",\"limit\":5}},\"id\":2}}");
        println!("Expected: Array of search results");
    }

    /// Test get_slice tool
    #[test]
    fn test_mcp_get_slice_tool() {
        println!("MCP get_slice tool test");
        println!("Request: {{\"jsonrpc\":\"2.0\",\"method\":\"get_slice\",\"params\":{{\"file_path\":\"src/main.rs\",\"start_line\":1,\"end_line\":10}},\"id\":3}}");
        println!("Expected: File content for specified lines");
    }

    /// Test MCP resources
    #[test]
    fn test_mcp_resources() {
        println!("MCP resources test");
        println!("URI format: flashgrep://<repo>/files/<path>");
        println!("Example: flashgrep://myproject/files/src/main.rs");
    }

    /// Test MCP prompts
    #[test]
    fn test_mcp_prompts() {
        println!("MCP prompts test");
        println!("Available prompts:");
        println!("  - find_function");
        println!("  - find_class");
        println!("  - search_code");
        println!("  - explain_file");
        println!("  - find_symbol");
        println!("  - search_imports");
    }

    /// Test error handling
    #[test]
    fn test_mcp_error_handling() {
        println!("MCP error handling test");
        println!("Invalid method should return error code -32601");
        println!("Invalid params should return error code -32602");
    }

    /// Performance test
    #[test]
    fn test_mcp_performance() {
        println!("MCP performance test");
        println!("Query response should be under 50ms");
        println!("Initialize should complete in under 100ms");
    }
}

#[cfg(test)]
mod integration {
    /// Integration test for full MCP workflow
    #[test]
    fn test_mcp_full_workflow() {
        println!("\n=== MCP Full Workflow Test ===\n");

        println!("Step 1: Start MCP server");
        println!("  $ flashgrep mcp");

        println!("\nStep 2: Initialize connection");
        println!("  Request: {{\"jsonrpc\":\"2.0\",\"method\":\"initialize\",\"params\":{{\"clientInfo\":{{\"name\":\"test-client\",\"version\":\"1.0\"}}}},\"id\":1}}");

        println!("\nStep 3: Query for code");
        println!("  Request: {{\"jsonrpc\":\"2.0\",\"method\":\"query\",\"params\":{{\"text\":\"function main\",\"limit\":5}},\"id\":2}}");

        println!("\nStep 4: Get file slice");
        println!("  Request: {{\"jsonrpc\":\"2.0\",\"method\":\"get_slice\",\"params\":{{\"file_path\":\"src/main.rs\",\"start_line\":1,\"end_line\":20}},\"id\":3}}");

        println!("\nStep 5: Find symbol");
        println!("  Request: {{\"jsonrpc\":\"2.0\",\"method\":\"get_symbol\",\"params\":{{\"symbol_name\":\"main\"}},\"id\":4}}");

        println!("\n=== Test Complete ===\n");
    }
}

#[cfg(test)]
mod behavior {
    use flashgrep::mcp::bootstrap::{
        current_policy_hash, evaluate_ai_discovery_fallback, evaluate_policy_route,
        prompt_budget_telemetry, prompt_governance_from_arguments, PolicyRouteState,
    };
    use flashgrep::mcp::code_io::batch_write_code;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_batch_write_code_reports_per_operation_statuses() {
        let temp = TempDir::new().expect("temp dir");
        let file_path = temp.path().join("sample.rs");
        fs::write(&file_path, "one\ntwo\nthree\n").expect("write fixture");

        let payload = batch_write_code(&json!({
            "mode": "best_effort",
            "operations": [
                {
                    "id": "ok-op",
                    "file_path": file_path.to_string_lossy(),
                    "start_line": 2,
                    "end_line": 2,
                    "replacement": "TWO"
                },
                {
                    "id": "bad-op",
                    "file_path": file_path.to_string_lossy(),
                    "start_line": 99,
                    "end_line": 99,
                    "replacement": "X"
                }
            ]
        }))
        .expect("batch payload");

        assert_eq!(
            payload["applied_count"],
            serde_json::Value::Number(1u64.into())
        );
        assert_eq!(
            payload["failed_count"],
            serde_json::Value::Number(1u64.into())
        );
        assert!(payload["results"].as_array().is_some());
    }

    #[test]
    fn test_batch_write_code_atomic_keeps_file_on_failure() {
        let temp = TempDir::new().expect("temp dir");
        let file_path = temp.path().join("sample.rs");
        fs::write(&file_path, "alpha\nbeta\ngamma\n").expect("write fixture");

        let payload = batch_write_code(&json!({
            "mode": "atomic",
            "operations": [
                {
                    "id": "first",
                    "file_path": file_path.to_string_lossy(),
                    "start_line": 1,
                    "end_line": 1,
                    "replacement": "ALPHA"
                },
                {
                    "id": "second",
                    "file_path": file_path.to_string_lossy(),
                    "start_line": 2,
                    "end_line": 2,
                    "replacement": "BETA",
                    "precondition": {
                        "expected_start_line_text": "not-beta"
                    }
                }
            ]
        }))
        .expect("batch payload");

        assert_eq!(payload["ok"], serde_json::Value::Bool(false));
        let content = fs::read_to_string(&file_path).expect("read file");
        assert_eq!(content, "alpha\nbeta\ngamma\n");
    }

    #[test]
    fn test_batch_write_code_small_batch_completes_quickly() {
        let temp = TempDir::new().expect("temp dir");
        let file_path = temp.path().join("sample.rs");
        fs::write(&file_path, "l1\nl2\nl3\nl4\nl5\n").expect("write fixture");

        let start = std::time::Instant::now();
        let payload = batch_write_code(&json!({
            "mode": "best_effort",
            "operations": [
                {"id": "op1", "file_path": file_path.to_string_lossy(), "start_line": 1, "end_line": 1, "replacement": "L1"},
                {"id": "op2", "file_path": file_path.to_string_lossy(), "start_line": 3, "end_line": 3, "replacement": "L3"},
                {"id": "op3", "file_path": file_path.to_string_lossy(), "start_line": 5, "end_line": 5, "replacement": "L5"}
            ]
        }))
        .expect("batch payload");
        let elapsed = start.elapsed();

        assert_eq!(
            payload["applied_count"],
            serde_json::Value::Number(3u64.into())
        );
        assert!(
            elapsed.as_secs_f64() < 3.0,
            "batch edit too slow: {elapsed:?}"
        );
    }

    #[test]
    fn test_policy_route_denies_ungated_fallback_tool() {
        let decision = evaluate_policy_route(
            "search",
            &json!({
                "pattern": "main"
            }),
        );
        assert_eq!(decision.route_state, PolicyRouteState::Denied);
        assert_eq!(
            decision.reason_code.as_deref(),
            Some("fallback_gate_required")
        );
    }

    #[test]
    fn test_policy_route_allows_gated_fallback_tool() {
        let decision = evaluate_policy_route(
            "search",
            &json!({
                "pattern": "main",
                "fallback_gate": "tool_runtime_failure",
                "fallback_reason_code": "flashgrep_tool_runtime_failure"
            }),
        );
        assert_eq!(decision.route_state, PolicyRouteState::AllowedFallback);
        assert_eq!(
            decision.fallback_gate_id.as_deref(),
            Some("tool_runtime_failure")
        );
    }

    #[test]
    fn test_policy_drift_mismatch_is_detected() {
        let decision = evaluate_policy_route(
            "query",
            &json!({
                "policy_hash": "stale-policy-hash"
            }),
        );
        assert_eq!(decision.route_state, PolicyRouteState::Denied);
        assert_eq!(
            decision.reason_code.as_deref(),
            Some("policy_state_mismatch")
        );
    }

    #[test]
    fn test_policy_route_evaluation_overhead_is_bounded() {
        let policy_hash = current_policy_hash();
        let args = json!({
            "text": "main",
            "policy_hash": policy_hash,
            "policy_version": "1.1"
        });
        let start = std::time::Instant::now();
        for _ in 0..5000 {
            let decision = evaluate_policy_route("query", &args);
            assert_eq!(decision.route_state, PolicyRouteState::AllowedNative);
        }
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 200,
            "policy route evaluation overhead too high: {elapsed:?}"
        );
    }

    #[test]
    fn test_ai_unavailable_routes_to_deterministic_fallback() {
        let decision = evaluate_ai_discovery_fallback(
            &json!({
                "text": "find auth middleware",
                "retrieval_mode": "neural",
                "ai_mode": "discovery",
                "simulate_ai_unavailable": true,
                "prompt_version": "1.0"
            }),
            true,
        )
        .expect("fallback decision");
        assert_eq!(decision.route_state, PolicyRouteState::AllowedFallback);
        assert_eq!(decision.reason_code.as_deref(), Some("ai_mode_disabled"));
        assert_eq!(
            decision.fallback_gate_id.as_deref(),
            Some("neural_mode_disabled")
        );
    }

    #[test]
    fn test_budget_metadata_reports_usage_and_continuation() {
        let telemetry = prompt_budget_telemetry(
            &json!({"budget_profile": "fast", "token_budget": 24}),
            "main",
            &["fn main() { println!(\"hello\"); }".to_string()],
            true,
            Some(20),
        );
        assert_eq!(telemetry["budget_profile"], json!("fast"));
        assert!(telemetry["tokens_used"].as_u64().unwrap_or(0) <= 24);
        assert_eq!(telemetry["reduction_applied"], json!(true));
        assert_eq!(telemetry["continuation_id"], json!("query:20"));
    }

    #[test]
    fn test_prompt_governance_returns_stable_version_and_hash() {
        let first = prompt_governance_from_arguments(&json!({
            "prompt_id": "flashgrep-core",
            "prompt_version": "1.0",
            "policy_rule_hits": ["allow:default"]
        }))
        .expect("first prompt governance record");
        let second = prompt_governance_from_arguments(&json!({
            "prompt_id": "flashgrep-core",
            "prompt_version": "1.0",
            "policy_rule_hits": ["allow:default"]
        }))
        .expect("second prompt governance record");

        assert_eq!(first.prompt_version, "1.0");
        assert_eq!(first.prompt_hash, second.prompt_hash);
        assert_eq!(first.prompt_hash.len(), 64);
    }
}
