use anyhow::anyhow;
use blockifier::execution::native::syscall_handler::NativeSyscallHandler;
use cairo_native::starknet::{
    ExecutionInfo, ExecutionInfoV2, Secp256k1Point, Secp256r1Point, StarknetSyscallHandler,
    SyscallResult, U256,
};
use cairo_vm::Felt252;
use conversions::{
    byte_array::ByteArray,
    serde::serialize::{SerializeToFeltVec, raw::RawFeltVec},
};

use crate::CheatcodeHandlingResult;

pub struct NativeExtendedRuntime<E: NativeExtensionLogic> {
    pub extension: E,
    pub runtime: E::Runtime,
}

pub enum NativeSyscallHandlingResult<T> {
    Forwarded,
    Handled(T),
}

pub trait NativeExtensionLogic {
    type Runtime: StarknetSyscallHandler;

    fn handle_cheatcode(
        &mut self,
        _selector: Felt252,
        _input: &[Felt252],
        _runtime: &mut Self::Runtime,
    ) -> anyhow::Result<CheatcodeHandlingResult> {
        Ok(CheatcodeHandlingResult::Forwarded)
    }

    fn handle_get_block_hash(
        &mut self,
        _block_number: u64,
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<Felt252>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_get_execution_info(
        &mut self,
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<ExecutionInfo>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_get_execution_info_v2(
        &mut self,
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<ExecutionInfoV2>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_deploy(
        &mut self,
        _class_hash: Felt252,
        _contract_address_salt: Felt252,
        _calldata: &[Felt252],
        _deploy_from_zero: bool,
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<(Felt252, Vec<Felt252>)>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_replace_class(
        &mut self,
        _class_hash: Felt252,
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<()>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_library_call(
        &mut self,
        _class_hash: Felt252,
        _function_selector: Felt252,
        _calldata: &[Felt252],
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<Vec<Felt252>>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_call_contract(
        &mut self,
        _address: Felt252,
        _entry_point_selector: Felt252,
        _calldata: &[Felt252],
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<Vec<Felt252>>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_storage_read(
        &mut self,
        _address_domain: u32,
        _address: Felt252,
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<Felt252>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_storage_write(
        &mut self,
        _address_domain: u32,
        _address: Felt252,
        _value: Felt252,
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<()>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_emit_event(
        &mut self,
        _keys: &[Felt252],
        _data: &[Felt252],
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<()>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_send_message_to_l1(
        &mut self,
        _to_address: Felt252,
        _payload: &[Felt252],
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<()>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_keccak(
        &mut self,
        _input: &[u64],
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<U256>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_secp256k1_new(
        &mut self,
        _x: U256,
        _y: U256,
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<Option<Secp256k1Point>>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_secp256k1_add(
        &mut self,
        _p0: Secp256k1Point,
        _p1: Secp256k1Point,
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<Secp256k1Point>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_secp256k1_mul(
        &mut self,
        _p: Secp256k1Point,
        _m: U256,
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<Secp256k1Point>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_secp256k1_get_point_from_x(
        &mut self,
        _x: U256,
        _y_parity: bool,
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<Option<Secp256k1Point>>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_secp256k1_get_xy(
        &mut self,
        _p: Secp256k1Point,
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<(U256, U256)>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_secp256r1_new(
        &mut self,
        _x: U256,
        _y: U256,
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<Option<Secp256r1Point>>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_secp256r1_add(
        &mut self,
        _p0: Secp256r1Point,
        _p1: Secp256r1Point,
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<Secp256r1Point>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_secp256r1_mul(
        &mut self,
        _p: Secp256r1Point,
        _m: U256,
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<Secp256r1Point>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_secp256r1_get_point_from_x(
        &mut self,
        _x: U256,
        _y_parity: bool,
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<Option<Secp256r1Point>>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_secp256r1_get_xy(
        &mut self,
        _p: Secp256r1Point,
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<(U256, U256)>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_sha256_process_block(
        &mut self,
        _state: &mut [u32; 8],
        _block: &[u32; 16],
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<()>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_get_class_hash_at(
        &mut self,
        _contract_address: Felt252,
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<Felt252>> {
        NativeSyscallHandlingResult::Forwarded
    }

    fn handle_meta_tx_v0(
        &mut self,
        _address: Felt252,
        _entry_point_selector: Felt252,
        _calldata: &[Felt252],
        _signature: &[Felt252],
        _remaining_gas: &mut u64,
        _runtime: &mut Self::Runtime,
    ) -> NativeSyscallHandlingResult<SyscallResult<Vec<Felt252>>> {
        NativeSyscallHandlingResult::Forwarded
    }
}

impl<E: NativeExtensionLogic> StarknetSyscallHandler for &mut NativeExtendedRuntime<E> {
    fn cheatcode(&mut self, selector: Felt252, input: &[Felt252]) -> Vec<Felt252> {
        match self
            .extension
            .handle_cheatcode(selector, input, &mut self.runtime)
        {
            Ok(CheatcodeHandlingResult::Forwarded) => {
                return self.runtime.cheatcode(selector, input);
            }
            Ok(CheatcodeHandlingResult::Handled(result)) => Ok(RawFeltVec::new(result)),
            Err(err) => Err(ByteArray::from(err.to_string().as_str())),
        }
        .serialize_to_vec()
    }

    fn get_block_hash(
        &mut self,
        block_number: u64,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Felt252> {
        match self
            .extension
            .handle_get_block_hash(block_number, remaining_gas, &mut self.runtime)
        {
            NativeSyscallHandlingResult::Forwarded => {
                self.runtime.get_block_hash(block_number, remaining_gas)
            }
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn get_execution_info(&mut self, remaining_gas: &mut u64) -> SyscallResult<ExecutionInfo> {
        match self
            .extension
            .handle_get_execution_info(remaining_gas, &mut self.runtime)
        {
            NativeSyscallHandlingResult::Forwarded => {
                self.runtime.get_execution_info(remaining_gas)
            }
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn get_execution_info_v2(&mut self, remaining_gas: &mut u64) -> SyscallResult<ExecutionInfoV2> {
        match self
            .extension
            .handle_get_execution_info_v2(remaining_gas, &mut self.runtime)
        {
            NativeSyscallHandlingResult::Forwarded => {
                self.runtime.get_execution_info_v2(remaining_gas)
            }
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn deploy(
        &mut self,
        class_hash: Felt252,
        contract_address_salt: Felt252,
        calldata: &[Felt252],
        deploy_from_zero: bool,
        remaining_gas: &mut u64,
    ) -> SyscallResult<(Felt252, Vec<Felt252>)> {
        match self.extension.handle_deploy(
            class_hash,
            contract_address_salt,
            calldata,
            deploy_from_zero,
            remaining_gas,
            &mut self.runtime,
        ) {
            NativeSyscallHandlingResult::Forwarded => self.runtime.deploy(
                class_hash,
                contract_address_salt,
                calldata,
                deploy_from_zero,
                remaining_gas,
            ),
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn replace_class(&mut self, class_hash: Felt252, remaining_gas: &mut u64) -> SyscallResult<()> {
        match self
            .extension
            .handle_replace_class(class_hash, remaining_gas, &mut self.runtime)
        {
            NativeSyscallHandlingResult::Forwarded => {
                self.runtime.replace_class(class_hash, remaining_gas)
            }
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn library_call(
        &mut self,
        class_hash: Felt252,
        function_selector: Felt252,
        calldata: &[Felt252],
        remaining_gas: &mut u64,
    ) -> SyscallResult<Vec<Felt252>> {
        match self.extension.handle_library_call(
            class_hash,
            function_selector,
            calldata,
            remaining_gas,
            &mut self.runtime,
        ) {
            NativeSyscallHandlingResult::Forwarded => {
                self.runtime
                    .library_call(class_hash, function_selector, calldata, remaining_gas)
            }
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn call_contract(
        &mut self,
        address: Felt252,
        entry_point_selector: Felt252,
        calldata: &[Felt252],
        remaining_gas: &mut u64,
    ) -> SyscallResult<Vec<Felt252>> {
        match self.extension.handle_call_contract(
            address,
            entry_point_selector,
            calldata,
            remaining_gas,
            &mut self.runtime,
        ) {
            NativeSyscallHandlingResult::Forwarded => {
                self.runtime
                    .call_contract(address, entry_point_selector, calldata, remaining_gas)
            }
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn storage_read(
        &mut self,
        address_domain: u32,
        address: Felt252,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Felt252> {
        match self.extension.handle_storage_read(
            address_domain,
            address,
            remaining_gas,
            &mut self.runtime,
        ) {
            NativeSyscallHandlingResult::Forwarded => {
                self.runtime
                    .storage_read(address_domain, address, remaining_gas)
            }
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn storage_write(
        &mut self,
        address_domain: u32,
        address: Felt252,
        value: Felt252,
        remaining_gas: &mut u64,
    ) -> SyscallResult<()> {
        match self.extension.handle_storage_write(
            address_domain,
            address,
            value,
            remaining_gas,
            &mut self.runtime,
        ) {
            NativeSyscallHandlingResult::Forwarded => {
                self.runtime
                    .storage_write(address_domain, address, value, remaining_gas)
            }
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn emit_event(
        &mut self,
        keys: &[Felt252],
        data: &[Felt252],
        remaining_gas: &mut u64,
    ) -> SyscallResult<()> {
        match self
            .extension
            .handle_emit_event(keys, data, remaining_gas, &mut self.runtime)
        {
            NativeSyscallHandlingResult::Forwarded => {
                self.runtime.emit_event(keys, data, remaining_gas)
            }
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn send_message_to_l1(
        &mut self,
        to_address: Felt252,
        payload: &[Felt252],
        remaining_gas: &mut u64,
    ) -> SyscallResult<()> {
        match self.extension.handle_send_message_to_l1(
            to_address,
            payload,
            remaining_gas,
            &mut self.runtime,
        ) {
            NativeSyscallHandlingResult::Forwarded => {
                self.runtime
                    .send_message_to_l1(to_address, payload, remaining_gas)
            }
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn keccak(&mut self, input: &[u64], remaining_gas: &mut u64) -> SyscallResult<U256> {
        match self
            .extension
            .handle_keccak(input, remaining_gas, &mut self.runtime)
        {
            NativeSyscallHandlingResult::Forwarded => self.runtime.keccak(input, remaining_gas),
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn secp256k1_new(
        &mut self,
        x: U256,
        y: U256,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Option<Secp256k1Point>> {
        match self
            .extension
            .handle_secp256k1_new(x, y, remaining_gas, &mut self.runtime)
        {
            NativeSyscallHandlingResult::Forwarded => {
                self.runtime.secp256k1_new(x, y, remaining_gas)
            }
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn secp256k1_add(
        &mut self,
        p0: Secp256k1Point,
        p1: Secp256k1Point,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Secp256k1Point> {
        match self
            .extension
            .handle_secp256k1_add(p0, p1, remaining_gas, &mut self.runtime)
        {
            NativeSyscallHandlingResult::Forwarded => {
                self.runtime.secp256k1_add(p0, p1, remaining_gas)
            }
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn secp256k1_mul(
        &mut self,
        p: Secp256k1Point,
        m: U256,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Secp256k1Point> {
        match self
            .extension
            .handle_secp256k1_mul(p, m, remaining_gas, &mut self.runtime)
        {
            NativeSyscallHandlingResult::Forwarded => {
                self.runtime.secp256k1_mul(p, m, remaining_gas)
            }
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn secp256k1_get_point_from_x(
        &mut self,
        x: U256,
        y_parity: bool,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Option<Secp256k1Point>> {
        match self.extension.handle_secp256k1_get_point_from_x(
            x,
            y_parity,
            remaining_gas,
            &mut self.runtime,
        ) {
            NativeSyscallHandlingResult::Forwarded => {
                self.runtime
                    .secp256k1_get_point_from_x(x, y_parity, remaining_gas)
            }
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn secp256k1_get_xy(
        &mut self,
        p: Secp256k1Point,
        remaining_gas: &mut u64,
    ) -> SyscallResult<(U256, U256)> {
        match self
            .extension
            .handle_secp256k1_get_xy(p, remaining_gas, &mut self.runtime)
        {
            NativeSyscallHandlingResult::Forwarded => {
                self.runtime.secp256k1_get_xy(p, remaining_gas)
            }
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn secp256r1_new(
        &mut self,
        x: U256,
        y: U256,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Option<Secp256r1Point>> {
        match self
            .extension
            .handle_secp256r1_new(x, y, remaining_gas, &mut self.runtime)
        {
            NativeSyscallHandlingResult::Forwarded => {
                self.runtime.secp256r1_new(x, y, remaining_gas)
            }
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn secp256r1_add(
        &mut self,
        p0: Secp256r1Point,
        p1: Secp256r1Point,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Secp256r1Point> {
        match self
            .extension
            .handle_secp256r1_add(p0, p1, remaining_gas, &mut self.runtime)
        {
            NativeSyscallHandlingResult::Forwarded => {
                self.runtime.secp256r1_add(p0, p1, remaining_gas)
            }
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn secp256r1_mul(
        &mut self,
        p: Secp256r1Point,
        m: U256,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Secp256r1Point> {
        match self
            .extension
            .handle_secp256r1_mul(p, m, remaining_gas, &mut self.runtime)
        {
            NativeSyscallHandlingResult::Forwarded => {
                self.runtime.secp256r1_mul(p, m, remaining_gas)
            }
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn secp256r1_get_point_from_x(
        &mut self,
        x: U256,
        y_parity: bool,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Option<Secp256r1Point>> {
        match self.extension.handle_secp256r1_get_point_from_x(
            x,
            y_parity,
            remaining_gas,
            &mut self.runtime,
        ) {
            NativeSyscallHandlingResult::Forwarded => {
                self.runtime
                    .secp256r1_get_point_from_x(x, y_parity, remaining_gas)
            }
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn secp256r1_get_xy(
        &mut self,
        p: Secp256r1Point,
        remaining_gas: &mut u64,
    ) -> SyscallResult<(U256, U256)> {
        match self
            .extension
            .handle_secp256r1_get_xy(p, remaining_gas, &mut self.runtime)
        {
            NativeSyscallHandlingResult::Forwarded => {
                self.runtime.secp256r1_get_xy(p, remaining_gas)
            }
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn sha256_process_block(
        &mut self,
        state: &mut [u32; 8],
        block: &[u32; 16],
        remaining_gas: &mut u64,
    ) -> SyscallResult<()> {
        match self.extension.handle_sha256_process_block(
            state,
            block,
            remaining_gas,
            &mut self.runtime,
        ) {
            NativeSyscallHandlingResult::Forwarded => {
                self.runtime
                    .sha256_process_block(state, block, remaining_gas)
            }
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn get_class_hash_at(
        &mut self,
        contract_address: Felt252,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Felt252> {
        match self.extension.handle_get_class_hash_at(
            contract_address,
            remaining_gas,
            &mut self.runtime,
        ) {
            NativeSyscallHandlingResult::Forwarded => self
                .runtime
                .get_class_hash_at(contract_address, remaining_gas),
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }

    fn meta_tx_v0(
        &mut self,
        address: Felt252,
        entry_point_selector: Felt252,
        calldata: &[Felt252],
        signature: &[Felt252],
        remaining_gas: &mut u64,
    ) -> SyscallResult<Vec<Felt252>> {
        match self.extension.handle_meta_tx_v0(
            address,
            entry_point_selector,
            calldata,
            signature,
            remaining_gas,
            &mut self.runtime,
        ) {
            NativeSyscallHandlingResult::Forwarded => self.runtime.meta_tx_v0(
                address,
                entry_point_selector,
                calldata,
                signature,
                remaining_gas,
            ),
            NativeSyscallHandlingResult::Handled(result) => result,
        }
    }
}

pub struct NativeStarknetRuntime<'a> {
    pub syscall_handler: NativeSyscallHandler<'a>,
}

impl<'a> StarknetSyscallHandler for NativeStarknetRuntime<'a> {
    fn get_block_hash(
        &mut self,
        block_number: u64,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Felt252> {
        (&mut self.syscall_handler).get_block_hash(block_number, remaining_gas)
    }

    fn get_execution_info(&mut self, remaining_gas: &mut u64) -> SyscallResult<ExecutionInfo> {
        (&mut self.syscall_handler).get_execution_info(remaining_gas)
    }

    fn get_execution_info_v2(&mut self, remaining_gas: &mut u64) -> SyscallResult<ExecutionInfoV2> {
        (&mut self.syscall_handler).get_execution_info_v2(remaining_gas)
    }

    fn deploy(
        &mut self,
        class_hash: Felt252,
        contract_address_salt: Felt252,
        calldata: &[Felt252],
        deploy_from_zero: bool,
        remaining_gas: &mut u64,
    ) -> SyscallResult<(Felt252, Vec<Felt252>)> {
        (&mut self.syscall_handler).deploy(
            class_hash,
            contract_address_salt,
            calldata,
            deploy_from_zero,
            remaining_gas,
        )
    }

    fn replace_class(&mut self, class_hash: Felt252, remaining_gas: &mut u64) -> SyscallResult<()> {
        (&mut self.syscall_handler).replace_class(class_hash, remaining_gas)
    }

    fn library_call(
        &mut self,
        class_hash: Felt252,
        function_selector: Felt252,
        calldata: &[Felt252],
        remaining_gas: &mut u64,
    ) -> SyscallResult<Vec<Felt252>> {
        (&mut self.syscall_handler).library_call(
            class_hash,
            function_selector,
            calldata,
            remaining_gas,
        )
    }

    fn call_contract(
        &mut self,
        address: Felt252,
        entry_point_selector: Felt252,
        calldata: &[Felt252],
        remaining_gas: &mut u64,
    ) -> SyscallResult<Vec<Felt252>> {
        (&mut self.syscall_handler).call_contract(
            address,
            entry_point_selector,
            calldata,
            remaining_gas,
        )
    }

    fn storage_read(
        &mut self,
        address_domain: u32,
        address: Felt252,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Felt252> {
        (&mut self.syscall_handler).storage_read(address_domain, address, remaining_gas)
    }

    fn storage_write(
        &mut self,
        address_domain: u32,
        address: Felt252,
        value: Felt252,
        remaining_gas: &mut u64,
    ) -> SyscallResult<()> {
        (&mut self.syscall_handler).storage_write(address_domain, address, value, remaining_gas)
    }

    fn emit_event(
        &mut self,
        keys: &[Felt252],
        data: &[Felt252],
        remaining_gas: &mut u64,
    ) -> SyscallResult<()> {
        (&mut self.syscall_handler).emit_event(keys, data, remaining_gas)
    }

    fn send_message_to_l1(
        &mut self,
        to_address: Felt252,
        payload: &[Felt252],
        remaining_gas: &mut u64,
    ) -> SyscallResult<()> {
        (&mut self.syscall_handler).send_message_to_l1(to_address, payload, remaining_gas)
    }

    fn keccak(&mut self, input: &[u64], remaining_gas: &mut u64) -> SyscallResult<U256> {
        (&mut self.syscall_handler).keccak(input, remaining_gas)
    }

    fn secp256k1_new(
        &mut self,
        x: U256,
        y: U256,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Option<Secp256k1Point>> {
        (&mut self.syscall_handler).secp256k1_new(x, y, remaining_gas)
    }

    fn secp256k1_add(
        &mut self,
        p0: Secp256k1Point,
        p1: Secp256k1Point,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Secp256k1Point> {
        (&mut self.syscall_handler).secp256k1_add(p0, p1, remaining_gas)
    }

    fn secp256k1_mul(
        &mut self,
        p: Secp256k1Point,
        m: U256,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Secp256k1Point> {
        (&mut self.syscall_handler).secp256k1_mul(p, m, remaining_gas)
    }

    fn secp256k1_get_point_from_x(
        &mut self,
        x: U256,
        y_parity: bool,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Option<Secp256k1Point>> {
        (&mut self.syscall_handler).secp256k1_get_point_from_x(x, y_parity, remaining_gas)
    }

    fn secp256k1_get_xy(
        &mut self,
        p: Secp256k1Point,
        remaining_gas: &mut u64,
    ) -> SyscallResult<(U256, U256)> {
        (&mut self.syscall_handler).secp256k1_get_xy(p, remaining_gas)
    }

    fn secp256r1_new(
        &mut self,
        x: U256,
        y: U256,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Option<Secp256r1Point>> {
        (&mut self.syscall_handler).secp256r1_new(x, y, remaining_gas)
    }

    fn secp256r1_add(
        &mut self,
        p0: Secp256r1Point,
        p1: Secp256r1Point,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Secp256r1Point> {
        (&mut self.syscall_handler).secp256r1_add(p0, p1, remaining_gas)
    }

    fn secp256r1_mul(
        &mut self,
        p: Secp256r1Point,
        m: U256,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Secp256r1Point> {
        (&mut self.syscall_handler).secp256r1_mul(p, m, remaining_gas)
    }

    fn secp256r1_get_point_from_x(
        &mut self,
        x: U256,
        y_parity: bool,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Option<Secp256r1Point>> {
        (&mut self.syscall_handler).secp256r1_get_point_from_x(x, y_parity, remaining_gas)
    }

    fn secp256r1_get_xy(
        &mut self,
        p: Secp256r1Point,
        remaining_gas: &mut u64,
    ) -> SyscallResult<(U256, U256)> {
        (&mut self.syscall_handler).secp256r1_get_xy(p, remaining_gas)
    }

    fn sha256_process_block(
        &mut self,
        state: &mut [u32; 8],
        block: &[u32; 16],
        remaining_gas: &mut u64,
    ) -> SyscallResult<()> {
        (&mut self.syscall_handler).sha256_process_block(state, block, remaining_gas)
    }

    fn get_class_hash_at(
        &mut self,
        contract_address: Felt252,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Felt252> {
        (&mut self.syscall_handler).get_class_hash_at(contract_address, remaining_gas)
    }

    fn meta_tx_v0(
        &mut self,
        address: Felt252,
        entry_point_selector: Felt252,
        calldata: &[Felt252],
        signature: &[Felt252],
        remaining_gas: &mut u64,
    ) -> SyscallResult<Vec<Felt252>> {
        (&mut self.syscall_handler).meta_tx_v0(
            address,
            entry_point_selector,
            calldata,
            signature,
            remaining_gas,
        )
    }

    fn cheatcode(&mut self, selector: Felt252, input: &[Felt252]) -> Vec<Felt252> {
        fn cheatcode(selector: Felt252, _input: &[Felt252]) -> anyhow::Result<Vec<Felt252>> {
            let selector_bytes = selector.to_bytes_be();
            let selector = std::str::from_utf8(&selector_bytes)?.trim_start_matches('\0');
            Err(anyhow!("invalid selector: {}", selector))
        }

        cheatcode(selector, input).serialize_to_vec()
    }
}
