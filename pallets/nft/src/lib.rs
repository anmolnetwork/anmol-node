#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
use frame_system::{
	pallet_prelude::*,
};

use orml_nft::Module as OrmlNft;
pub use pallet::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::DispatchError;
use sp_std::{cmp::Ordering, vec::Vec};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type ByteVector = Vec<u8>;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, Ord)]
pub struct PendingNft<AccountId, ClassId> {
	account_id: AccountId,
	class_id: ClassId,
	token_data: TokenData,
}

impl<AccountId, ClassId> PartialOrd for PendingNft<AccountId, ClassId>
where
	AccountId: Ord,
	ClassId: Ord,
{
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

type PendingNftOf<T> =
	PendingNft<<T as frame_system::Config>::AccountId, <T as orml_nft::Config>::ClassId>;
pub type PendingNftQueueOf<T> = Vec<PendingNftOf<T>>;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct ClassData {
	// To be expanded
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, PartialOrd, Ord)]
pub struct TokenData {
	// To be expanded
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ orml_nft::Config<TokenData = TokenData, ClassData = ClassData>
	{
		type Call: From<Call<Self>>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type PendingNftQueue<T: Config> = StorageValue<_, PendingNftQueueOf<T>, ValueQuery>;

	#[pallet::error]
	pub enum Error<T> {
		
	}

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NftClassCreated(T::AccountId, T::ClassId, ClassData, ByteVector),
		IpfsNftMinted(T::AccountId, T::TokenId, ByteVector),
		NftError(DispatchError),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 2))]
		pub fn create_nft_class(
			origin: OriginFor<T>,
			metadata: ByteVector,
		) -> DispatchResultWithPostInfo {
			let account_id = ensure_signed(origin)?;

			let class_data = ClassData {}; // TODO: To be expanded
			let class_id =
				OrmlNft::<T>::create_class(&account_id, metadata.clone(), class_data.clone())?;

			Self::deposit_event(Event::NftClassCreated(
				account_id, class_id, class_data, metadata,
			));
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(0, 0))]
		pub fn mint_ipfs_nft(
			origin: OriginFor<T>,
			ipfs_cid: ByteVector,
		) -> DispatchResultWithPostInfo {
			let account_id = ensure_signed(origin)?;

			let token_id = OrmlNft::<T>::mint(
				&account_id,
				0_u32.into(), // TODO: Replace with enum NftClassId.IpfsNft
				ipfs_cid.clone(),
				Default::default(),
			)?;

			debug::info!("--- IPFS NFT minted: {:?}", ipfs_cid);

			Self::deposit_event(Event::IpfsNftMinted(account_id, token_id, ipfs_cid));
			Ok(().into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {

	}
}
