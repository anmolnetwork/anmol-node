#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use sp_std::{boxed::Box, vec, vec::Vec};

benchmarks! {
	create_nft_class {
		let caller: T::AccountId = whitelisted_caller();
		let ipfs_cid_metadata = ByteVector::default();
	}: _(RawOrigin::Signed(caller), ipfs_cid_metadata)

	mint_ipfs_nft {
		let caller: T::AccountId = whitelisted_caller();
		let signed_caller = RawOrigin::Signed(caller);
		let ipfs_cid_metadata = ByteVector::default();

		Pallet::<T>::create_nft_class(signed_caller.clone().into(), Default::default())?;
	}: _(signed_caller, ipfs_cid_metadata)
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Runtime);
