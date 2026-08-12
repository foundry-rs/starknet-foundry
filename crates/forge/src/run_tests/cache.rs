use camino::Utf8PathBuf;

pub struct CacheConfig {
    pub cache_dir: Utf8PathBuf,
    pub usc_cache_dir: Option<Utf8PathBuf>,
}
