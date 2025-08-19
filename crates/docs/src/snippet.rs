use std::sync::LazyLock;

use regex::Regex;
use scarb_api::ScarbCommand;
use semver::VersionReq;
use serde::{Deserialize, Serialize};

static RE_SNCAST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new( r"(?ms)^(?:<!--\s*(?P<config>\{.*?\})\s*-->\n)?```shell\n\$ (?P<command>sncast .+?)\n```(?:\s*<details>\n<summary>Output:<\/summary>\n\n```shell\n(?P<output>[\s\S]+?)\n```[\s]*<\/details>)?").expect("Failed to create regex for sncast snippet")
});

static RE_SNFORGE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new( r"(?ms)^(?:<!--\s*(?P<config>\{.*?\})\s*-->\n)?```shell\n\$ (?P<command>snforge .+?)\n```(?:\s*<details>\n<summary>Output:<\/summary>\n\n```shell\n(?P<output>[\s\S]+?)\n```[\s]*<\/details>)?").expect("Failed to create regex for snforge snippet")
});

#[derive(Clone, Debug)]
pub struct SnippetType(String);

impl SnippetType {
    #[must_use]
    pub fn forge() -> Self {
        SnippetType("snforge".to_string())
    }

    #[must_use]
    pub fn sncast() -> Self {
        SnippetType("sncast".to_string())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn get_re(&self) -> &'static Regex {
        // The regex pattern is used to match the snippet, its config and the output. Example:
        // <!-- { content of snippet config JSON } -->
        // ```shell
        // $ snforge or sncast command with args...
        // ```
        // <details>
        // <summary>Output:</summary>
        // ```shell
        // Output of the command...
        // ```
        // </details>

        match self.as_str() {
            "snforge" => &RE_SNFORGE,
            "sncast" => &RE_SNCAST,
            _ => panic!("Regex for {} not found", self.as_str()),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(default)]
pub struct SnippetConfig {
    pub ignored: bool,
    pub package_name: Option<String>,
    pub ignored_output: bool,
    pub replace_network: bool,
    pub scarb_version: Option<VersionReq>,
}

#[derive(Deserialize)]
#[serde(default)]
struct SnippetConfigProxy {
    ignored: bool,
    package_name: Option<String>,
    ignored_output: bool,
    replace_network: bool,
    scarb_version: Option<VersionReq>,
}

impl Default for SnippetConfigProxy {
    fn default() -> Self {
        Self {
            ignored: false,
            package_name: None,
            ignored_output: false,
            replace_network: true,
            scarb_version: None,
        }
    }
}

impl Default for SnippetConfig {
    fn default() -> Self {
        Self {
            ignored: false,
            package_name: None,
            ignored_output: false,
            replace_network: true,
            scarb_version: None,
        }
    }
}

impl SnippetConfig {
    fn check_scarb_compatibility(&mut self) {
        if let Some(ref scarb_version_req) = self.scarb_version {
            let current_scarb_version = ScarbCommand::version()
                .run()
                .expect("Failed to get scarb version")
                .scarb;

            if !scarb_version_req.matches(&current_scarb_version) {
                self.ignored = true;
            }
        }
    }
}

impl<'de> Deserialize<'de> for SnippetConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let proxy = SnippetConfigProxy::deserialize(deserializer)?;

        let mut config = SnippetConfig {
            ignored: proxy.ignored,
            package_name: proxy.package_name,
            ignored_output: proxy.ignored_output,
            replace_network: proxy.replace_network,
            scarb_version: proxy.scarb_version,
        };

        config.check_scarb_compatibility();

        Ok(config)
    }
}

#[derive(Debug)]
pub struct Snippet {
    pub command: String,
    pub output: Option<String>,
    pub file_path: String,
    pub line_start: usize,
    pub snippet_type: SnippetType,
    pub config: SnippetConfig,
}

impl Snippet {
    #[must_use]
    pub fn to_command_args(&self) -> Vec<String> {
        let cleaned_command = self
            .command
            .lines()
            .collect::<Vec<&str>>()
            .join(" ")
            .replace(" \\", "");

        shell_words::split(&cleaned_command)
            .expect("Failed to parse snippet string")
            .into_iter()
            .map(|arg| arg.trim().to_string())
            .collect()
    }

    #[must_use]
    pub fn capture_package_from_output(&self) -> Option<String> {
        let re =
            Regex::new(r"Collected \d+ test\(s\) from ([a-zA-Z_][a-zA-Z0-9_]*) package").unwrap();

        re.captures_iter(self.output.as_ref()?)
            .filter_map(|caps| caps.get(1))
            .last()
            .map(|m| m.as_str().to_string())
    }
}
