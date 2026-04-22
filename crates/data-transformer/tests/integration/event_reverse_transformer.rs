use data_transformer::{ReverseTransformEventError, reverse_transform_event};
use starknet_rust::core::types::contract::{
    AbiEntry, AbiEvent, AbiEventEnum, AbiEventStruct, EventField, EventFieldKind, TypedAbiEvent,
    UntypedAbiEvent,
};
use starknet_rust::core::utils::get_selector_from_name;
use starknet_types_core::felt::Felt;

#[test]
fn test_event_struct_with_data_only_field() {
    let abi = vec![typed_struct_event(
        "test::ValueEmitted",
        &[("value", "core::felt252", EventFieldKind::Data)],
    )];

    let result = reverse_transform_event(&[selector("ValueEmitted")], &[felt(42)], &abi).unwrap();

    assert_eq!(result, "ValueEmitted { value: 0x2a }");
}

#[test]
fn test_event_struct_with_key_field() {
    let abi = vec![typed_struct_event(
        "test::Transfer",
        &[
            (
                "from",
                "core::starknet::contract_address::ContractAddress",
                EventFieldKind::Key,
            ),
            (
                "to",
                "core::starknet::contract_address::ContractAddress",
                EventFieldKind::Key,
            ),
            ("amount", "core::felt252", EventFieldKind::Data),
        ],
    )];

    let result =
        reverse_transform_event(&[selector("Transfer"), felt(1), felt(2)], &[felt(3)], &abi)
            .unwrap();

    assert_eq!(
        result,
        "Transfer { from: ContractAddress(0x1), to: ContractAddress(0x2), amount: 0x3 }"
    );
}

#[test]
fn test_top_level_event_enum_variant() {
    let abi = vec![
        typed_enum_event(
            "test::Event",
            &[("ValueEmitted", "test::ValueEmitted", EventFieldKind::Nested)],
        ),
        typed_struct_event(
            "test::ValueEmitted",
            &[("value", "core::felt252", EventFieldKind::Data)],
        ),
    ];

    let result = reverse_transform_event(&[selector("ValueEmitted")], &[felt(42)], &abi).unwrap();

    assert_eq!(result, "Event::ValueEmitted(ValueEmitted { value: 0x2a })");
}

#[test]
fn test_flattened_event_enum_variant() {
    let abi = vec![
        typed_enum_event(
            "test::Event",
            &[(
                "AccessControlEvent",
                "test::AccessControlEvent",
                EventFieldKind::Flat,
            )],
        ),
        typed_enum_event(
            "test::AccessControlEvent",
            &[("RoleGranted", "test::RoleGranted", EventFieldKind::Nested)],
        ),
        typed_struct_event(
            "test::RoleGranted",
            &[("role", "core::felt252", EventFieldKind::Data)],
        ),
    ];

    let result = reverse_transform_event(&[selector("RoleGranted")], &[felt(7)], &abi).unwrap();

    assert_eq!(
        result,
        "AccessControlEvent::RoleGranted(RoleGranted { role: 0x7 })"
    );
}

#[test]
fn test_nested_event_field() {
    let abi = vec![
        typed_struct_event(
            "test::OuterEvent",
            &[
                ("inner", "test::InnerEvent", EventFieldKind::Nested),
                ("tail", "core::felt252", EventFieldKind::Data),
            ],
        ),
        typed_struct_event(
            "test::InnerEvent",
            &[("value", "core::felt252", EventFieldKind::Data)],
        ),
    ];

    let result = reverse_transform_event(
        &[selector("OuterEvent"), selector("InnerEvent")],
        &[felt(1), felt(2)],
        &abi,
    )
    .unwrap();

    assert_eq!(
        result,
        "OuterEvent { inner: InnerEvent { value: 0x1 }, tail: 0x2 }"
    );
}

#[test]
fn test_untyped_event_is_unsupported() {
    let abi = vec![AbiEntry::Event(AbiEvent::Untyped(UntypedAbiEvent {
        name: "test::LegacyEvent".to_string(),
        inputs: vec![],
    }))];

    let error = reverse_transform_event(&[selector("LegacyEvent")], &[], &abi).unwrap_err();

    assert!(matches!(
        error,
        ReverseTransformEventError::UnsupportedUntypedEvent(name) if name == "test::LegacyEvent"
    ));
}

#[test]
fn test_malformed_event_abi_returns_invalid_abi() {
    let abi = vec![typed_enum_event(
        "test::Event",
        &[("Missing", "test::MissingEvent", EventFieldKind::Nested)],
    )];

    let error = reverse_transform_event(&[selector("Missing")], &[], &abi).unwrap_err();

    assert!(matches!(error, ReverseTransformEventError::InvalidAbi));
}

#[test]
fn test_nonstandard_event_returns_not_found() {
    let abi = vec![typed_struct_event(
        "test::ValueEmitted",
        &[("value", "core::felt252", EventFieldKind::Data)],
    )];

    let error = reverse_transform_event(&[felt(0x123)], &[felt(0x456)], &abi).unwrap_err();

    assert!(matches!(
        error,
        ReverseTransformEventError::EventNotFound(selector) if selector == felt(0x123)
    ));
}

fn typed_struct_event(name: &str, members: &[(&str, &str, EventFieldKind)]) -> AbiEntry {
    AbiEntry::Event(AbiEvent::Typed(TypedAbiEvent::Struct(AbiEventStruct {
        name: name.to_string(),
        members: members
            .iter()
            .map(|(name, ty, kind)| EventField {
                name: name.to_string(),
                r#type: ty.to_string(),
                kind: kind.clone(),
            })
            .collect(),
    })))
}

fn typed_enum_event(name: &str, variants: &[(&str, &str, EventFieldKind)]) -> AbiEntry {
    AbiEntry::Event(AbiEvent::Typed(TypedAbiEvent::Enum(AbiEventEnum {
        name: name.to_string(),
        variants: variants
            .iter()
            .map(|(name, ty, kind)| EventField {
                name: name.to_string(),
                r#type: ty.to_string(),
                kind: kind.clone(),
            })
            .collect(),
    })))
}

fn selector(name: &str) -> Felt {
    get_selector_from_name(name).unwrap()
}

fn felt(value: u64) -> Felt {
    Felt::from(value)
}
