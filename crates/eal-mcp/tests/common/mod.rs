use serde_json::Value;

/// The Python oracle stays frozen. Remove exactly the documented additive
/// native descriptors, then compare every remaining field with that oracle.
pub fn assert_inventory_compatibility(mut actual: Value, expected: &Value, registered: bool) {
    let tools = actual.as_array_mut().unwrap();
    assert_eq!(
        tools.len(),
        expected.as_array().unwrap().len() + usize::from(registered)
    );
    let new_tools = tools
        .iter()
        .filter(|tool| tool["name"] == "eal_get_known_packet")
        .count();
    assert_eq!(new_tools, usize::from(registered));
    tools.retain(|tool| tool["name"] != "eal_get_known_packet");
    for tool in tools {
        if ["eal_packet", "eal_assess_known"]
            .iter()
            .any(|name| tool["name"] == *name)
        {
            let budget = tool["inputSchema"]["properties"]
                .as_object_mut()
                .unwrap()
                .remove("packet_limits")
                .unwrap();
            assert_eq!(budget["type"], "object");
            assert_eq!(budget["additionalProperties"], false);
            let properties = budget["properties"].as_object().unwrap();
            assert_eq!(properties.len(), 2);
            assert_eq!(properties["max_bytes"]["minimum"], 4096);
            assert_eq!(properties["max_bytes"]["maximum"], 65536);
            assert_eq!(properties["max_statement_bytes"]["minimum"], 1);
            assert_eq!(properties["max_statement_bytes"]["maximum"], 4096);
        }
    }
    assert_eq!(actual, *expected);
}
