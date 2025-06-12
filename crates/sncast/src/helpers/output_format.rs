use foundry_ui::OutputFormat;

#[must_use]
pub fn output_format_from_json_flag(json: bool) -> OutputFormat {
    if json {
        OutputFormat::Json
    } else {
        OutputFormat::Human
    }
}
