#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
use frame_system::pallet_prelude::*;
#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
use sp_std::vec;
use sp_std::vec::Vec;
use orml_nft::Module as OrmlNft;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ClassData {
	// To be expanded
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct TokenData {
	dna: Vec<u8>,
	// To be expanded
}

pub type Cid = Vec<u8>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + orml_nft::Config<TokenData = TokenData, ClassData = ClassData> {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NftClassCreated(T::AccountId, T::ClassId, Cid, ClassData),
		NftMinted(T::AccountId, T::ClassId, T::TokenId, Cid, TokenData),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T:Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 2))]
		pub fn create_nft_class(origin: OriginFor<T>, metadata: Cid) -> DispatchResultWithPostInfo {
			let account_id = ensure_signed(origin)?;

			let class_data = ClassData{}; // TODO: To be expanded
			let class_id = OrmlNft::<T>::create_class(&account_id, metadata.clone(), class_data.clone())?;

			Self::deposit_event(Event::NftClassCreated(account_id, class_id, metadata, class_data));
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 4))]
		pub fn mint_nft(origin: OriginFor<T>, class_id: T::ClassId, metadata: Cid) -> DispatchResultWithPostInfo {
			let account_id = ensure_signed(origin)?;

			let dna = vec![0, 1, 2]; // TODO: Generic DNA
			let token_data = TokenData {
				dna,
			};
			let token_id = OrmlNft::<T>::mint(&account_id, class_id.clone(), metadata.clone(), token_data.clone())?;

			Self::deposit_event(Event::NftMinted(account_id, class_id, token_id, metadata, token_data));
			Ok(().into())
		}
	}
}
