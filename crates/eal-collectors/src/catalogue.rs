//! Version-selected native contracts without configuring or executing tools.
use eal_collector_contracts::Catalogue;
use std::sync::OnceLock;

/// Published compatibility catalogue, available until the next major native-contract release.
pub const NATIVE_JSON: &str = include_str!("../../../contracts/collectors/native.json");
pub const NATIVE_V2_JSON: &str = include_str!("../../../contracts/collectors/native-v2.json");
pub const NATIVE_V3_JSON: &str = include_str!("../../../contracts/collectors/native-v3.json");
/// Preserve the existing default. New consumers select version 2 explicitly.
pub fn native() -> &'static Catalogue {
    native_version("1").expect("bundled v1 catalogue")
}
pub fn native_version(version: &str) -> Result<&'static Catalogue, String> {
    static V1: OnceLock<Catalogue> = OnceLock::new();
    static V2: OnceLock<Catalogue> = OnceLock::new();
    static V3: OnceLock<Catalogue> = OnceLock::new();
    let (slot, text) = match version {
        "1" => (&V1, NATIVE_JSON),
        "2" => (&V2, NATIVE_V2_JSON),
        "3" => (&V3, NATIVE_V3_JSON),
        _ => return Err("Unsupported native catalogue version".into()),
    };
    Ok(slot.get_or_init(|| {
        let value = eal_collector_contracts::strict_json(
            text.as_bytes(),
            eal_collector_contracts::JsonLimits::default(),
        )
        .expect("bundled catalogue strict JSON");
        let catalogue: Catalogue =
            serde_json::from_value(value).expect("bundled catalogue wire shape");
        catalogue.validate().expect("bundled catalogue valid");
        catalogue
    }))
}
