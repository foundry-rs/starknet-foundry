use async_trait::async_trait;
use starknet::{
    accounts::{
        AccountFactory, PreparedAccountDeploymentV1, PreparedAccountDeploymentV3,
        RawAccountDeploymentV1, RawAccountDeploymentV3,
    },
    core::types::{BlockId, BlockTag, FieldElement},
    providers::Provider,
    signers::Signer,
};
use starknet_crypto::poseidon_hash_many;

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
impl<S, P> AccountFactory for BraavosAccountFactory<S, P>
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

    async fn sign_deployment_v1(
        &self,
        deployment: &RawAccountDeploymentV1,
    ) -> Result<Vec<FieldElement>, Self::SignError> {
        let tx_hash =
            PreparedAccountDeploymentV1::from_raw(deployment.clone(), self).transaction_hash();

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

    async fn sign_deployment_v3(
        &self,
        deployment: &RawAccountDeploymentV3,
    ) -> Result<Vec<FieldElement>, Self::SignError> {
        let tx_hash =
            PreparedAccountDeploymentV3::from_raw(deployment.clone(), self).transaction_hash();

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
