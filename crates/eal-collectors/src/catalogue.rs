//! Publish the versioned native authoring catalogue without configuring or executing tools.
use eal_collector_contracts::Catalogue;
use std::sync::OnceLock;

/// The checked-in descriptor is the current producer contract for authoring clients.
pub const NATIVE_JSON: &str = include_str!("../../../contracts/collectors/native.json");

pub fn native() -> &'static Catalogue {
    static CATALOGUE: OnceLock<Catalogue> = OnceLock::new();
    CATALOGUE.get_or_init(|| {
        let value = eal_runtime::acquisition::strict_json(NATIVE_JSON)
            .expect("the bundled collector catalogue is strict JSON");
        let catalogue: Catalogue = serde_json::from_value(value)
            .expect("the bundled collector catalogue has its wire shape");
        catalogue
            .validate()
            .expect("the bundled collector catalogue is valid");
        catalogue
    })
}
