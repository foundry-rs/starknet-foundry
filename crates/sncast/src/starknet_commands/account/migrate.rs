use anyhow::Result;
use camino::Utf8PathBuf;
use clap::Args;

use sncast::accounts::AccountRepository;
use sncast::response::account::migrate::AccountMigrateResponse;

#[derive(Args, Debug)]
#[command(about = "Migrate an accounts file to the latest schema version")]
pub struct Migrate;

pub fn migrate(accounts_file: &Utf8PathBuf) -> Result<AccountMigrateResponse> {
    let repository = AccountRepository::new(accounts_file.clone());
    repository.load().map_err(|error| anyhow::anyhow!(error))?;
    let result = repository
        .mutate(|_| Ok(()))
        .map_err(|error| anyhow::anyhow!(error))?;

    Ok(AccountMigrateResponse {
        migrated: result.migrated_from_v1,
        backup: result
            .migrated_from_v1
            .then(|| repository.v1_backup_path().to_string()),
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn migrates_v1_and_reports_backup() {
        let directory = tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(directory.path().join("accounts.json")).unwrap();
        fs::write(
            &path,
            r#"{"alpha-sepolia":{"alice":{"public_key":"0x1","private_key":"0x2"}}}"#,
        )
        .unwrap();

        let response = migrate(&path).unwrap();

        assert!(response.migrated);
        assert!(response.backup.is_some());
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&path).unwrap()).unwrap()
                ["version"],
            2
        );
    }
}
