/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate lazy_static;

use bellman::groth16;
use bls12_381::Bls12;

mod serializing;

pub mod errors;
pub mod keys;
pub mod merkle_note;
pub mod merkle_note_hash;
pub mod mining;
pub mod nacl;
pub mod note;
pub mod primitives;
pub mod proofs;
pub mod receiving;
pub mod rolling_filter;
pub mod sapling_bls12;
pub mod spending;
pub mod transaction;
pub mod witness;
pub use {
    keys::{IncomingViewKey, OutgoingViewKey, PublicAddress, SaplingKey, ViewKeys},
    merkle_note::MerkleNote,
    merkle_note_hash::MerkleNoteHash,
    note::Note,
    primitives::asset_type::AssetType,
    receiving::{ReceiptParams, ReceiptProof},
    spending::{SpendParams, SpendProof},
    transaction::{ProposedTransaction, Transaction},
};

#[cfg(test)]
pub(crate) mod test_util; // I'm not sure if this is the right way to publish the utility library.

// The main entry-point to the sapling API. Construct this with loaded parameters, and then call
// methods on it to do the actual work.
//
// spend and output are two arithmetic circuits for use in zksnark calculations provided by Bellman.
// Though the *_params have a verifying key on them, they are not the prepared verifying keys,
// so we store the prepared keys separately at the time of loading the params.
//
// The values are all loaded from a file in serialized form.
pub struct Sapling {
    spend_params: groth16::Parameters<Bls12>,
    receipt_params: groth16::Parameters<Bls12>,
    create_asset_params: groth16::Parameters<Bls12>,
    mint_asset_params: groth16::Parameters<Bls12>,
    spend_verifying_key: groth16::PreparedVerifyingKey<Bls12>,
    receipt_verifying_key: groth16::PreparedVerifyingKey<Bls12>,
    create_asset_verifying_key: groth16::PreparedVerifyingKey<Bls12>,
    mint_asset_verifying_key: groth16::PreparedVerifyingKey<Bls12>,
}

impl Sapling {
    /// Initialize a Sapling instance and prepare for proving. Load the parameters from a config file
    /// at a known location (`./sapling_params`, for now).
    pub fn load() -> Self {
        // TODO: We'll need to build our own parameters using a trusted set up at some point.
        // These params were borrowed from zcash
        let spend_bytes = include_bytes!("sapling_params/sapling-spend.params");
        let receipt_bytes = include_bytes!("sapling_params/sapling-output.params");
        let create_asset_bytes = include_bytes!("sapling_params/sapling-create-asset.params");
        let mint_asset_bytes = include_bytes!("sapling_params/sapling-mint-asset.params");

        let spend_params = Sapling::load_params(&spend_bytes[..]);
        let receipt_params = Sapling::load_params(&receipt_bytes[..]);
        let create_asset_params = Sapling::load_params(&create_asset_bytes[..]);
        let mint_asset_params = Sapling::load_params(&mint_asset_bytes[..]);

        let spend_vk = groth16::prepare_verifying_key(&spend_params.vk);
        let receipt_vk = groth16::prepare_verifying_key(&receipt_params.vk);
        let create_asset_vk = groth16::prepare_verifying_key(&create_asset_params.vk);
        let mint_asset_vk = groth16::prepare_verifying_key(&mint_asset_params.vk);

        Sapling {
            spend_verifying_key: spend_vk,
            receipt_verifying_key: receipt_vk,
            create_asset_verifying_key: create_asset_vk,
            mint_asset_verifying_key: mint_asset_vk,
            spend_params,
            receipt_params,
            create_asset_params,
            mint_asset_params,
        }
    }

    /// Load sapling parameters from a provided filename. The parameters are huge and take a
    /// couple seconds to load. They primarily contain the "toxic waste" for a specific sapling
    /// curve.
    ///
    /// NOTE: If this is stupidly slow for you, try compiling in --release mode
    fn load_params(bytes: &[u8]) -> groth16::Parameters<Bls12> {
        groth16::Parameters::read(bytes, false).unwrap()
    }
}
