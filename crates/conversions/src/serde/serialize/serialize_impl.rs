use super::{BufferWriter, CairoSerialize};
use crate::{byte_array::ByteArray, IntoConv};
use blockifier::execution::entry_point::{CallEntryPoint, CallType};
use starknet::core::types::{ContractErrorData, FieldElement, TransactionExecutionErrorData};
use starknet_api::core::EthAddress;
use starknet_api::{
    core::{ClassHash, ContractAddress, EntryPointSelector, Nonce},
    deprecated_contract_class::EntryPointType,
    transaction::Calldata,
};
use starknet_types_core::felt::Felt as Felt252;
use std::{
    cell::{Ref, RefCell},
    rc::Rc,
    sync::Arc,
};

impl CairoSerialize for CallEntryPoint {
    fn serialize(&self, output: &mut BufferWriter) {
        self.entry_point_type.serialize(output);
        self.entry_point_selector.serialize(output);
        self.calldata.serialize(output);
        self.storage_address.serialize(output);
        self.caller_address.serialize(output);
        self.call_type.serialize(output);
    }
}

impl CairoSerialize for ContractErrorData {
    fn serialize(&self, output: &mut BufferWriter) {
        self.revert_error.serialize(output);
    }
}

impl CairoSerialize for TransactionExecutionErrorData {
    fn serialize(&self, output: &mut BufferWriter) {
        self.transaction_index.serialize(output);
        self.execution_error.serialize(output);
    }
}

impl CairoSerialize for anyhow::Error {
    fn serialize(&self, output: &mut BufferWriter) {
        self.to_string().serialize(output);
    }
}

impl CairoSerialize for Calldata {
    fn serialize(&self, output: &mut BufferWriter) {
        self.0.serialize(output);
    }
}

impl CairoSerialize for EntryPointType {
    fn serialize(&self, output: &mut BufferWriter) {
        match self {
            EntryPointType::Constructor => output.write_felt(0.into()),
            EntryPointType::External => output.write_felt(1.into()),
            EntryPointType::L1Handler => output.write_felt(2.into()),
        }
    }
}

impl CairoSerialize for CallType {
    fn serialize(&self, output: &mut BufferWriter) {
        match self {
            CallType::Call => output.write_felt(0.into()),
            CallType::Delegate => output.write_felt(1.into()),
        }
    }
}

impl CairoSerialize for bool {
    fn serialize(&self, output: &mut BufferWriter) {
        if *self {
            Felt252::from(1).serialize(output);
        } else {
            Felt252::from(0).serialize(output);
        }
    }
}

impl<T> CairoSerialize for Arc<T>
where
    T: CairoSerialize,
{
    fn serialize(&self, output: &mut BufferWriter) {
        T::serialize(self, output);
    }
}

impl<T> CairoSerialize for Rc<T>
where
    T: CairoSerialize,
{
    fn serialize(&self, output: &mut BufferWriter) {
        T::serialize(self, output);
    }
}

impl<T> CairoSerialize for RefCell<T>
where
    T: CairoSerialize,
{
    fn serialize(&self, output: &mut BufferWriter) {
        self.borrow().serialize(output);
    }
}

impl<T> CairoSerialize for Ref<'_, T>
where
    T: CairoSerialize,
{
    fn serialize(&self, output: &mut BufferWriter) {
        T::serialize(self, output);
    }
}

// Try remove impls for String, ByteArray should be used explicitly instead
impl CairoSerialize for &str {
    fn serialize(&self, output: &mut BufferWriter) {
        ByteArray::from(*self).serialize(output);
    }
}

impl CairoSerialize for String {
    fn serialize(&self, output: &mut BufferWriter) {
        self.as_str().serialize(output);
    }
}

impl<T> CairoSerialize for Vec<T>
where
    T: CairoSerialize,
{
    fn serialize(&self, output: &mut BufferWriter) {
        self.len().serialize(output);

        for e in self {
            e.serialize(output);
        }
    }
}

impl<T: CairoSerialize, E: CairoSerialize> CairoSerialize for Result<T, E> {
    fn serialize(&self, output: &mut BufferWriter) {
        match self {
            Ok(val) => {
                output.write_felt(Felt252::from(0));
                val.serialize(output);
            }
            Err(err) => {
                output.write_felt(Felt252::from(1));
                err.serialize(output);
            }
        }
    }
}

impl<T: CairoSerialize> CairoSerialize for Option<T> {
    fn serialize(&self, output: &mut BufferWriter) {
        match self {
            Some(val) => {
                output.write_felt(Felt252::from(0));
                val.serialize(output);
            }
            None => output.write_felt(Felt252::from(1)),
        }
    }
}

impl<T> CairoSerialize for &T
where
    T: CairoSerialize + ?Sized,
{
    fn serialize(&self, output: &mut BufferWriter) {
        T::serialize(self, output);
    }
}

macro_rules! impl_serialize_for_felt_type {
    ($type:ty) => {
        impl CairoSerialize for $type {
            fn serialize(&self, output: &mut BufferWriter) {
                output.write_felt(self.clone().into_());
            }
        }
    };
}

macro_rules! impl_serialize_for_num_type {
    ($type:ty) => {
        impl CairoSerialize for $type {
            fn serialize(&self, output: &mut BufferWriter) {
                Felt252::from(*self).serialize(output);
            }
        }
    };
}

macro_rules! impl_serialize_for_tuple {
    ($($ty:ident),*) => {
        impl<$( $ty ),*> CairoSerialize for ( $( $ty, )* )
        where
        $( $ty: CairoSerialize, )*
        {
            #[allow(non_snake_case)]
            #[allow(unused_variables)]
            fn serialize(&self, output: &mut BufferWriter) {
                let ( $( $ty, )* ) = self;

                $( $ty.serialize(output); )*
            }
        }
    };
}

impl_serialize_for_felt_type!(Felt252);
impl_serialize_for_felt_type!(FieldElement);
impl_serialize_for_felt_type!(ClassHash);
impl_serialize_for_felt_type!(ContractAddress);
impl_serialize_for_felt_type!(Nonce);
impl_serialize_for_felt_type!(EntryPointSelector);
impl_serialize_for_felt_type!(EthAddress);

impl_serialize_for_num_type!(u8);
impl_serialize_for_num_type!(u16);
impl_serialize_for_num_type!(u32);
impl_serialize_for_num_type!(u64);
impl_serialize_for_num_type!(u128);
impl_serialize_for_num_type!(usize);

impl_serialize_for_tuple!();
impl_serialize_for_tuple!(A);
impl_serialize_for_tuple!(A, B);
impl_serialize_for_tuple!(A, B, C);
impl_serialize_for_tuple!(A, B, C, D); // cairo serde supports tuples in range 0 - 4 only
