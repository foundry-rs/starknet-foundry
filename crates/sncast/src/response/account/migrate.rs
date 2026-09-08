use crate::{accounts::MigrationOutcome, response::cast_message::SncastCommandMessage};
use foundry_ui::styling;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct AccountMigrateResponse {
    migration_outcome: MigrationOutcome,
}

impl From<MigrationOutcome> for AccountMigrateResponse {
    fn from(migration_outcome: MigrationOutcome) -> Self {
        Self { migration_outcome }
    }
}

impl SncastCommandMessage for AccountMigrateResponse {
    fn text(&self) -> String {
        let builder = styling::OutputBuilder::new();
        let builder = match &self.migration_outcome {
            MigrationOutcome::NotRequired { version } => builder.success_message(&format!(
                "Accounts file is already the latest version {version}"
            )),
            MigrationOutcome::Performed {
                from,
                to,
                backup_path,
            } => builder
                .success_message(&format!(
                    "Accounts file migrated from version {from} to version {to}"
                ))
                .field("V1 Backup", backup_path.as_str()),
        };
        builder.build()
    }
}
