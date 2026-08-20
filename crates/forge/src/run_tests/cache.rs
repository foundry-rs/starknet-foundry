use camino::Utf8PathBuf;

pub const USC_CACHE_DIR: &str = "universal-sierra-compiler";

pub struct CacheConfig {
    pub cache_dir: Utf8PathBuf,
    pub usc_cache_dir: Utf8PathBuf,
}

impl CacheConfig {
    #[must_use]
    pub fn new(cache_dir: Utf8PathBuf) -> Self {
        let usc_cache_dir = cache_dir.join(USC_CACHE_DIR);
        Self {
            cache_dir,
            usc_cache_dir,
        }
    }
}
