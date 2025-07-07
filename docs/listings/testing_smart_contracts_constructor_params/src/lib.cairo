#[derive(Copy, Debug, Drop, Serde, starknet::Store)]
pub struct Product {
    pub name: felt252,
    pub price: u64,
    pub quantity: u64,
}

#[starknet::interface]
pub trait IShoppingCart<TContractState> {
    fn get_products(self: @TContractState) -> Array<Product>;
}

#[starknet::contract]
mod ShoppingCart {
    use starknet::storage::{MutableVecTrait, StoragePointerReadAccess, Vec, VecTrait};
    use super::Product;

    #[storage]
    struct Storage {
        products: Vec<Product>,
    }

    #[constructor]
    fn constructor(ref self: ContractState, initial_products: Array<Product>) {
        for product in initial_products {
            self.products.push(product);
        }
    }


    #[abi(embed_v0)]
    impl ShoppingCartImpl of super::IShoppingCart<ContractState> {
        fn get_products(self: @ContractState) -> Array<Product> {
            let mut products = array![];
            for i in 0..self.products.len() {
                products.append(self.products.at(i).read());
            }
            products
        }
    }
}
