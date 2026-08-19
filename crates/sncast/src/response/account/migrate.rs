use crate::response::cast_message::SncastCommandMessage;
use foundry_ui::styling;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct AccountMigrateResponse {
    pub migrated: bool,
    pub backup: Option<String>,
}

impl SncastCommandMessage for AccountMigrateResponse {
    fn text(&self) -> String {
        let message = if self.migrated {
            "Accounts file migrated to version 2"
        } else {
            "Accounts file is already version 2"
        };
        styling::OutputBuilder::new()
            .success_message(message)
            .if_some(self.backup.as_ref(), |builder, backup| {
                builder.field("V1 Backup", backup)
            })
            .build()
    }
}
