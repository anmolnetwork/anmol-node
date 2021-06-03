#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use frame_support::{
	dispatch::{DispatchResultWithPostInfo, DispatchResult}, pallet_prelude::*,
	storage::{IterableStorageDoubleMap},
};
use frame_system::{
	pallet_prelude::*,
	offchain::{self, CreateSignedTransaction, AppCrypto, Signer, SendSignedTransaction},
};
#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
use sp_std::{
	vec::Vec,
	cmp::{Ordering},
	str,
	fmt,
};
use sp_core::{
	crypto::KeyTypeId,
};
use sp_runtime::{
	DispatchError,
	offchain::{
		storage::{StorageValueRef},
	},
};
use orml_nft::Module as OrmlNft;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"_nft");

pub mod crypto {
	use crate::KEY_TYPE;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		MultiSignature, MultiSigner,
	};
	use frame_system::{
		offchain::{AppCrypto},
	};

	app_crypto!(sr25519, KEY_TYPE);

	pub struct TestAuthId;

	impl AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

pub type TextMessage = Vec<u8>;

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

pub type PendingNftOf<T> = PendingNft<<T as frame_system::Config>::AccountId, <T as orml_nft::Config>::ClassId>;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct ClassData {
	// To be expanded
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, PartialOrd, Ord)]
pub struct TokenData {
	dna: TextMessage,
	// To be expanded
}

#[cfg(test)]
impl TokenData {
	fn new(dna: TextMessage) -> Self {
		TokenData {
			dna,
		}
	}
}

struct OffchainStorageKey(Vec<u8>);

impl fmt::LowerHex for OffchainStorageKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.0 {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

const LOCAL_PENDING_NFT_QUEUE: &[u8] = b"nft_pending_queue";
const LOCAL_NFT_FROM_BLOCK_PREFIX: &[u8] = b"nft_from_block_";

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config
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
	pub(super) type NftPendingQueue<T: Config> = StorageValue<_, Vec<PendingNftOf<T>>, ValueQuery>;

