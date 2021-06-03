use crate::{
    PendingNftOf,
    Config,
    Error,
    pallet::{Call},
};
use sp_core::{
	crypto::KeyTypeId,
};
use sp_std::{
    fmt,
    vec::Vec,
};
use sp_runtime::{
    offchain::{
		storage::{StorageValueRef},
	},
};
use frame_support::{
    storage::{IterableStorageDoubleMap},
    pallet_prelude::{debug, Encode},
};
use frame_system::{
    offchain::{Signer, SendSignedTransaction},
};

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"_nft");

pub mod crypto {
	use super::KEY_TYPE;
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

pub struct StorageKey(pub Vec<u8>);

impl fmt::LowerHex for StorageKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.0 {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

pub const LOCAL_PENDING_NFT_QUEUE: &[u8] = b"nft_pending_queue";
pub const LOCAL_NFT_FROM_BLOCK_PREFIX: &[u8] = b"nft_from_block_";

pub fn hook_init<T: Config>(block_number: T::BlockNumber) {
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



    let result = offchain_update_local_pending_nft_queue::<T>(block_number);

    match result {
        Ok(pending_nft_queue) => {
            if pending_nft_queue.len() > 0 {
                debug::info!("--- execute pending_nft_queue: {:?}", pending_nft_queue);
                execute_nft_from_pending_queue::<T>(pending_nft_queue[0].clone());
            }
        },
        Err(x) => {
            debug::error!("--- result error: {:?}", x);
        }
    }
}

pub fn get_offchain_new_nft_items_key<T: Config>(block_number: T::BlockNumber) -> StorageKey {
    let mut key = LOCAL_NFT_FROM_BLOCK_PREFIX.to_vec();
    key.extend(block_number.encode());
    StorageKey(key)
}

fn get_offchain_new_nft_items<T: Config>(block_number: T::BlockNumber) -> Result<Vec<PendingNftOf<T>>, Error<T>>{
    let key = get_offchain_new_nft_items_key::<T>(block_number);
    let new_nft_items = StorageValueRef::persistent(&key.0)
        .get::<Vec<PendingNftOf<T>>>()
        .ok_or(Error::<T>::OffchainValueNotFound)?
        .ok_or(Error::<T>::OffchainValueDecode)?;

    debug::info!("--- get_offchain_new_nft_items key: {:x}, new_nft_items: {:?}", key, new_nft_items);

    Ok(new_nft_items)
}

pub fn offchain_update_local_pending_nft_queue<T: Config>(block_number: T::BlockNumber) -> Result<Vec<PendingNftOf<T>>, Error<T>> {
    let value_ref = StorageValueRef::persistent(LOCAL_PENDING_NFT_QUEUE);
    let result = value_ref.mutate(|x: Option<Option<Vec<PendingNftOf<T>>>>| {
        match x {
            Some(Some(mut current_pending_nft_queue)) => {
                debug::info!("--- current_pending_nft_queue: {:?}", current_pending_nft_queue);
                let new_nft_items = get_offchain_new_nft_items(block_number)?;
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

pub fn execute_nft_from_pending_queue<T: Config>(pending_nft: PendingNftOf<T>) {
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

        let cancel_nft_closure = |_: &frame_system::offchain::Account<T>| return Call::cancel_nft_request(pending_nft.clone(), b"DNA is not unique".to_vec());
        let _result = send_signed(cancel_nft_closure);
        return
    }

    // TODO: Replace metadata with IPFS CID
    let metadata = Vec::new();

    let mint_nft_closure = |_: &frame_system::offchain::Account<T>| return Call::mint_nft(metadata.clone(), pending_nft.clone());
    let _result = send_signed(mint_nft_closure);
}

fn send_signed<T: Config>(call_closure: impl Fn(&frame_system::offchain::Account<T>) -> Call<T>) -> Result<(), Error<T>> {
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