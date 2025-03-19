use anyhow::{Error, bail};
use async_trait::async_trait;
use starknet::{
    accounts::{AccountFactory, PreparedAccountDeploymentV3, RawAccountDeploymentV3},
    core::types::{BlockId, BlockTag},
    providers::Provider,
    signers::{Signer, SignerInteractivityContext},
};
use starknet_crypto::poseidon_hash_many;
use starknet_types_core::felt::Felt;

use crate::AccountType;

// Adapted from strakli as there is currently no implementation of braavos account factory in starknet-rs
pub struct BraavosAccountFactory<S, P> {
    class_hash: Felt,
    base_class_hash: Felt,
    chain_id: Felt,
    signer_public_key: Felt,
    signer: S,
    provider: P,
    block_id: BlockId,
}

impl<S, P> BraavosAccountFactory<S, P>
where
    S: Signer,
{
    pub async fn new(
        class_hash: Felt,
        base_class_hash: Felt,
        chain_id: Felt,
        signer: S,
        provider: P,
    ) -> Result<Self, S::GetPublicKeyError> {
        let signer_public_key = signer.get_public_key().await?;
        Ok(Self {
            class_hash,
            base_class_hash,
            chain_id,
            signer_public_key: signer_public_key.scalar(),
            signer,
            provider,
            block_id: BlockId::Tag(BlockTag::Latest),
        })
    }

    pub fn set_block_id(&mut self, block_id: BlockId) -> &Self {
        self.block_id = block_id;
        self
    }

    async fn sign_deployment(&self, tx_hash: Felt) -> Result<Vec<Felt>, S::SignError> {
        let signature = self.signer.sign_hash(&tx_hash).await?;

        // You can see params here:
        // https://github.com/myBraavos/braavos-account-cairo/blob/6efdfd597bb051e99c79a512fccd14ee2523c898/src/presets/braavos_account.cairo#L104
        // Order of the params is important, you can see way and order of deserialization here:
        // https://github.com/myBraavos/braavos-account-cairo/blob/6efdfd597bb051e99c79a512fccd14ee2523c898/src/presets/braavos_account.cairo#L141
        // first 3 elements in sig are always [tx hash(r, s), account impl, ...]
        // last 2 elements are sig on the aux data sent in the sig preceded by chain id:
        // [..., account_impl, ..., chain_id, aux(r, s)]
        // ref: https://github.com/myBraavos/braavos-account-cairo/blob/6efdfd597bb051e99c79a512fccd14ee2523c898/src/presets/braavos_base_account.cairo#L74
        let aux_data: Vec<Felt> = vec![
            // account_implementation
            self.class_hash,
            // signer_type
            Felt::ZERO,
            // secp256r1_signer.x.low
            Felt::ZERO,
            // secp256r1_signer.x.high
            Felt::ZERO,
            // secp256r1_signer.y.low
            Felt::ZERO,
            // secp256r1_signer.y.high
            Felt::ZERO,
            // multisig_threshold
            Felt::ZERO,
            // withdrawal_limit_low
            Felt::ZERO,
            // fee_rate
            Felt::ZERO,
            // stark_fee_rate
            Felt::ZERO,
            // chain_id
            self.chain_id,
        ];

        let aux_hash = poseidon_hash_many(&aux_data[..]);
        let aux_signature = self.signer.sign_hash(&aux_hash).await?;

        Ok([
            vec![signature.r, signature.s],
            aux_data,
            vec![aux_signature.r, aux_signature.s],
        ]
        .concat())
    }
}

#[async_trait]
impl<S, P> AccountFactory for BraavosAccountFactory<S, P>
where
    S: Signer + Sync + Send,
    P: Provider + Sync + Send,
{
    type Provider = P;
    type SignError = S::SignError;

    #[expect(clippy::misnamed_getters)]
    fn class_hash(&self) -> Felt {
        self.base_class_hash
    }

    fn calldata(&self) -> Vec<Felt> {
        vec![self.signer_public_key]
    }

    fn chain_id(&self) -> Felt {
        self.chain_id
    }

    fn provider(&self) -> &Self::Provider {
        &self.provider
    }

    fn block_id(&self) -> BlockId {
        self.block_id
    }

    async fn sign_deployment_v3(
        &self,
        deployment: &RawAccountDeploymentV3,
        query_only: bool,
    ) -> Result<Vec<Felt>, Self::SignError> {
        let tx_hash = PreparedAccountDeploymentV3::from_raw(deployment.clone(), self)
            .transaction_hash(query_only);
        self.sign_deployment(tx_hash).await
    }

    fn is_signer_interactive(&self) -> bool {
        self.signer
            .is_interactive(SignerInteractivityContext::Other)
    }
}

pub fn assert_non_braavos_account_type(account_type: AccountType) -> Result<(), Error> {
    if let AccountType::Braavos = account_type {
        bail!("Integration with Braavos accounts is currently disabled")
    }
    Ok(())
}
