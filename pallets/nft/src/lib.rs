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

pub type Dna = Vec<u8>;
pub type Cid = Vec<u8>;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct PendingNft<AccountId, ClassId> {
	account_id: AccountId,
	class_id: ClassId,
	token_data: TokenData,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct ClassData {
	// To be expanded
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct TokenData {
	dna: Dna,
	// To be expanded
}

impl TokenData {
	fn new(dna: Dna) -> Self {
		TokenData {
			dna,
		}
	}
}

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

	#[pallet::storage]
	pub(super) type NftPendingQueue<T: Config> = StorageValue<_, Vec<PendingNft<T::AccountId, T::ClassId>>, ValueQuery>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NftClassCreated(T::AccountId, T::ClassId, Cid, ClassData),
		NftRequest(PendingNft<T::AccountId, T::ClassId>),
		NftMinted(T::AccountId, T::ClassId, T::TokenId, Cid, TokenData),
	}

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

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn nft_request(origin: OriginFor<T>, class_id: T::ClassId, token_data: TokenData) -> DispatchResultWithPostInfo {
			let account_id = ensure_signed(origin)?;

			// TODO: Check DNA if valid and unique

			let pending_nft = PendingNft {
				account_id,
				class_id,
				token_data,
			};

			NftPendingQueue::<T>::mutate(|pending_nft_queue| {
				pending_nft_queue.push(pending_nft.clone());
			});

			Self::deposit_event(Event::NftRequest(pending_nft));
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 4))]
		pub fn mint_nft(origin: OriginFor<T>, class_id: T::ClassId, metadata: Cid) -> DispatchResultWithPostInfo {
			let account_id = ensure_signed(origin)?;

			let dna = vec![0, 1, 2]; // TODO: Generic DNA
			let token_data = TokenData::new(dna);
			let token_id = OrmlNft::<T>::mint(&account_id, class_id.clone(), metadata.clone(), token_data.clone())?;

			Self::deposit_event(Event::NftMinted(account_id, class_id, token_id, metadata, token_data));
			Ok(().into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(_block_number: T::BlockNumber) {
			let pending_nft_queue = NftPendingQueue::<T>::get();
			if pending_nft_queue.len() > 0 {
				Self::execute_nft_from_pending_queue(pending_nft_queue[0].clone());
			}
		}
	}

	impl<T:Config> Pallet<T> {
		fn execute_nft_from_pending_queue(pending_nft: PendingNft<T::AccountId, T::ClassId>) {
			debug::RuntimeLogger::init();
			debug::info!("--- execute_nft_from_pending_queue(): {:?}", pending_nft);
		}
	}
}
