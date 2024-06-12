use crate::helpers::accounts_format::AccountType;
use crate::helpers::constants::BRAAVOS_BASE_ACCOUNT_CLASS_HASH;
use async_trait::async_trait;
use starknet::accounts::{ArgentAccountFactory, OpenZeppelinAccountFactory};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::LocalWallet;
use starknet::{
    accounts::{PreparedAccountDeployment, RawAccountDeployment},
    core::types::{BlockId, BlockTag, FieldElement},
    providers::Provider,
    signers::Signer,
};
use starknet_crypto::poseidon_hash_many;

pub enum AccountFactory<'a> {
    Oz(OpenZeppelinAccountFactory<LocalWallet, &'a JsonRpcClient<HttpTransport>>),
    Argent(ArgentAccountFactory<LocalWallet, &'a JsonRpcClient<HttpTransport>>),
    Braavos(BraavosAccountFactory<LocalWallet, &'a JsonRpcClient<HttpTransport>>),
}

pub async fn create_account_factory(
    account_type: AccountType,
    class_hash: FieldElement,
    chain_id: FieldElement,
    signer: LocalWallet,
    provider: &JsonRpcClient<HttpTransport>,
) -> anyhow::Result<AccountFactory> {
    let factory = match account_type {
        AccountType::Oz => AccountFactory::Oz(
            OpenZeppelinAccountFactory::new(class_hash, chain_id, signer, provider).await?,
        ),
        AccountType::Argent => AccountFactory::Argent(
            ArgentAccountFactory::new(class_hash, chain_id, FieldElement::ZERO, signer, provider)
                .await?,
        ),
        AccountType::Braavos => AccountFactory::Braavos(
            BraavosAccountFactory::new(
                class_hash,
                BRAAVOS_BASE_ACCOUNT_CLASS_HASH,
                chain_id,
                signer,
                provider,
            )
            .await?,
        ),
    };

    Ok(factory)
}

// Adapted from strakli as there is currently no implementation of braavos account factory in starknet-rs
pub struct BraavosAccountFactory<S, P> {
    class_hash: FieldElement,
    base_class_hash: FieldElement,
    chain_id: FieldElement,
    signer_public_key: FieldElement,
    signer: S,
    provider: P,
    block_id: BlockId,
}

impl<S, P> BraavosAccountFactory<S, P>
where
    S: Signer,
{
    pub async fn new(
        class_hash: FieldElement,
        base_class_hash: FieldElement,
        chain_id: FieldElement,
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
}

#[async_trait]
impl<S, P> starknet::accounts::AccountFactory for BraavosAccountFactory<S, P>
where
    S: Signer + Sync + Send,
    P: Provider + Sync + Send,
{
    type Provider = P;
    type SignError = S::SignError;

    #[allow(clippy::misnamed_getters)]
    fn class_hash(&self) -> FieldElement {
        self.base_class_hash
    }

    fn calldata(&self) -> Vec<FieldElement> {
        vec![self.signer_public_key]
    }

    fn chain_id(&self) -> FieldElement {
        self.chain_id
    }

    fn provider(&self) -> &Self::Provider {
        &self.provider
    }

    fn block_id(&self) -> BlockId {
        self.block_id
    }

    async fn sign_deployment(
        &self,
        deployment: &RawAccountDeployment,
    ) -> Result<Vec<FieldElement>, Self::SignError> {
        let tx_hash =
            PreparedAccountDeployment::from_raw(deployment.clone(), self).transaction_hash();

        let signature = self.signer.sign_hash(&tx_hash).await?;

        let mut aux_data = vec![
            // account_implementation
            self.class_hash,
            // signer_type
            FieldElement::ZERO,
            // secp256r1_signer.x.low
            FieldElement::ZERO,
            // secp256r1_signer.x.high
            FieldElement::ZERO,
            // secp256r1_signer.y.low
            FieldElement::ZERO,
            // secp256r1_signer.y.high
            FieldElement::ZERO,
            // multisig_threshold
            FieldElement::ZERO,
            // withdrawal_limit_low
            FieldElement::ZERO,
            // fee_rate
            FieldElement::ZERO,
            // stark_fee_rate
            FieldElement::ZERO,
            // chain_id
            self.chain_id,
        ];

        let aux_hash = poseidon_hash_many(&aux_data);

        let aux_signature = self.signer.sign_hash(&aux_hash).await?;

        let mut full_signature_payload = vec![signature.r, signature.s];
        full_signature_payload.append(&mut aux_data);
        full_signature_payload.push(aux_signature.r);
        full_signature_payload.push(aux_signature.s);

        Ok(full_signature_payload)
    }
}
