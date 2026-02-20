use crate::e2e::ledger::{automation, setup_speculos};
use crate::helpers::constants::URL;
use crate::helpers::runner::runner;
use docs::snippet::SnippetType;
use docs::utils::{get_nth_ancestor, print_ignored_snippet_message};
use docs::validation::{extract_snippets_from_directory, extract_snippets_from_file};
use shared::test_utils::output_assert::assert_stdout_contains;
use std::sync::Arc;
use tempfile::TempDir;

const DOCS_SNIPPETS_PORT_BASE: u16 = 4006;

async fn setup_speculos_automation(client: &Arc<speculos_client::SpeculosClient>, args: &[&str]) {
    if args.contains(&"get-public-key") && !args.contains(&"--no-display") {
        client
            .automation(&[automation::APPROVE_PUBLIC_KEY])
            .await
            .unwrap();
    } else if args.contains(&"sign-hash") {
        client
            .automation(&[
                automation::ENABLE_BLIND_SIGN,
                automation::APPROVE_BLIND_SIGN_HASH,
            ])
            .await
            .unwrap();
    } else if args.contains(&"--ledger-path") && args.contains(&"create") {
        client
            .automation(&[automation::APPROVE_PUBLIC_KEY])
            .await
            .unwrap();
    }
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_ledger_docs_snippets() {
    let root_dir_path = get_nth_ancestor(2);
    let ledger_appendix_dir = root_dir_path.join("docs/src/appendix/sncast/ledger");
    let ledger_guide_path = root_dir_path.join("docs/src/starknet/ledger.md");

    let snippet_type = SnippetType::sncast();

    let appendix_snippets = extract_snippets_from_directory(&ledger_appendix_dir, &snippet_type)
        .expect("Failed to extract ledger appendix snippets");
    let guide_snippets = extract_snippets_from_file(&ledger_guide_path, &snippet_type)
        .expect("Failed to extract ledger guide snippets");

    let snippets: Vec<_> = appendix_snippets
        .into_iter()
        .chain(guide_snippets)
        .collect();

    let tempdir = TempDir::new().expect("Unable to create a temporary directory");
    std::fs::write(tempdir.path().join("accounts.json"), "{}").unwrap();

    let target_accounts_json_path = tempdir.path().join("accounts.json");

    let mut port_offset = 0u16;
    let mut validated = 0usize;
    let mut ignored = 0usize;

    for snippet in &snippets {
        if snippet.config.ignored {
            print_ignored_snippet_message(snippet);
            ignored += 1;
            continue;
        }

        if !snippet.config.requires_ledger {
            continue;
        }

        let args = snippet.to_command_args();
        let mut args: Vec<&str> = args.iter().map(String::as_str).collect();

        // remove "sncast" from the args
        args.remove(0);

        args.insert(0, "--accounts-file");
        args.insert(1, target_accounts_json_path.to_str().unwrap());

        if snippet.config.replace_network {
            let network_pos = args.iter().position(|arg| *arg == "--network");
            if let Some(network_pos) = network_pos {
                args[network_pos] = "--url";
                args[network_pos + 1] = URL;
            }
        }

        let port = DOCS_SNIPPETS_PORT_BASE + port_offset;
        port_offset += 1;

        let (client, url) = setup_speculos(port);
        setup_speculos_automation(&client, &args).await;

        let snapbox = runner(&args)
            .env("LEDGER_EMULATOR_URL", &url)
            .current_dir(tempdir.path());
        let output = snapbox.assert().success();

        if let Some(expected_stdout) = &snippet.output
            && !snippet.config.ignored_output
        {
            assert_stdout_contains(output, expected_stdout);
        }

        validated += 1;
    }

    println!(
        "Finished validation of ledger docs snippets\nValidated: {validated}, Ignored: {ignored}"
    );
}
