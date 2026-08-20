use camino::Utf8PathBuf;

pub const USC_CACHE_DIR: &str = "universal-sierra-compiler";

pub struct CacheConfig {
    pub cache_dir: Utf8PathBuf,
    pub usc_cache_dir: Utf8PathBuf,
}
