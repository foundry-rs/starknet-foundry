use fork_testing::{IPokemonGalleryDispatcher, IPokemonGalleryDispatcherTrait};

const CONTRACT_ADDRESS: felt252 =
    0x0522dc7cbe288037382a02569af5a4169531053d284193623948eac8dd051716;

#[test]
#[fork("SEPOLIA_LATEST", block_number: 200000)]
fn test_using_forked_state() {
    let dispatcher = IPokemonGalleryDispatcher {
        contract_address: CONTRACT_ADDRESS.try_into().unwrap(),
    };

    dispatcher.like("Charizard");
    let pokemon = dispatcher.pokemon("Charizard");

    assert!(pokemon.is_some());
    assert_eq!(pokemon.unwrap().likes, 1);
}
