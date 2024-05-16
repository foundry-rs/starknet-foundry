use crate::{
    felt252::{RawFeltVec, SerializeAsFelt252Vec},
    IntoConv,
};
use blockifier::execution::entry_point::{CallEntryPoint, CallType};
use cairo_felt::Felt252;
use starknet::core::types::FieldElement;
use starknet_api::{
    core::{ClassHash, ContractAddress, EntryPointSelector, Nonce},
    deprecated_contract_class::EntryPointType,
    hash::StarkFelt,
    transaction::Calldata,
};
use std::{cell::RefCell, rc::Rc, sync::Arc};

impl SerializeAsFelt252Vec for CallEntryPoint {
    fn serialize_into_felt252_vec(&self, output: &mut Vec<Felt252>) {
        self.entry_point_type.serialize_into_felt252_vec(output);
        self.entry_point_selector.serialize_into_felt252_vec(output);
        self.calldata.serialize_into_felt252_vec(output);
        self.storage_address.serialize_into_felt252_vec(output);
        self.caller_address.serialize_into_felt252_vec(output);
        self.call_type.serialize_into_felt252_vec(output);
    }
}

impl SerializeAsFelt252Vec for Calldata {
    fn serialize_into_felt252_vec(&self, output: &mut Vec<Felt252>) {
        self.0.serialize_into_felt252_vec(output)
    }
}

impl SerializeAsFelt252Vec for EntryPointType {
    fn serialize_into_felt252_vec(&self, output: &mut Vec<Felt252>) {
        match self {
            EntryPointType::Constructor => output.push(0.into()),
            EntryPointType::External => output.push(1.into()),
            EntryPointType::L1Handler => output.push(2.into()),
        }
    }
}

impl SerializeAsFelt252Vec for CallType {
    fn serialize_into_felt252_vec(&self, output: &mut Vec<Felt252>) {
        match self {
            CallType::Call => output.push(0.into()),
            CallType::Delegate => output.push(1.into()),
        }
    }
}

impl<T> SerializeAsFelt252Vec for Arc<T>
where
    T: SerializeAsFelt252Vec,
{
    fn serialize_into_felt252_vec(&self, output: &mut Vec<Felt252>) {
        T::serialize_into_felt252_vec(self, output);
    }
}

impl<T> SerializeAsFelt252Vec for Rc<T>
where
    T: SerializeAsFelt252Vec,
{
    fn serialize_into_felt252_vec(&self, output: &mut Vec<Felt252>) {
        T::serialize_into_felt252_vec(self, output);
    }
}

impl<T> SerializeAsFelt252Vec for RefCell<T>
where
    T: SerializeAsFelt252Vec,
{
    fn serialize_into_felt252_vec(&self, output: &mut Vec<Felt252>) {
        self.borrow().serialize_into_felt252_vec(output)
    }
}

impl<T> SerializeAsFelt252Vec for RawFeltVec<T>
where
    T: SerializeAsFelt252Vec,
{
    fn serialize_into_felt252_vec(&self, output: &mut Vec<Felt252>) {
        for e in &self.0 {
            e.serialize_into_felt252_vec(output);
        }
    }
}

impl<T> SerializeAsFelt252Vec for Vec<T>
where
    T: SerializeAsFelt252Vec,
{
    fn serialize_into_felt252_vec(&self, output: &mut Vec<Felt252>) {
        let len: Felt252 = self.len().into();

        len.serialize_into_felt252_vec(output);

        for e in self {
            e.serialize_into_felt252_vec(output);
        }
    }
}

impl<T: SerializeAsFelt252Vec, E: SerializeAsFelt252Vec> SerializeAsFelt252Vec for Result<T, E> {
    fn serialize_into_felt252_vec(&self, output: &mut Vec<Felt252>) {
        match self {
            Ok(val) => {
                output.push(Felt252::from(0));
                val.serialize_into_felt252_vec(output);
            }
            Err(err) => {
                output.push(Felt252::from(1));
                err.serialize_into_felt252_vec(output);
            }
        }
    }
}

impl SerializeAsFelt252Vec for () {
    fn serialize_into_felt252_vec(&self, _output: &mut Vec<Felt252>) {}
}

impl SerializeAsFelt252Vec for &str {
    fn serialize_into_felt252_vec(&self, output: &mut Vec<Felt252>) {
        output.extend(self.serialize_as_felt252_vec());
    }
}

impl SerializeAsFelt252Vec for String {
    fn serialize_into_felt252_vec(&self, output: &mut Vec<Felt252>) {
        self.as_str().serialize_into_felt252_vec(output);
    }
}

impl<T> SerializeAsFelt252Vec for &T
where
    T: SerializeAsFelt252Vec + ?Sized,
{
    fn serialize_into_felt252_vec(&self, output: &mut Vec<Felt252>) {
        T::serialize_into_felt252_vec(self, output);
    }
}

macro_rules! impl_serialize_as_felt252_vec_for_felt_type {
    ($type:ty) => {
        impl SerializeAsFelt252Vec for $type {
            fn serialize_into_felt252_vec(&self, output: &mut Vec<Felt252>) {
                output.push(self.clone().into_());
            }
        }
    };
}

macro_rules! impl_serialize_as_felt252_vec_for_num_type {
    ($type:ty) => {
        impl SerializeAsFelt252Vec for $type {
            fn serialize_into_felt252_vec(&self, output: &mut Vec<Felt252>) {
                let felt = Felt252::from(*self);

                felt.serialize_into_felt252_vec(output);
            }
        }
    };
}

impl_serialize_as_felt252_vec_for_felt_type!(Felt252);
impl_serialize_as_felt252_vec_for_felt_type!(FieldElement);
impl_serialize_as_felt252_vec_for_felt_type!(ClassHash);
impl_serialize_as_felt252_vec_for_felt_type!(StarkFelt);
impl_serialize_as_felt252_vec_for_felt_type!(ContractAddress);
impl_serialize_as_felt252_vec_for_felt_type!(Nonce);
impl_serialize_as_felt252_vec_for_felt_type!(EntryPointSelector);

impl_serialize_as_felt252_vec_for_num_type!(u8);
impl_serialize_as_felt252_vec_for_num_type!(u16);
impl_serialize_as_felt252_vec_for_num_type!(u32);
impl_serialize_as_felt252_vec_for_num_type!(u64);
impl_serialize_as_felt252_vec_for_num_type!(u128);
impl_serialize_as_felt252_vec_for_num_type!(usize);
