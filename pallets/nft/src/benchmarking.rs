#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use sp_std::{boxed::Box, vec, vec::Vec};

fn get_ipfs_cid() -> ByteVector {
	vec![1_u8; MAX_IPFS_CID_CHAR_LENGTH - 1]
}

benchmarks! {
	create_nft_class {
		let caller: T::AccountId = whitelisted_caller();
		let ipfs_cid_metadata = get_ipfs_cid();
	}: _(RawOrigin::Signed(caller), ipfs_cid_metadata)

	mint_ipfs_nft {
		let caller: T::AccountId = whitelisted_caller();
		let signed_caller = RawOrigin::Signed(caller);
		let ipfs_cid_metadata = get_ipfs_cid();

		Pallet::<T>::create_nft_class(signed_caller.clone().into(), ipfs_cid_metadata.clone())?;
	}: _(signed_caller, ipfs_cid_metadata)
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Runtime);
