use core::fmt::Debug;
pub use super::cheatcodes::generate_arg::generate_arg;

const MAX_FELT: felt252 = 0x800000000000011000000000000000000000000000000000000000000000000;

pub trait Fuzzable<T, +Debug<T>> {
    fn blank() -> T;
    fn generate() -> T;
}

impl FuzzableFelt of Fuzzable<felt252> {
    fn blank() -> felt252 {
        0x0
    }

    fn generate() -> felt252 {
        generate_arg(0x0, MAX_FELT)
    }
}

mod nums {
    use core::num::traits::{Bounded, Zero};
    use super::{Debug, Fuzzable, generate_arg};

    pub impl FuzzableNum<
        T, +Zero<T>, +Bounded<T>, +Drop<T>, +Serde<T>, +Into<T, felt252>, +Debug<T>,
    > of Fuzzable<T> {
        fn blank() -> T {
            Zero::<T>::zero()
        }

        fn generate() -> T {
            generate_arg(Bounded::<T>::MIN, Bounded::<T>::MAX)
        }
    }
}

pub impl FuzzableU8 = nums::FuzzableNum<u8>;
pub impl FuzzableU16 = nums::FuzzableNum<u16>;
pub impl FuzzableU32 = nums::FuzzableNum<u32>;
pub impl FuzzableU64 = nums::FuzzableNum<u64>;
pub impl FuzzableU128 = nums::FuzzableNum<u128>;

pub impl FuzzableI8 = nums::FuzzableNum<i8>;
pub impl FuzzableI16 = nums::FuzzableNum<i16>;
pub impl FuzzableI32 = nums::FuzzableNum<i32>;
pub impl FuzzableI64 = nums::FuzzableNum<i64>;
pub impl FuzzableI128 = nums::FuzzableNum<i128>;


pub impl FuzzableU256 of Fuzzable<u256> {
    fn blank() -> u256 {
        0
    }

    fn generate() -> u256 {
        let mut serialized: Span<felt252> = array![
            Fuzzable::<u128>::generate().into(), Fuzzable::<u128>::generate().into(),
        ]
            .span();
        Serde::deserialize(ref serialized).unwrap()
    }
}


pub impl FuzzableByteArray1000ASCII of Fuzzable<ByteArray> {
    fn blank() -> ByteArray {
        ""
    }

    // Generates a random string of length 0 to 1000
    fn generate() -> ByteArray {
        let mut ba_len: u32 = generate_arg(0, 1000);

        let mut ba = "";
        while ba_len > 0 {
            // Limit only to printable characters with ASCII codes 32-126
            let letter = Fuzzable::<u8>::generate() % 95;
            ba.append_byte(letter + 32);
            ba_len = ba_len - 1;
        };

        ba
    }
}
