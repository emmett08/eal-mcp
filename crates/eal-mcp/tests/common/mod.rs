use serde_json::{Value, json};
use std::collections::BTreeSet;

/// Check the public native inventories and closed request-schema boundaries.
pub fn assert_inventory(actual: Value, registered: bool) {
    let tools = actual.as_array().unwrap();
    let expected = if registered {
        vec![
            "eal_sources",
            "eal_find_claims",
            "eal_assess_known",
            "eal_get_known_packet",
        ]
    } else {
        vec![
            "eal_describe",
            "eal_format",
            "eal_validate",
            "eal_plan",
            "eal_collect",
            "eal_collect_claim",
            "eal_execute",
            "eal_reason",
            "eal_compile_aspic",
            "eal_explain",
            "eal_packet",
            "eal_grounded",
        ]
    };
    assert_eq!(tools.len(), expected.len());
    assert_eq!(
        tools
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect::<BTreeSet<_>>(),
        expected.into_iter().collect()
    );
    for tool in tools {
        assert_eq!(tool["inputSchema"]["type"], "object");
        assert_eq!(tool["inputSchema"]["additionalProperties"], false);
        if ["eal_packet", "eal_assess_known", "eal_get_known_packet"]
            .iter()
            .any(|name| tool["name"] == *name)
        {
            let budget = &tool["inputSchema"]["properties"]["packet_limits"];
            assert_eq!(budget["type"], "object");
            assert_eq!(budget["additionalProperties"], false);
            assert_eq!(budget["properties"]["max_bytes"]["minimum"], 4096);
            assert_eq!(budget["properties"]["max_bytes"]["maximum"], 65536);
            assert_eq!(budget["properties"]["max_statement_bytes"]["minimum"], 1);
            assert_eq!(budget["properties"]["max_statement_bytes"]["maximum"], 4096);
        }
    }
}
pub fn invalid_calls(registered: bool) -> Vec<(&'static str, Value)> {
    if registered {
        vec![
            ("eal_validate", json!({"source":""})),
            ("eal_sources", json!({"limit":-1})),
            ("eal_sources", json!({"limit":1.0})),
            ("eal_assess_known", json!({"entry_id":"synthetic"})),
            (
                "eal_assess_known",
                json!({"entry_id":"foreign","claim":"outcome"}),
            ),
        ]
    } else {
        vec![
            ("unknown_tool", json!({})),
            ("eal_validate", json!({})),
            ("eal_plan", json!({"source":""})),
            ("eal_describe", json!({"unexpected":true})),
            ("eal_collect", json!({"source":"","context":[]})),
            ("eal_grounded", json!({"arguments":[],"attacks":"bad"})),
        ]
    }
}
