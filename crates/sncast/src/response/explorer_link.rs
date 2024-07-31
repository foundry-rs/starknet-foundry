use super::{print::OutputFormat, structs::OutputLink};

const STARKSCAN_SEARCH: &str = "https://starkscan.co/search";

pub fn print_block_explorer_link_if_allowed<T: OutputLink>(
    result: &anyhow::Result<T>,
    output_format: OutputFormat,
    search_url: &Option<String>,
) {
    if let (Ok(response), OutputFormat::Human) = (result, output_format) {
        let service = match search_url {
            None => STARKSCAN_SEARCH,
            Some(ref url) => url.trim_end_matches('/'),
        };

        let url = response.format_url(service);
        println!("\nVisit {url}\nto see transaction details");
    }
}
