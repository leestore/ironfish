// TODO: Decide on a name?
// CreateAssetNote?
// AssetNote?
// What's less confusing when talking about it and trying to differentiate
// between a regular "Note"

use std::slice;

use bls12_381::Scalar;
use group::Curve;
use rand::{thread_rng, Rng};
use zcash_primitives::{
    constants::{GH_FIRST_BLOCK, NOTE_COMMITMENT_RANDOMNESS_GENERATOR},
    sapling::pedersen_hash,
};

use crate::primitives::asset_type::AssetInfo;

/// A create asset note represents an asset in the owner's "account"
/// Expected API:
/// let can = CreateAssetNote::new(asset_info);
/// proposed_transaction.create_asset(spender_key, &can);
/// proposed_transaction.post, verify, etc.
pub struct CreateAssetNote {
    pub(crate) asset_info: AssetInfo,
    pub(crate) randomness: jubjub::Fr,
}

impl CreateAssetNote {
    // TODO: carry over all? fns from Note
    pub fn new(asset_info: AssetInfo) -> Self {
        let mut buffer = [0u8; 64];
        thread_rng().fill(&mut buffer[..]);

        let randomness: jubjub::Fr = jubjub::Fr::from_bytes_wide(&buffer);

        Self {
            asset_info,
            randomness,
        }
    }

    pub fn commitment_point(&self) -> Scalar {
        jubjub::ExtendedPoint::from(self.commitment_full_point())
            .to_affine()
            .get_u()
    }

    // TODO: Look into how many times this is called in the object's lifecycle
    // and see if caching the preimage, hash, etc makes sense.
    fn commitment_full_point(&self) -> jubjub::SubgroupPoint {
        let mut create_commitment_plaintext: Vec<u8> = vec![];
        create_commitment_plaintext.extend(GH_FIRST_BLOCK);
        create_commitment_plaintext.extend(self.asset_info.name());
        create_commitment_plaintext.extend(self.asset_info.public_address_bytes());
        create_commitment_plaintext.extend(slice::from_ref(self.asset_info.nonce()));

        let create_commitment_hash = pedersen_hash::pedersen_hash(
            pedersen_hash::Personalization::NoteCommitment,
            create_commitment_plaintext
                .into_iter()
                .flat_map(|byte| (0..8).map(move |i| ((byte >> i) & 1) == 1)),
        );

        create_commitment_hash + (NOTE_COMMITMENT_RANDOMNESS_GENERATOR * self.randomness)
    }
}
