use crate::CheatnetState;
use rand::prelude::*;
use cairo_vm::Felt252;

impl CheatnetState {
    pub fn generate_random_felt252(&self) -> Felt252 {
        let mut rng = rand::thread_rng();

        let high_bits: u128 = rng.gen::<u128>();
        let low_bits: u128 = rng.gen::<u128>() >> 4;
        
        let combined_value = (high_bits as u128) | (low_bits << 128);
        
        Felt252::from(combined_value) 
    }
}
