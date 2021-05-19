use crate::{mock::{Event, *}};
use frame_support::{assert_ok, assert_noop};

const ALICE: AccountId = AccountId::new([1u8; 32]);
const CLASS_ID: <Runtime as orml_nft::Config>::ClassId = 0;
const TOKEN_ID: <Runtime as orml_nft::Config>::TokenId = 0;

#[test]
fn mint_nft_works() {
	new_test_ext().execute_with(|| {
        assert_noop!(
            Nft::mint_nft(Origin::signed(ALICE), CLASS_ID, vec![1]),
            orml_nft::Error::<Runtime>::ClassNotFound
        );

        assert_ok!(Nft::create_nft_class(Origin::signed(ALICE), vec![2]));

        let event = Event::pallet_nft(
            crate::Event::NftClassCreated(
                ALICE,
                CLASS_ID,
                vec![2],
                Default::default(),
            )
        );
        assert_eq!(last_event(), event);
        
        assert_ok!(Nft::mint_nft(Origin::signed(ALICE), CLASS_ID, vec![1]));

        let dna = vec![0, 1, 2];
        let event = Event::pallet_nft(
            crate::Event::NftMinted(
                ALICE,
                CLASS_ID,
                TOKEN_ID,
                vec![1],
                crate::TokenData::new(dna),
            )
        );
        assert_eq!(last_event(), event);
	});
}