use super::{print::OutputFormat, structs::OutputLink};

#[allow(dead_code)]
const STARKSCAN_SEARCH: &'static str = "https://starkscan.co/search";
#[allow(dead_code)]
const VOYAGER_SEARCH: &'static str = "https://voyager.online/tx";

pub fn print_block_explorer_link_if_allowed<T: OutputLink>(
    result: &anyhow::Result<T>,
    output_format: OutputFormat,
) {
    if let (Ok(response), OutputFormat::Human) = (result, output_format) {
        let url = response.format_url(STARKSCAN_SEARCH);
        println!("\nVisit {url}\nto see transaction details");
    }
}
