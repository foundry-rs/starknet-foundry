use crate::{
    output_transform::{OutputTransform, PassByPrint},
    ScarbCommand, ScarbCommandError,
};
use scarb_metadata::{Metadata, MetadataCommandError, VersionPin};
use serde_json::Value;
use std::{borrow::Cow, io::BufRead};

impl From<ScarbCommandError> for MetadataCommandError {
    fn from(value: ScarbCommandError) -> Self {
        match value {
            ScarbCommandError::Io(io) => MetadataCommandError::Io(io),
            ScarbCommandError::ScarbError { stdout, stderr } => {
                MetadataCommandError::ScarbError { stdout, stderr }
            }
        }
    }
}

#[derive(Default, Clone)]
pub struct ScarbErrorPrettyPrint;

fn pretty_print<'a>(out: &'a [u8]) -> Cow<'a, [u8]> {
    let message = serde_json::from_slice::<'a, Value>(out).map_or_else(
        |_| String::from_utf8_lossy(out).to_string(),
        |value| match value["message"].as_str() {
            Some(message) if value["type"] == "error" => message.to_owned(),
            _ => serde_json::to_string_pretty(&value).unwrap(),
        },
    );

    let mut result = Vec::new();

    result.extend_from_slice(b"\nScarb returned error:\n");

    for line in message.lines() {
        result.extend(b"Scarb error:    ");
        result.extend(line.as_bytes());
        result.extend(b"\n");
    }

    result.extend(b"\n");

    result.into()
}

impl OutputTransform for ScarbErrorPrettyPrint {
    fn transform_stderr(stderr: &[u8]) -> Cow<'_, [u8]> {
        pretty_print(stderr)
    }
    fn transform_stdout(stdout: &[u8]) -> Cow<'_, [u8]> {
        pretty_print(stdout)
    }
}

pub struct ScarbMetadataCommand<Stdout = PassByPrint, Stderr = PassByPrint> {
    inner: ScarbCommand<Stdout, Stderr>,
    no_deps: bool,
}

impl<Stdout, Stderr> ScarbMetadataCommand<Stdout, Stderr> {
    pub(crate) fn new(inner: ScarbCommand<Stdout, Stderr>) -> Self {
        Self {
            inner,
            no_deps: false,
        }
    }

    /// Output information only about workspace members and don't fetch dependencies.
    pub fn no_deps(&mut self) -> &mut Self {
        self.no_deps = true;
        self
    }

    /// Pretty prints Scarb error
    ///
    /// It will override previously set transformers
    #[must_use]
    pub fn error_pretty_print(
        self,
    ) -> ScarbMetadataCommand<ScarbErrorPrettyPrint, ScarbErrorPrettyPrint>
    where
        Stdout: OutputTransform,
        Stderr: OutputTransform,
    {
        ScarbMetadataCommand {
            inner: self
                .inner
                .use_custom_print_stdout::<ScarbErrorPrettyPrint>()
                .use_custom_print_stderr::<ScarbErrorPrettyPrint>(),
            no_deps: self.no_deps,
        }
    }

    /// Runs configured `scarb metadata` and returns parsed `Metadata`.
    pub fn run(mut self) -> Result<Metadata, MetadataCommandError>
    where
        Stdout: OutputTransform,
        Stderr: OutputTransform,
    {
        self.inner
            .json()
            .args(["metadata", "--format-version"])
            .arg(VersionPin.numeric().to_string());

        if self.no_deps {
            self.inner.arg("--no-deps");
        }

        let output = self.inner.run()?;
        let data = output.stdout.as_slice();

        // copied from: https://github.com/software-mansion/scarb/blob/b54a1b1cd7b854111e527b50bb85a2c1e69bed32/scarb-metadata/src/command/metadata_command.rs#L154-L184
        let mut err = None;
        for line in BufRead::split(data, b'\n') {
            let line = line?;

            if !line.starts_with(br#"{"version":"#) {
                continue;
            }
            match serde_json::from_slice(&line) {
                Ok(metadata) => return Ok(metadata),
                Err(serde_err) => err = Some(serde_err.into()),
            }
        }

        Err(err.unwrap_or_else(|| MetadataCommandError::NotFound {
            stdout: String::from_utf8_lossy(data).into(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use indoc::formatdoc;
    use serde_json::json;

    use super::ScarbErrorPrettyPrint;
    use crate::output_transform::OutputTransform;

    #[test]
    fn is_pretty() {
        let message = json!({
            "type": "error",
            "message": formatdoc!(r#"
                multiline
                message
            "#)
        })
        .to_string();

        let stderr = ScarbErrorPrettyPrint::transform_stderr(message.as_bytes());
        let stdout = ScarbErrorPrettyPrint::transform_stdout(message.as_bytes());

        let expected = formatdoc!(
            r#"

                Scarb returned error:
                Scarb error:    multiline
                Scarb error:    message

            "#
        );

        let stdout: &str = &String::from_utf8_lossy(&stdout);
        let stderr: &str = &String::from_utf8_lossy(&stderr);

        assert_eq!(stdout, expected);
        assert_eq!(stderr, expected);
    }

    #[test]
    fn invalid_type_should_pretty_print_json() {
        let message = json!({
            "type": "definitly not error",
            "message": formatdoc!(r#"
                multiline
                message
            "#)
        });
        let one_line_message = message.to_string();

        let stderr = ScarbErrorPrettyPrint::transform_stderr(one_line_message.as_bytes());
        let stdout = ScarbErrorPrettyPrint::transform_stdout(one_line_message.as_bytes());

        let expected = formatdoc!(
            r#"

                Scarb returned error:
                Scarb error:    {{
                Scarb error:      "message": "multiline\nmessage\n",
                Scarb error:      "type": "definitly not error"
                Scarb error:    }}

            "#
        );

        let stdout: &str = &String::from_utf8_lossy(&stdout);
        let stderr: &str = &String::from_utf8_lossy(&stderr);

        assert_eq!(stdout, expected);
        assert_eq!(stderr, expected);
    }

    #[test]
    fn non_json_should_be_passed_by() {
        let message = formatdoc!(
            r#"
                multiline
                message
            "#
        );

        let stderr = ScarbErrorPrettyPrint::transform_stderr(message.as_bytes());
        let stdout = ScarbErrorPrettyPrint::transform_stdout(message.as_bytes());

        let expected = formatdoc!(
            r#"

                Scarb returned error:
                Scarb error:    multiline
                Scarb error:    message

            "#
        );

        let stdout: &str = &String::from_utf8_lossy(&stdout);
        let stderr: &str = &String::from_utf8_lossy(&stderr);

        assert_eq!(stdout, expected);
        assert_eq!(stderr, expected);
    }
}
