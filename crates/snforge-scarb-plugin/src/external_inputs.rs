use cairo_lang_macro::fingerprint;
use std::env;
use std::hash::{Hash, Hasher};
use xxhash_rust::xxh3::Xxh3;

/// All external inputs that influence the compilation should be added here.
#[derive(Hash)]
pub struct ExternalInput {
    pub forge_test_filter: Option<String>,
}

impl ExternalInput {
    pub fn get() -> Self {
        Self {
            forge_test_filter: env::var("SNFORGE_TEST_FILTER").ok(),
        }
    }
}

/// This function implements a callback that Scarb will use to determine
/// whether Cairo code depending on this macro should be recompiled.
/// The callback is concerned with informing Scarb about changes to inputs that don't come from Scarb directly,
/// like the `SNFORGE_TEST_FILTER` environmental variable.
///
/// Warning: Removing this callback can break incremental compilation with this macro!
#[fingerprint]
fn test_filter_fingerprint() -> u64 {
    // The hashes need to be consistent across different runs.
    // Thus, we cannot use the default hasher, which is rng-seeded.
    let mut hasher = Xxh3::default();
    ExternalInput::get().hash(&mut hasher);
    hasher.finish()
}
