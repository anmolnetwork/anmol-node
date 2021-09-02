#![cfg(test)]

use crate::mock::{Event, *};
use frame_support::{assert_noop, assert_ok};

const ALICE: AccountId = AccountId::new([1u8; 32]);
const CLASS_ID_IPFS_NFT: <Runtime as base_nft::Config>::ClassId = 0;

#[test]
fn mint_ipfs_nft_works() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Nft::mint_ipfs_nft(Origin::signed(ALICE), vec![0, 1, 2]),
			base_nft::Error::<Runtime>::ClassNotFound,
		);

		let too_long_ipfs_cid = vec![1_u8; crate::MAX_IPFS_CID_CHAR_LENGTH];
		assert_noop!(
			Nft::create_nft_class(Origin::signed(ALICE), too_long_ipfs_cid.clone()),
			crate::Error::<Runtime>::MaxIpfsCidCharLength
		);
		assert_ok!(Nft::create_nft_class(Origin::signed(ALICE), vec![1]));

		let event = Event::pallet_nft(crate::Event::NftClassCreated(
			ALICE,
			CLASS_ID_IPFS_NFT,
			vec![1],
		));
		assert_eq!(last_event(), event);

		assert_noop!(
			Nft::mint_ipfs_nft(Origin::signed(ALICE), too_long_ipfs_cid),
			crate::Error::<Runtime>::MaxIpfsCidCharLength
		);
		assert_ok!(Nft::mint_ipfs_nft(Origin::signed(ALICE), vec![0, 1, 2]),);

		let event = Event::pallet_nft(crate::Event::IpfsNftMinted(ALICE, 0, vec![0, 1, 2]));
		assert_eq!(last_event(), event);
	});
}
