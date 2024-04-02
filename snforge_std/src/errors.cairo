use core::option::OptionTrait;
use core::starknet::SyscallResult;
use core::byte_array::BYTE_ARRAY_MAGIC;

pub enum PanicDataOrString {
    String: ByteArray,
    PanicData: Array<felt252>,
}

type ExtendedSyscallResult<T> = Result<T, PanicDataOrString>;

trait SyscallResultStringErrorTrait<T> {
    fn map_error_to_string(self: SyscallResult<T>) -> ExtendedSyscallResult<T>;
}

impl SyscallResultStringErrorTraitImpl<T> of SyscallResultStringErrorTrait<T> {
    fn map_error_to_string(self: SyscallResult<T>) -> ExtendedSyscallResult<T> {
        match self {
            Result::Ok(x) => Result::Ok(x),
            Result::Err(panic_data) => {
                if panic_data.len() > 0 && *panic_data.at(0) == BYTE_ARRAY_MAGIC {
                    let mut panic_data_span = panic_data.span().slice(1, panic_data.len() - 1);
                    let deserialized = Serde::<ByteArray>::deserialize(ref panic_data_span)
                        .expect('panic string not deserializable');
                    return Result::Err(PanicDataOrString::String(deserialized));
                }
                Result::Err(PanicDataOrString::PanicData(panic_data))
            }
        }
    }
}
