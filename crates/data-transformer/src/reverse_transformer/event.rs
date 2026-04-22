use crate::reverse_transformer::transform::{ReverseTransformer, TransformationError};
use crate::reverse_transformer::types::{Enum, Struct, StructField, Type};
use conversions::serde::deserialize::BufferReadError;
use starknet_rust::core::types::contract::{
    AbiEntry, AbiEvent, AbiEventEnum, AbiEventStruct, EventField, EventFieldKind, TypedAbiEvent,
};
use starknet_rust::core::utils::get_selector_from_name;
use starknet_types_core::felt::Felt;

#[derive(Debug, thiserror::Error)]
pub enum ReverseTransformEventError {
    #[error(r#"Typed event matching emitted selector "{0:#x}" not found in ABI of the contract"#)]
    EventNotFound(Felt),
    #[error("untyped event `{0}` is unsupported by reverse transformer")]
    UnsupportedUntypedEvent(String),
    #[error("event `{0}` is unsupported by reverse transformer")]
    UnsupportedEvent(String),
    #[error("abi is invalid")]
    InvalidAbi,
    #[error(transparent)]
    TransformationError(#[from] TransformationError),
}

#[derive(Clone, Copy)]
enum SelectorSource {
    Consume,
    Provided(Felt),
    None,
}

enum InternalEventError {
    SelectorMismatch,
    Other(ReverseTransformEventError),
}

impl From<ReverseTransformEventError> for InternalEventError {
    fn from(value: ReverseTransformEventError) -> Self {
        Self::Other(value)
    }
}

/// Transforms a set of event keys and data into a Cairo-like string representation of the event.
pub fn reverse_transform_event(
    keys: &[Felt],
    data: &[Felt],
    abi: &[AbiEntry],
) -> Result<String, ReverseTransformEventError> {
    let selector = keys.first().copied().ok_or_else(|| {
        ReverseTransformEventError::TransformationError(TransformationError::BufferReaderError(
            BufferReadError::EndOfBuffer,
        ))
    })?;

    let mut first_error = None;

    for typed_event in top_level_typed_events(abi) {
        let mut transformer = EventReverseTransformer::new(keys, data, abi);

        match transformer.transform_typed_event(typed_event, SelectorSource::Consume) {
            Ok(decoded) if transformer.keys.is_empty() && transformer.data.is_empty() => {
                return Ok(decoded.to_string());
            }
            Ok(_) => {
                if first_error.is_none() {
                    first_error = Some(ReverseTransformEventError::InvalidAbi);
                }
            }
            Err(InternalEventError::SelectorMismatch) => {}
            Err(InternalEventError::Other(error)) => {
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
    }

    if let Some(error) = first_error {
        return Err(error);
    }

    if let Some(untyped_name) = matching_untyped_event_name(selector, abi) {
        return Err(ReverseTransformEventError::UnsupportedUntypedEvent(
            untyped_name.to_string(),
        ));
    }

    Err(ReverseTransformEventError::EventNotFound(selector))
}

struct EventReverseTransformer<'a> {
    abi: &'a [AbiEntry],
    keys: &'a [Felt],
    data: &'a [Felt],
}

impl<'a> EventReverseTransformer<'a> {
    fn new(keys: &'a [Felt], data: &'a [Felt], abi: &'a [AbiEntry]) -> Self {
        Self { abi, keys, data }
    }

    fn transform_typed_event(
        &mut self,
        typed_event: &TypedAbiEvent,
        selector_source: SelectorSource,
    ) -> Result<Type, InternalEventError> {
        match typed_event {
            TypedAbiEvent::Struct(abi_event_struct) => {
                self.transform_event_struct(abi_event_struct, selector_source)
            }
            TypedAbiEvent::Enum(abi_event_enum) => {
                self.transform_event_enum(abi_event_enum, selector_source)
            }
        }
    }

    fn transform_event_struct(
        &mut self,
        abi_event_struct: &AbiEventStruct,
        selector_source: SelectorSource,
    ) -> Result<Type, InternalEventError> {
        self.ensure_selector_matches(&abi_event_struct.name, selector_source)?;

        let fields = abi_event_struct
            .members
            .iter()
            .map(|member| {
                Ok(StructField {
                    name: member.name.clone(),
                    value: self.transform_event_field(member)?,
                })
            })
            .collect::<Result<Vec<_>, InternalEventError>>()?;

        Ok(Type::Struct(Struct {
            name: short_name(&abi_event_struct.name),
            fields,
        }))
    }

    fn transform_event_enum(
        &mut self,
        abi_event_enum: &AbiEventEnum,
        selector_source: SelectorSource,
    ) -> Result<Type, InternalEventError> {
        let selector = self.resolve_selector(selector_source)?;

        for variant in &abi_event_enum.variants {
            match variant.kind {
                EventFieldKind::Flat => {
                    if self.event_type_matches_selector(&variant.r#type, selector)? {
                        return self.transform_event_type(
                            &variant.r#type,
                            SelectorSource::Provided(selector),
                        );
                    }
                }
                EventFieldKind::Nested | EventFieldKind::Data | EventFieldKind::Key => {
                    if selector_matches_name(selector, &variant.name)? {
                        let argument = if variant.r#type == "()" {
                            None
                        } else {
                            Some(Box::new(self.transform_event_type(
                                &variant.r#type,
                                SelectorSource::Provided(selector),
                            )?))
                        };

                        return Ok(Type::Enum(Enum {
                            name: short_name(&abi_event_enum.name),
                            variant: variant.name.clone(),
                            argument,
                        }));
                    }
                }
            }
        }

        Err(InternalEventError::SelectorMismatch)
    }

    fn transform_event_field(&mut self, field: &EventField) -> Result<Type, InternalEventError> {
        match field.kind {
            EventFieldKind::Key => self.transform_from_keys(&field.r#type).map_err(Into::into),
            EventFieldKind::Data => self.transform_from_data(&field.r#type).map_err(Into::into),
            EventFieldKind::Nested => {
                self.transform_event_type(&field.r#type, SelectorSource::Consume)
            }
            EventFieldKind::Flat => self.transform_event_type(&field.r#type, SelectorSource::None),
        }
    }

    fn transform_event_type(
        &mut self,
        type_name: &str,
        selector_source: SelectorSource,
    ) -> Result<Type, InternalEventError> {
        let typed_event =
            find_typed_event(self.abi, type_name).ok_or(ReverseTransformEventError::InvalidAbi)?;
        self.transform_typed_event(typed_event, selector_source)
    }

    fn transform_from_keys(&mut self, expr: &str) -> Result<Type, ReverseTransformEventError> {
        let mut reverse_transformer = ReverseTransformer::new(self.keys, self.abi);
        let value = reverse_transformer.parse_and_transform(expr)?;
        self.keys = reverse_transformer.into_remaining();
        Ok(value)
    }

    fn transform_from_data(&mut self, expr: &str) -> Result<Type, ReverseTransformEventError> {
        let mut reverse_transformer = ReverseTransformer::new(self.data, self.abi);
        let value = reverse_transformer.parse_and_transform(expr)?;
        self.data = reverse_transformer.into_remaining();
        Ok(value)
    }

    fn ensure_selector_matches(
        &mut self,
        event_name: &str,
        selector_source: SelectorSource,
    ) -> Result<(), InternalEventError> {
        let expected_selector = selector_from_name(event_name)?;
        let selector = self.resolve_selector(selector_source)?;

        if selector == expected_selector {
            Ok(())
        } else {
            Err(InternalEventError::SelectorMismatch)
        }
    }

    fn resolve_selector(
        &mut self,
        selector_source: SelectorSource,
    ) -> Result<Felt, InternalEventError> {
        match selector_source {
            SelectorSource::Consume => {
                let [head, tail @ ..] = self.keys else {
                    return Err(ReverseTransformEventError::TransformationError(
                        TransformationError::BufferReaderError(BufferReadError::EndOfBuffer),
                    )
                    .into());
                };
                self.keys = tail;
                Ok(*head)
            }
            SelectorSource::Provided(selector) => Ok(selector),
            SelectorSource::None => Err(ReverseTransformEventError::UnsupportedEvent(
                "event enum without selector".to_string(),
            )
            .into()),
        }
    }

    fn event_type_matches_selector(
        &self,
        type_name: &str,
        selector: Felt,
    ) -> Result<bool, ReverseTransformEventError> {
        let typed_event =
            find_typed_event(self.abi, type_name).ok_or(ReverseTransformEventError::InvalidAbi)?;

        match typed_event {
            TypedAbiEvent::Struct(abi_event_struct) => {
                Ok(selector_matches_name(selector, &abi_event_struct.name)?)
            }
            TypedAbiEvent::Enum(abi_event_enum) => {
                for variant in &abi_event_enum.variants {
                    match variant.kind {
                        EventFieldKind::Flat => {
                            if self.event_type_matches_selector(&variant.r#type, selector)? {
                                return Ok(true);
                            }
                        }
                        EventFieldKind::Nested | EventFieldKind::Data | EventFieldKind::Key => {
                            if selector_matches_name(selector, &variant.name)? {
                                return Ok(true);
                            }
                        }
                    }
                }
                Ok(false)
            }
        }
    }
}

fn matching_untyped_event_name(selector: Felt, abi: &[AbiEntry]) -> Option<&str> {
    abi.iter().find_map(|entry| match entry {
        AbiEntry::Event(AbiEvent::Untyped(untyped_event))
            if selector_matches_name(selector, &untyped_event.name).ok()? =>
        {
            Some(untyped_event.name.as_str())
        }
        _ => None,
    })
}

fn find_typed_event<'a>(abi: &'a [AbiEntry], type_name: &str) -> Option<&'a TypedAbiEvent> {
    abi.iter().find_map(|entry| match entry {
        AbiEntry::Event(AbiEvent::Typed(typed_event))
            if typed_event_name(typed_event) == type_name =>
        {
            Some(typed_event)
        }
        _ => None,
    })
}

/// Extracts the typed *top-level* events from the ABI.
///
/// Sometimes, the ABI contains helper events (e.g. from components)
/// that are not emitted directly by the contract and appear only as payloads of other events.
///
/// In reverse transformation, we only care about *top-level* events,
/// i.e. such that are emitted directly and thus don't appear inside other events.
fn top_level_typed_events(abi: &[AbiEntry]) -> impl Iterator<Item = &TypedAbiEvent> {
    // All types ever referenced inside any typed event
    let all_types_in_events = abi
        .iter()
        .filter_map(|entry| match entry {
            AbiEntry::Event(AbiEvent::Typed(typed_event)) => Some(typed_event),
            _ => None,
        })
        .flat_map(|typed_event| match typed_event {
            TypedAbiEvent::Struct(abi_event_struct) => abi_event_struct
                .members
                .iter()
                .map(|field| field.r#type.as_str())
                .collect::<Vec<_>>(),
            TypedAbiEvent::Enum(abi_event_enum) => abi_event_enum
                .variants
                .iter()
                .map(|field| field.r#type.as_str())
                .collect::<Vec<_>>(),
        })
        .collect::<Vec<_>>();

    // Select only top-level events, i.e. such that are not referenced by other events.
    abi.iter().filter_map(move |entry| match entry {
        AbiEntry::Event(AbiEvent::Typed(typed_event))
            if !all_types_in_events.contains(&typed_event_name(typed_event)) =>
        {
            Some(typed_event)
        }
        _ => None,
    })
}

/// Extracts the name of the typed event.
fn typed_event_name(typed_event: &TypedAbiEvent) -> &str {
    match typed_event {
        TypedAbiEvent::Struct(abi_event_struct) => &abi_event_struct.name,
        TypedAbiEvent::Enum(abi_event_enum) => &abi_event_enum.name,
    }
}

/// Checks if the given `selector` refers to the `name`.
fn selector_matches_name(selector: Felt, name: &str) -> Result<bool, ReverseTransformEventError> {
    Ok(selector == selector_from_name(name)?)
}

/// Calculates the selector value for a given `name`.
fn selector_from_name(name: &str) -> Result<Felt, ReverseTransformEventError> {
    get_selector_from_name(&short_name(name)).map_err(|_| ReverseTransformEventError::InvalidAbi)
}

fn short_name(name: &str) -> String {
    name.split("::")
        .last()
        .expect("path should not be empty")
        .to_string()
}
