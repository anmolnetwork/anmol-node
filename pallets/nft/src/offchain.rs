use crate::{
    PendingNftOf,
    PendingNftQueueOf,
    Config,
    Error,
    pallet::{Call},
    local_storage::{LocalStorageValue, VecKey},
};
use sp_core::{
	crypto::KeyTypeId,
};
use sp_std::{
    vec::Vec,
};
use frame_support::{
    storage::{IterableStorageDoubleMap},
    pallet_prelude::{debug},
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

const NEW_NFT_REQUESTS_KEY: &[u8] = b"new_nft_requests";
const NFT_PENDING_QUEUE: &[u8] = b"nft_pending_queue";

pub fn hook_init<T: Config>(block_number: T::BlockNumber) {
    debug::info!("--- offchain_worker block_number: {:?}", block_number);

    let new_nft_requests = LocalStorageValue::<PendingNftQueueOf<T>>::new(NEW_NFT_REQUESTS_KEY);
    let pending_nft_queue = LocalStorageValue::<PendingNftQueueOf<T>>::new(NFT_PENDING_QUEUE);

    let result = offchain_update_pending_nft_queue::<T>(pending_nft_queue, new_nft_requests);

    match result {
        Ok(pending_nft_queue) => {
            if pending_nft_queue.len() > 0 {
                debug::info!("--- Pending nft queue: {:?}", pending_nft_queue);
                execute_nft_from_pending_queue::<T>(pending_nft_queue[0].clone());
            }
        },
        Err(x) => {
            debug::error!("--- result error: {:?}", x);
        }
    }
}

pub fn offchain_new_nft_requests_key() -> VecKey {
    let key = NEW_NFT_REQUESTS_KEY.to_vec();
    VecKey(key)
}

fn offchain_update_pending_nft_queue<T: Config>
    (
        pending_nft_queue: LocalStorageValue::<PendingNftQueueOf<T>>,
        new_nft_requests: LocalStorageValue::<PendingNftQueueOf<T>>
    ) -> Result<PendingNftQueueOf<T>, Error<T>> {
    pending_nft_queue.mutate(|mut current_pending_nft_queue| {
        let new_nft_requests = new_nft_requests.get()?;

        for v in new_nft_requests {
            current_pending_nft_queue.push(v);
        }
        Ok(current_pending_nft_queue)
    })
}

fn execute_nft_from_pending_queue<T: Config>(pending_nft: PendingNftOf<T>) {
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