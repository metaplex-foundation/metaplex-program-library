use anchor_lang::prelude::*;
use mpl_token_metadata::state::Metadata;
use solana_program::keccak;

use crate::{error::BubblegumError, utils::get_asset_id};

use super::{
    leaf_schema::LeafSchema,
    metaplex_adapter::{
        Collection, Creator, MetadataArgs, TokenProgramVersion, TokenStandard, Uses,
    },
};

pub struct MetaplexMetadata {
    pub metadata: Metadata,
    pub owner: Pubkey,
    pub delegate: Pubkey,
    pub leaf_index: u64,
    pub merkle_tree: Pubkey,
}

impl MetaplexMetadata {
    pub fn to_leaf_schema_v0(&self) -> Result<LeafSchema> {
        let data_hash = self.hash_metadata()?;
        let creator_hash = self.hash_creators();
        let asset_id = get_asset_id(&self.merkle_tree, self.leaf_index);
        Ok(LeafSchema::new_v0(
            asset_id,
            self.owner.clone(),
            self.delegate.clone(),
            self.leaf_index,
            data_hash,
            creator_hash,
        ))
    }

    pub fn to_metadata_args(&self) -> Result<MetadataArgs> {
        let token_standard: Result<Option<TokenStandard>> = match self.metadata.token_standard {
            Some(mpl_token_metadata::state::TokenStandard::NonFungible) => {
                Ok(Some(TokenStandard::NonFungible))
            }
            Some(mpl_token_metadata::state::TokenStandard::FungibleAsset) => {
                Ok(Some(TokenStandard::FungibleAsset))
            }
            Some(mpl_token_metadata::state::TokenStandard::Fungible) => {
                Ok(Some(TokenStandard::Fungible))
            }
            Some(mpl_token_metadata::state::TokenStandard::NonFungibleEdition) => {
                Ok(Some(TokenStandard::NonFungibleEdition))
            }
            Some(mpl_token_metadata::state::TokenStandard::ProgrammableNonFungible) => {
                Err(BubblegumError::TokenStandardNotSupported.into())
            }
            None => Ok(None),
        };
        let token_standard = token_standard?;

        Ok(MetadataArgs {
            name: self.metadata.data.name.clone(),
            symbol: self.metadata.data.symbol.clone(),
            uri: self.metadata.data.uri.clone(),
            seller_fee_basis_points: self.metadata.data.seller_fee_basis_points,
            creators: self
                .metadata
                .data
                .creators
                .as_ref()
                .map_or(Vec::<_>::new(), |creators| {
                    creators
                        .into_iter()
                        .map(|c| Creator::from(c.clone()))
                        .collect()
                }),
            token_standard,
            is_mutable: self.metadata.is_mutable,
            primary_sale_happened: self.metadata.primary_sale_happened,
            uses: self.metadata.uses.as_ref().map(|uses| Uses::from(&uses)),
            collection: self
                .metadata
                .collection
                .as_ref()
                .map(|c| Collection::from(&c)),
            edition_nonce: self.metadata.edition_nonce,
            token_program_version: TokenProgramVersion::Original,
        })
    }

    pub fn from_metadata_args(
        args: &MetadataArgs,
        owner: &Pubkey,
        delegate: &Pubkey,
        leaf_index: u64,
        merkle_tree: &Pubkey,
        metadata_authority: &Pubkey,
    ) -> Result<Self> {
        let metadata: Metadata = args.clone().to_metadata(metadata_authority)?;
        Ok(Self {
            metadata,
            owner: owner.clone(),
            delegate: delegate.clone(),
            leaf_index,
            merkle_tree: merkle_tree.clone(),
        })
    }

    pub fn hash_creators(&self) -> [u8; 32] {
        // Convert creator Vec to bytes Vec.
        let creator_data = self
            .metadata
            .data
            .creators
            .as_ref()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|c| [c.address.as_ref(), &[c.verified as u8], &[c.share]].concat())
            .collect::<Vec<_>>();

        // Calculate new creator hash.
        keccak::hashv(
            creator_data
                .iter()
                .map(|c| c.as_slice())
                .collect::<Vec<&[u8]>>()
                .as_ref(),
        )
        .to_bytes()
    }

    pub fn hash_metadata(&self) -> Result<[u8; 32]> {
        let metadata: MetadataArgs = self.to_metadata_args()?;
        // @dev: seller_fee_basis points is encoded twice so that it can be passed to marketplace
        // instructions, without passing the entire, un-hashed MetadataArgs struct
        let metadata_args_hash = keccak::hashv(&[metadata.try_to_vec()?.as_slice()]);
        Ok(keccak::hashv(&[
            &metadata_args_hash.to_bytes(),
            &metadata.seller_fee_basis_points.to_le_bytes(),
        ])
        .to_bytes())
    }
}