	#[pallet::error]
	pub enum Error<T> {
		NoLocalAccountForSigning,
		OffchainSignedTxError,
		TryToRemoveNftWhichDoesNotExist,
		OffchainLock,
		OffchainValueNotFound,
		OffchainValueDecode,
		OffchainValueMutate,
	}

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NftClassCreated(T::AccountId, T::ClassId, ClassData, TextMessage),
		NftRequest(PendingNftOf<T>),
		CancelNftRequest(TextMessage, PendingNftOf<T>),
		NftMinted(PendingNftOf<T>, TextMessage),
		NftError(DispatchError),
	}

	#[pallet::call]
	impl<T:Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 2))]
		pub fn create_nft_class(origin: OriginFor<T>, metadata: TextMessage) -> DispatchResultWithPostInfo {
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

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 1))]
		pub fn cancel_nft_request(origin: OriginFor<T>, pending_nft: PendingNftOf<T>, reason: TextMessage) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			// TODO: Check if account_id is signed by off-chain worker

			Self::remove_nft_from_pending_queue(pending_nft.clone())?;

			Self::deposit_event(Event::CancelNftRequest(reason, pending_nft));
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(4, 5))]
		pub fn mint_nft(origin: OriginFor<T>, metadata: TextMessage, pending_nft: PendingNftOf<T>) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			// TODO: Check if account_id is signed by off-chain worker

			Self::remove_nft_from_pending_queue(pending_nft.clone())?;

			let minting_result = OrmlNft::<T>::mint(
				&pending_nft.account_id,
				pending_nft.class_id.clone(),
				metadata.clone(),
				pending_nft.token_data.clone(),
			);

			if let Err(error) = minting_result {
				debug::error!("--- Nft minting error: {:?}", error);
				Self::deposit_event(Event::NftError(error));

				return Err(error.into())
			}

			debug::info!("--- Nft minted: {:?}", pending_nft);

			Self::deposit_event(Event::NftMinted(pending_nft, metadata));
			Ok(().into())
		}
	}

	impl<T:Config> Pallet<T> {
		fn remove_nft_from_pending_queue(pending_nft: PendingNftOf<T>) -> DispatchResult {
			let mut nft_pending_queue = NftPendingQueue::<T>::get();

			match nft_pending_queue.binary_search(&pending_nft) {
				Ok(index) => {
					nft_pending_queue.remove(index);
					NftPendingQueue::<T>::put(nft_pending_queue);
					debug::info!("--- Removed nft from pending_queue: {:?}", pending_nft);

					Ok(())
				},
				Err(_) => Err(Error::<T>::TryToRemoveNftWhichDoesNotExist.into())
			}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(block_number: T::BlockNumber) {
			let key = Self::get_offchain_new_nft_items_key(block_number);
			let pending_nft_queue = NftPendingQueue::<T>::get();

			sp_io::offchain_index::set(&key.0, &pending_nft_queue.encode());
			
			debug::info!("--- on_finalize block_number: {:?}, key: {:x}, value: {:?}", block_number, key, pending_nft_queue);
			
			NftPendingQueue::<T>::put(Vec::<PendingNftOf<T>>::new());
		}

		fn offchain_worker(block_number: T::BlockNumber) {
			debug::info!("--- offchain_worker block_number: {:?}", block_number);

			// let key = Self::get_offchain_new_nft_items_key(block_number);
			// let local_data: Vec<PendingNftOf::<T>> = match StorageValueRef::persistent(&key.0).get() {
			// 	Some(Some(x)) => x,
			// 	Some(None) => {
			// 		debug::info!("--- error OffchainValueDecode");
			// 		Vec::<PendingNftOf<T>>::new()
			// 	},
			// 	None => {
			// 		debug::info!("--- error OffchainValueNotFound");
			// 		Vec::<PendingNftOf<T>>::new()
			// 	}
			// };
			// debug::info!("--- offchain_worker key: {:x} value: {:?}", key, local_data);



			let result = Self::offchain_update_local_pending_nft_queue(block_number);

			match result {
				Ok(pending_nft_queue) => {
					if pending_nft_queue.len() > 0 {
						debug::info!("--- execute pending_nft_queue: {:?}", pending_nft_queue);
						Self::execute_nft_from_pending_queue(pending_nft_queue[0].clone());
					}
				},
				Err(x) => {
					debug::error!("--- result error: {:?}", x);
				}
			}
		}
	}

	impl<T:Config> Pallet<T> {
		fn get_offchain_new_nft_items_key(block_number: T::BlockNumber) -> OffchainStorageKey {
			let mut key = LOCAL_NFT_FROM_BLOCK_PREFIX.to_vec();
			key.extend(block_number.encode());
			OffchainStorageKey(key)
		}

		fn get_offchain_new_nft_items(block_number: T::BlockNumber) -> Result<Vec<PendingNftOf<T>>, Error<T>>{
			let key = Self::get_offchain_new_nft_items_key(block_number);
			let new_nft_items = StorageValueRef::persistent(&key.0)
				.get::<Vec<PendingNftOf<T>>>()
				.ok_or(Error::<T>::OffchainValueNotFound)?
				.ok_or(Error::<T>::OffchainValueDecode)?;

			debug::info!("--- get_offchain_new_nft_items key: {:x}, new_nft_items: {:?}", key, new_nft_items);

			Ok(new_nft_items)
		}

		fn offchain_update_local_pending_nft_queue(block_number: T::BlockNumber) -> Result<Vec<PendingNftOf<T>>, Error<T>> {
			let value_ref = StorageValueRef::persistent(LOCAL_PENDING_NFT_QUEUE);
			let result = value_ref.mutate(|x: Option<Option<Vec<PendingNftOf<T>>>>| {
				match x {
					Some(Some(mut current_pending_nft_queue)) => {
						debug::info!("--- current_pending_nft_queue: {:?}", current_pending_nft_queue);
						let new_nft_items = Self::get_offchain_new_nft_items(block_number)?;
						for v in new_nft_items {
							current_pending_nft_queue.push(v);
						}
						
						Ok(current_pending_nft_queue)
					},
					_ => {
						debug::info!("--- current_pending_nft_queue: EMPTY");
						Ok(Vec::<PendingNftOf<T>>::new())
					},
				}
			});

			match result {
				Ok(Ok(pending_nft_queue)) => {
					Ok(pending_nft_queue)
				},
				Err(x) => Err(x),
				Ok(Err(_)) => Err(Error::<T>::OffchainLock),
			}
		}

		fn execute_nft_from_pending_queue(pending_nft: PendingNftOf<T>) {
			debug::RuntimeLogger::init();
			debug::info!("--- Execute nft from pending queue: {:?}", pending_nft);

			let mut tokens_iterator = <orml_nft::Tokens<T> as IterableStorageDoubleMap<T::ClassId, T::TokenId, orml_nft::TokenInfoOf<T>>>
				::iter_prefix(pending_nft.class_id);

			let mut unique_dna = true;
			while let Some((token_id, token_info)) = tokens_iterator.next() {
				debug::info!("--- Token to compare uniqueness: token_id: {:?}, token_info: {:?}", token_id, token_info);

				if pending_nft.token_data.dna == token_info.data.dna {
					unique_dna = false;
					break;
				}
			}

			if !unique_dna {
				debug::info!("--- Not a unique dna: {:?}", pending_nft.token_data.dna);

				let cancel_nft_closure = |_: &_| return Call::cancel_nft_request(pending_nft.clone(), b"DNA is not unique".to_vec());
				let _result = Self::send_signed(cancel_nft_closure);
				return
			}

			// TODO: Replace metadata with IPFS CID
			let metadata = Vec::new();

			let mint_nft_closure = |_: &_| return Call::mint_nft(metadata.clone(), pending_nft.clone());
			let _result = Self::send_signed(mint_nft_closure);
		}

		fn send_signed(call_closure: impl Fn(&offchain::Account<T>) -> Call<T>) -> Result<(), Error<T>> {
			let signer = Signer::<T, T::AuthorityId>::any_account();
			let result = signer.send_signed_transaction(call_closure);

			if let Some((acc, res)) = result {
				if res.is_err() {
					debug::error!("--- Send signed - Error: {:?}, account id: {:?}", res, acc.id);
					return Err(Error::<T>::OffchainSignedTxError)
				}

				debug::info!("--- Send signed - Ok");
				return Ok(());
			} 

			debug::error!("--- Send signed - No local account available");
			return Err(Error::<T>::NoLocalAccountForSigning);
		}
	}
}
