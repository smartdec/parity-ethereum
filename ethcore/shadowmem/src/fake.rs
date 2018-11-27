//! Fake Shadow memory

extern crate ethereum_types;
extern crate parity_bytes;

use ethereum_types::{U256, Address, H256};
use super::Shadow;
use parity_bytes::Bytes;

pub type ShadowFake = ();

impl Shadow for ShadowFake {
	fn for_calldata(_data: Bytes) -> ShadowFake { () }
	fn for_const(_v: U256) -> ShadowFake { () }
	fn for_non_const_address(_v: Address) -> ShadowFake { () }
	fn for_const_address(_v: Address) -> ShadowFake { () }
	fn for_non_const_word(_v: U256) -> ShadowFake { () }
	fn for_memory_size(_v: U256) -> ShadowFake { () }
	fn for_env_variable(_v: U256) -> ShadowFake { () }
	fn for_const_hash(_h: H256) -> ShadowFake { () }
	fn for_non_const_hash(_h: H256) -> ShadowFake { () }
	fn for_code(_data: Bytes) -> ShadowFake { () }

	fn merge(&left: &Self, &right: &Self) -> ShadowFake { () }
	fn aggregate(values: &[Self]) -> ShadowFake { () }
}
