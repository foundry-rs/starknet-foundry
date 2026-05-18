use crate::helpers::configuration::AliasesConfig;
use conversions::string::IntoHexStr;
use foundry_ui::Message;
use foundry_ui::styling;
use itertools::Itertools;
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Serialize, Clone, Debug)]
pub struct AliasEntryMessage {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Clone, Debug)]
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

impl Message for AliasesListMessage {
    fn text(&self) -> String {
        let mut builder = styling::OutputBuilder::new();

        if self.aliases.is_empty() {
            // TODO: consider expanding error message to include instructions on how to add aliases,
            //  either via link to docs, or by referring to potential future `alias add` command.
            return builder.text_field("No aliases configured").build();
        }

        for alias in &self.aliases {
            builder = builder.field(&alias.name, &alias.value);
        }

        builder.build()
    }

    fn json(&self) -> Value {
        json!(self)
    }
}
