use clap::Args;
use sncast::helpers::configuration::CastConfig;
use sncast::response::alias::list::AliasesListMessage;

#[derive(Args, Debug)]
#[command(
    name = "list",
    about = "List aliases from the current effective config"
)]
pub struct List;

pub fn list(config: &CastConfig) -> AliasesListMessage {
    AliasesListMessage::new(&config.aliases)
}
