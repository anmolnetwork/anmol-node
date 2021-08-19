#![cfg_attr(not(feature = "std"), no_std)]

use base_nft::Module as BaseNft;
use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};

use frame_system::{offchain::CreateSignedTransaction, pallet_prelude::*};

pub use pallet::*;
use sp_std::vec::Vec;

mod mock;
mod tests;

mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;

type ByteVector = Vec<u8>;

pub const MAX_IPFS_CID_CHAR_LENGTH: usize = 200;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config:
		frame_system::Config + base_nft::Config + CreateSignedTransaction<Call<Self>>
	{
		type Call: From<Call<Self>>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		MaxIpfsCidCharLength,
	}

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NftClassCreated(T::AccountId, T::ClassId, ByteVector),
		IpfsNftMinted(T::AccountId, T::TokenId, ByteVector),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::create_nft_class())]
		pub fn create_nft_class(
			origin: OriginFor<T>,
			ipfs_cid_metadata: ByteVector,
		) -> DispatchResultWithPostInfo {
			let account_id = ensure_signed(origin)?;

			ensure!(
				ipfs_cid_metadata.len() < MAX_IPFS_CID_CHAR_LENGTH,
				Error::<T>::MaxIpfsCidCharLength
			);

			let class_id = BaseNft::<T>::create_class(
				&account_id,
				ipfs_cid_metadata.clone(),
				Default::default(),
			)?;

			Self::deposit_event(Event::NftClassCreated(
				account_id,
				class_id,
				ipfs_cid_metadata,
			));
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 2))]
		pub fn transfer(
			origin: OriginFor<T>,
			from: T::AccountId,
			to: T::AccountId,
			token: (T::ClassId, T::TokenId),
			percentage: u8,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			BaseNft::<T>::transfer(&from, &to, token, percentage)?;

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::mint_ipfs_nft())]
		pub fn mint_ipfs_nft(
			origin: OriginFor<T>,
			ipfs_cid_metadata: ByteVector,
		) -> DispatchResultWithPostInfo {
			let account_id = ensure_signed(origin)?;

			ensure!(
				ipfs_cid_metadata.len() < MAX_IPFS_CID_CHAR_LENGTH,
				Error::<T>::MaxIpfsCidCharLength
			);

			let token_id = BaseNft::<T>::mint(
				&account_id,
				0_u32.into(), // TODO: Replace with enum NftClassId.IpfsNft
				ipfs_cid_metadata.clone(),
				Default::default(),
			)?;

			debug::info!("--- IPFS NFT minted: {:?}", ipfs_cid_metadata);

			Self::deposit_event(Event::IpfsNftMinted(
				account_id,
				token_id,
				ipfs_cid_metadata,
			));
			Ok(().into())
		}
	}

	#[pallet::hooks]

	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
}
