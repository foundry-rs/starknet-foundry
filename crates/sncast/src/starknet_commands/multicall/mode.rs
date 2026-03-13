/// Determines mode of `multicall` source.
#[derive(Copy, Clone, Debug)]
pub enum MulticallMode {
    /// Multicall defined in a `.toml` file.
    File,
    /// Multicall defined via CLI.
    Cli,
}

impl MulticallMode {
    /// Returns the registry lookup key for a given raw value, if it should be treated as an id.
    ///
    /// - For [`MulticallSource::File`], every value is considered a potential id, so the input
    ///   string is always returned.
    /// - For [`MulticallSource::Cli`], only values starting with `@` are considered ids, and the
    ///   returned key has the `@` prefix stripped.
    #[must_use]
    pub fn id_key<'a>(self, value: &'a str) -> Option<&'a str> {
        match self {
            MulticallMode::File => Some(value),
            MulticallMode::Cli => value.strip_prefix('@'),
        }
    }
}
