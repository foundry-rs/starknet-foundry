use crate::helpers::configuration::AliasesConfig;
use crate::response::cast_message::SncastCommandMessage;
use conversions::string::IntoHexStr;
use foundry_ui::styling;
use itertools::Itertools;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct AliasEntryMessage {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Clone)]
pub struct AliasesListMessage {
    aliases: Vec<AliasEntryMessage>,
}

impl AliasesListMessage {
    #[must_use]
    pub fn new(aliases: &AliasesConfig) -> Self {
        let aliases = aliases
            .iter()
            .sorted_by_key(|(name, _)| *name)
            .map(|(key, value)| AliasEntryMessage {
                name: key.clone(),
                value: value.into_hex_string(),
            })
            .collect();

        Self { aliases }
    }
}

impl SncastCommandMessage for AliasesListMessage {
    fn text(&self) -> String {
        let mut builder = styling::OutputBuilder::new();

        if self.aliases.is_empty() {
            return builder
                .text_field(
                    "No aliases configured. For details on adding aliases, see: https://foundry-rs.github.io/starknet-foundry/starknet/aliases.html",
                )
                .build();
        }

        for alias in &self.aliases {
            builder = builder.field(&alias.name, &alias.value);
        }

        builder.build()
    }
}
