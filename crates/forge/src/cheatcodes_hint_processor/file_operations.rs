use anyhow::Context;
use cairo_felt::Felt252;
use cairo_lang_runner::casm_run::MemBuffer;
use cairo_lang_runner::short_string::as_cairo_short_string;
use cheatnet::cheatcodes::EnhancedHintError;

pub(super) fn parse_txt(
    buffer: &mut MemBuffer,
    file_path: Felt252,
) -> Result<(), EnhancedHintError> {
    let file_path_str = as_cairo_short_string(&file_path)
        .with_context(|| format!("Failed to convert {file_path} to str"))?;
    let content = std::fs::read_to_string(file_path_str)?;
    let result: Vec<&str> = content.trim().split_ascii_whitespace().collect();
    println!("ESSA:\n{result:?}");
    // buffer.write_arr(result.iter()).unwrap();
    Ok(())
}
