#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
use frame_system::{
	offchain::{AppCrypto, CreateSignedTransaction},
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

pub mod local_storage;
pub mod offchain;
pub mod utils;

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
	dna: ByteVector,
	ipfs_cid: ByteVector,
}

#[cfg(test)]
impl TokenData {
	fn new(dna: ByteVector, ipfs_cid: ByteVector) -> Self {
		TokenData { dna, ipfs_cid }
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ orml_nft::Config<TokenData = TokenData, ClassData = ClassData>
		+ CreateSignedTransaction<Call<Self>>
	{
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
		type Call: From<Call<Self>>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type PendingNftQueue<T: Config> = StorageValue<_, PendingNftQueueOf<T>, ValueQuery>;

	#[pallet::error]
	#[derive(PartialEq)]
	pub enum Error<T> {
		NoLocalAccountForSigning,
		OffchainSignedTxError,
		OffchainLock,
		OffchainValueNotFound,
		OffchainValueDecode,
		OffchainLocalNftMetadataAlreadyExist,
		IncorrectNftKeyHash,
		RemoveVectorItem,
	}

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NftClassCreated(T::AccountId, T::ClassId, ClassData, ByteVector),
		NftRequest(PendingNftOf<T>),
		CancelNftRequest(ByteVector, PendingNftOf<T>),
		NftMinted(PendingNftOf<T>, local_storage::KeyHash),
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

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn nft_request(
			origin: OriginFor<T>,
			class_id: T::ClassId,
			token_data: TokenData,
		) -> DispatchResultWithPostInfo {
			let account_id = ensure_signed(origin)?;

			let pending_nft = PendingNft {
				account_id,
				class_id,
				token_data,
			};

			PendingNftQueue::<T>::mutate(|pending_nft_queue| {
				pending_nft_queue.push(pending_nft.clone());
			});

			Self::deposit_event(Event::NftRequest(pending_nft));
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(4, 5))]
		pub fn mint_nft(
			origin: OriginFor<T>,
			key_hash: local_storage::KeyHash,
			pending_nft: PendingNftOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			// TODO: Check if account_id is signed by off-chain worker

			if key_hash
				!= local_storage::get_nft_key_hash::<T>(
					pending_nft.class_id,
					pending_nft.token_data.clone(),
				) {
				debug::error!("--- Error: IncorrectNftKeyHash");
				return Err(Error::<T>::IncorrectNftKeyHash.into());
			}

			let minting_result = OrmlNft::<T>::mint(
				&pending_nft.account_id,
				pending_nft.class_id,
				key_hash.to_vec(),
				pending_nft.token_data.clone(),
			);

			if let Err(error) = minting_result {
				sp_io::offchain_index::clear(&key_hash);

				debug::error!("--- Nft minting error: {:?}", error);
				Self::deposit_event(Event::NftError(error));
				return Err(error.into());
			}

			debug::info!("--- Nft minted: {:?}", pending_nft);

			Self::deposit_event(Event::NftMinted(pending_nft, key_hash));
			Ok(().into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(block_number: T::BlockNumber) {
			let key = offchain::offchain_new_nft_requests_key();

			PendingNftQueue::<T>::mutate(|pending_nft_queue| {
				sp_io::offchain_index::set(&key.0, &pending_nft_queue.encode());
				debug::info!("--- on_finalize block_number: {:?}, new_nft_requests_key: {:x}, new_nft_requests_value: {:?}", block_number, key, pending_nft_queue);

				*pending_nft_queue = PendingNftQueueOf::<T>::new();
			});
		}

		fn offchain_worker(block_number: T::BlockNumber) {
			offchain::hook_init::<T>(block_number);
		}
	}
}
