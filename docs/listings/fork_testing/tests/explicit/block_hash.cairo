use fork_testing::{IPokemonGalleryDispatcher, IPokemonGalleryDispatcherTrait};

const CONTRACT_ADDRESS: felt252 =
    0x0522dc7cbe288037382a02569af5a4169531053d284193623948eac8dd051716;

#[test]
#[fork(
    url: "https://starknet-sepolia.public.blastapi.io/rpc/v0_7",
    block_hash: 0x0690f8d584b52c2798d76b3346217a516778abee9b1bd8e400beb4f05dd9a4e7,
)]
fn test_using_forked_state() {
    let dispatcher = IPokemonGalleryDispatcher {
        contract_address: CONTRACT_ADDRESS.try_into().unwrap(),
    };

    dispatcher.like("Charizard");
    let pokemon = dispatcher.pokemon("Charizard");

    assert!(pokemon.is_some());
    assert_eq!(pokemon.unwrap().likes, 1);
}
