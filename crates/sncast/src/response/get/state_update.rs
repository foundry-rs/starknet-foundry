use crate::response::cast_message::SncastCommandMessage;
use foundry_ui::styling::OutputBuilder;
use serde::Serialize;
use starknet_rust::core::types::{
    ContractStorageDiffItem, DeclaredClassItem, DeployedContractItem, MaybePreConfirmedStateUpdate,
    MigratedCompiledClassItem, NonceUpdate, PreConfirmedStateUpdate, ReplacedClassItem, StateDiff,
    StateUpdate,
};

#[derive(Clone, Serialize)]
pub struct StateUpdateResponse(pub MaybePreConfirmedStateUpdate);

impl SncastCommandMessage for StateUpdateResponse {
    fn text(&self) -> String {
        let builder = OutputBuilder::new()
            .success_message("State update retrieved")
            .blank_line();

        let builder = match &self.0 {
            MaybePreConfirmedStateUpdate::Update(update) => {
                // Take the concrete type so the exhaustive destructure turns any
                // field added upstream into a compile error instead of silently
                // dropping it.
                let StateUpdate {
                    block_hash,
                    old_root,
                    new_root,
                    state_diff,
                } = update;
                append_state_diff(
                    builder
                        .padded_felt_field("Block Hash", block_hash)
                        .padded_felt_field("Old Root", old_root)
                        .padded_felt_field("New Root", new_root),
                    state_diff,
                )
            }
            MaybePreConfirmedStateUpdate::PreConfirmedUpdate(update) => {
                let PreConfirmedStateUpdate {
                    old_root,
                    state_diff,
                } = update;
                let builder = builder.if_some(old_root.as_ref(), |builder, old_root| {
                    builder.padded_felt_field("Old Root", old_root)
                });
                append_state_diff(builder, state_diff)
            }
        };

        builder.build()
    }
}

fn append_state_diff(builder: OutputBuilder, state_diff: &StateDiff) -> OutputBuilder {
    let StateDiff {
        storage_diffs,
        deprecated_declared_classes,
        declared_classes,
        migrated_compiled_classes,
        deployed_contracts,
        replaced_classes,
        nonces,
    } = state_diff;

    let builder = append_storage_diffs(builder, storage_diffs);
    let builder = append_nonces(builder, nonces);
    let builder = append_deployed_contracts(builder, deployed_contracts);
    let builder = append_declared_classes(builder, declared_classes);
    let builder = append_migrated_compiled_classes(
        builder,
        migrated_compiled_classes.as_deref().unwrap_or_default(),
    );
    let builder = if deprecated_declared_classes.is_empty() {
        builder
    } else {
        builder
            .blank_line()
            .felt_list_field("Deprecated Declared Classes", deprecated_declared_classes)
    };
    append_replaced_classes(builder, replaced_classes)
}

fn append_storage_diffs(
    mut builder: OutputBuilder,
    storage_diffs: &[ContractStorageDiffItem],
) -> OutputBuilder {
    if storage_diffs.is_empty() {
        return builder;
    }
    builder = builder.blank_line().text_field("Storage Diffs");
    for ContractStorageDiffItem {
        address,
        storage_entries,
    } in storage_diffs
    {
        builder = builder
            .with_indent(2)
            .padded_felt_field("Contract Address", address);
        for entry in storage_entries {
            builder = builder
                .with_indent(4)
                .padded_felt_field("Key", &entry.key)
                .padded_felt_field("Value", &entry.value);
        }
    }
    builder.with_indent(0)
}

fn append_nonces(mut builder: OutputBuilder, nonces: &[NonceUpdate]) -> OutputBuilder {
    if nonces.is_empty() {
        return builder;
    }
    builder = builder.blank_line().text_field("Nonces");
    for NonceUpdate {
        contract_address,
        nonce,
    } in nonces
    {
        builder = builder
            .with_indent(2)
            .padded_felt_field("Contract Address", contract_address)
            .felt_field("Nonce", nonce);
    }
    builder.with_indent(0)
}

fn append_deployed_contracts(
    mut builder: OutputBuilder,
    deployed_contracts: &[DeployedContractItem],
) -> OutputBuilder {
    if deployed_contracts.is_empty() {
        return builder;
    }
    builder = builder.blank_line().text_field("Deployed Contracts");
    for DeployedContractItem {
        address,
        class_hash,
    } in deployed_contracts
    {
        builder = builder
            .with_indent(2)
            .padded_felt_field("Address", address)
            .padded_felt_field("Class Hash", class_hash);
    }
    builder.with_indent(0)
}

fn append_declared_classes(
    mut builder: OutputBuilder,
    declared_classes: &[DeclaredClassItem],
) -> OutputBuilder {
    if declared_classes.is_empty() {
        return builder;
    }
    builder = builder.blank_line().text_field("Declared Classes");
    for DeclaredClassItem {
        class_hash,
        compiled_class_hash,
    } in declared_classes
    {
        builder = builder
            .with_indent(2)
            .padded_felt_field("Class Hash", class_hash)
            .padded_felt_field("Compiled Class Hash", compiled_class_hash);
    }
    builder.with_indent(0)
}

fn append_migrated_compiled_classes(
    mut builder: OutputBuilder,
    migrated_compiled_classes: &[MigratedCompiledClassItem],
) -> OutputBuilder {
    if migrated_compiled_classes.is_empty() {
        return builder;
    }
    builder = builder.blank_line().text_field("Migrated Compiled Classes");
    for MigratedCompiledClassItem {
        class_hash,
        compiled_class_hash,
    } in migrated_compiled_classes
    {
        builder = builder
            .with_indent(2)
            .padded_felt_field("Class Hash", class_hash)
            .padded_felt_field("Compiled Class Hash", compiled_class_hash);
    }
    builder.with_indent(0)
}

fn append_replaced_classes(
    mut builder: OutputBuilder,
    replaced_classes: &[ReplacedClassItem],
) -> OutputBuilder {
    if replaced_classes.is_empty() {
        return builder;
    }
    builder = builder.blank_line().text_field("Replaced Classes");
    for ReplacedClassItem {
        contract_address,
        class_hash,
    } in replaced_classes
    {
        builder = builder
            .with_indent(2)
            .padded_felt_field("Contract Address", contract_address)
            .padded_felt_field("Class Hash", class_hash);
    }
    builder.with_indent(0)
}
