//! Shadow memory for stack

extern crate ethereum_types;
extern crate parity_bytes;

use ethereum_types::{U256, Address, H256};
use super::Shadow;
use parity_bytes::Bytes;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadowConst {
	Undefined,
	Const,
	NonConst,
}

impl Shadow for ShadowConst {
	fn for_calldata(_data: Bytes) -> ShadowConst { ShadowConst::NonConst }
	fn for_const(_v: U256) -> ShadowConst { ShadowConst::Const }
	fn for_non_const_address(_v: Address) -> ShadowConst { ShadowConst::NonConst }
	fn for_const_address(_v: Address) -> ShadowConst { ShadowConst::Const }
	fn for_non_const_word(_v: U256) -> ShadowConst { ShadowConst::NonConst }
	fn for_memory_size(_v: U256) -> ShadowConst { ShadowConst::NonConst }
	fn for_env_variable(_v: U256) -> ShadowConst { ShadowConst::NonConst }
	fn for_const_hash(_h: H256) -> ShadowConst { ShadowConst::Const }
	fn for_non_const_hash(_h: H256) -> ShadowConst { ShadowConst::Const }
	fn for_code(_data: Bytes) -> ShadowConst { ShadowConst::Const }

	fn merge(&left: &Self, &right: &Self) -> ShadowConst {
		match left {
			ShadowConst::Const => right,
			ShadowConst::Undefined => ShadowConst::Undefined,
			ShadowConst::NonConst => match right {
				ShadowConst::Const => ShadowConst::NonConst,
				_ => right
			}
		}
	}
	fn aggregate(values: &[Self]) -> Self {
		let mut result = ShadowConst::Const;
		for val in values {
			if result == ShadowConst::Const && *val == ShadowConst::NonConst {
				result = ShadowConst::NonConst;
			}
			if *val == ShadowConst::Undefined {
				result = ShadowConst::Undefined;
			}
		}
		return result;
	}
}

impl Default for ShadowConst {
	fn default() -> ShadowConst { ShadowConst::Undefined }
}
