#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use frame_support::{
	dispatch::DispatchResultWithPostInfo, pallet_prelude::*,
	storage::{IterableStorageDoubleMap},
};
use frame_system::pallet_prelude::*;
#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
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

pub type PendingNftOf<T> = PendingNft<<T as frame_system::Config>::AccountId, <T as orml_nft::Config>::ClassId>;

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

#[cfg(test)]
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
	pub(super) type NftPendingQueue<T: Config> = StorageValue<_, Vec<PendingNftOf<T>>, ValueQuery>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NftClassCreated(T::AccountId, T::ClassId, ClassData, Cid),
		NftRequest(PendingNftOf<T>),
		NftMinted(T::TokenId, Cid, PendingNftOf<T>),
	}

	#[pallet::call]
	impl<T:Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 2))]
		pub fn create_nft_class(origin: OriginFor<T>, metadata: Cid) -> DispatchResultWithPostInfo {
			let account_id = ensure_signed(origin)?;

			let class_data = ClassData{}; // TODO: To be expanded
			let class_id = OrmlNft::<T>::create_class(&account_id, metadata.clone(), class_data.clone())?;

			Self::deposit_event(Event::NftClassCreated(account_id, class_id, class_data, metadata));
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn nft_request(origin: OriginFor<T>, class_id: T::ClassId, token_data: TokenData) -> DispatchResultWithPostInfo {
			let account_id = ensure_signed(origin)?;

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
		pub fn root_mint_nft(origin: OriginFor<T>, metadata: Cid, pending_nft: PendingNftOf<T>) -> DispatchResultWithPostInfo {
			let _ = ensure_root(origin)?;

			let token_id = OrmlNft::<T>::mint(
				&pending_nft.account_id,
				pending_nft.class_id.clone(),
				metadata.clone(),
				pending_nft.token_data.clone(),
			)?;

			Self::deposit_event(Event::NftMinted(token_id, metadata, pending_nft));
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
		fn execute_nft_from_pending_queue(pending_nft: PendingNftOf<T>) {
			debug::RuntimeLogger::init();
			debug::info!("--- execute_nft_from_pending_queue(): {:?}", pending_nft);

			let mut tokens_iterator = <orml_nft::Tokens<T> as IterableStorageDoubleMap<T::ClassId, T::TokenId, orml_nft::TokenInfoOf<T>>>
				::iter_prefix(pending_nft.class_id);

			let mut unique_dna = true;
			while let Some((token_id, token_info)) = tokens_iterator.next() {
				debug::info!("--- token_id: {:?}, token_info: {:?}", token_id, token_info);
				
				if pending_nft.token_data.dna == token_info.data.dna {
					unique_dna = false;
					break;
				}
			}

			if !unique_dna {
				debug::info!("--- Not a unique dna: {:?}", pending_nft.token_data.dna);
				return
			}
			
			debug::info!("--- Create a new NFT on-chain");
		}
	}
}
