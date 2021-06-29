use crate::{
	local_storage,
	mock::{Event, *},
	pallet::PendingNftQueue,
	Error, PendingNft, TokenData,
};
use frame_support::{assert_noop, assert_ok};

const ALICE: AccountId = AccountId::new([1u8; 32]);
const CLASS_ID: <Runtime as orml_nft::Config>::ClassId = 0;

#[test]
fn mint_nft_works() {
	new_test_ext().execute_with(|| {
		let pending_nft = PendingNft {
			account_id: ALICE,
			class_id: CLASS_ID,
			token_data: TokenData::new(vec![0, 1, 2]),
		};

		assert_ok!(Nft::nft_request(
			Origin::signed(ALICE),
			CLASS_ID,
			pending_nft.clone().token_data
		));

		// TODO: DispatchError for ClassNotFound
		// assert_noop!(
		//     Nft::mint_nft(Origin::signed(ALICE), vec![0], pending_nft.clone()),
		//     crate::Error::<Runtime>::NftError(orml_nft::Error::<Runtime>::ClassNotFound)
		// );

		assert_ok!(Nft::create_nft_class(Origin::signed(ALICE), vec![1]));

		let event = Event::pallet_nft(crate::Event::NftClassCreated(
			ALICE,
			CLASS_ID,
			Default::default(),
			vec![1],
		));
		assert_eq!(last_event(), event);

		assert_noop!(
			Nft::mint_nft(Origin::signed(ALICE), [0_u8; 16], pending_nft.clone()),
			Error::<Runtime>::IncorrectNftKeyHash
		);

		let nft_key_hash =
			local_storage::get_nft_key_hash::<Runtime>(CLASS_ID, pending_nft.clone().token_data);
		assert_ok!(Nft::mint_nft(
			Origin::signed(ALICE),
			nft_key_hash,
			pending_nft.clone()
		));

		let event = Event::pallet_nft(crate::Event::NftMinted(pending_nft, nft_key_hash));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn nft_request_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(PendingNftQueue::<Runtime>::get(), vec![]);

		let token_data = TokenData::new(vec![0, 0, 1]);

		assert_ok!(Nft::nft_request(
			Origin::signed(ALICE),
			CLASS_ID,
			token_data.clone()
		));

		let pending_nft = PendingNft {
			account_id: ALICE,
			class_id: CLASS_ID,
			token_data,
		};

		let event = Event::pallet_nft(crate::Event::NftRequest(pending_nft.clone()));
		assert_eq!(last_event(), event);

		assert_eq!(PendingNftQueue::<Runtime>::get(), vec![pending_nft]);
	});
}
