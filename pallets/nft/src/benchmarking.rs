#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use sp_std::{boxed::Box, vec, vec::Vec};

benchmarks! {
	create_nft_class {
		let metadata = vec![5];
		let caller: T::AccountId = whitelisted_caller();
	}: _(RawOrigin::Signed(caller), metadata)
}

impl_benchmark_test_suite!(NftModule, crate::mock::new_test_ext(), crate::mock::Test,);
