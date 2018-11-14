//! Shadow memory for stack

extern crate ethereum_types;
extern crate parity_bytes;

use ethereum_types::{U256, Address. H256};
use super::Shadow;
use parity_bytes::Bytes;


#[derive(Clone)]
pub enum ShadowConst {
	Undefined,
	Const,
	NoConst,
}

impl Shadow for ShadowConst {
	fn for_calldata(data: Bytes) -> ShadowConst { ShadowConst::NoConst }
	fn for_const(v: U256) -> ShadowConst { ShadowConst::Const }
	fn for_no_const_address(v: Address) -> ShadowConst { ShadowConst::NoConst }
	fn for_const_address(v: Address) -> ShadowConst { ShadowConst::Const }
	fn for_no_const_word(v: U256) -> ShadowConst { ShadowConst::NoConst }
	fn for_memory_size(v: U256) -> ShadowConst { ShadowConst::NoConst }
	fn for_env_variable(v: U256) -> ShadowConst { ShadowConst::NoConst }
	fn for_const_hash(h: H256) -> ShadowConst { ShadowConst::Const }
	fn for_no_const_hash(h: H256) -> ShadowConst { ShadowConst::Const }
	fn for_external_code(data: Bytes) -> ShadowConst { ShadowConst::NoConst }
	fn merge(left: &Self, right: &Self) -> ShadowConstb{
		match *left {
			ShadowConst::Const => *right.clone(),
			ShadowConst::Undefined => ShadowConst::Undefined,
			ShadowConst::NoConst => match *right {
				ShadowConst::Const => ShadowConst::NoConst,
				_ => *right.clone()
			}
		}
	}
}

impl Default for ShadowConst {
	fn default() -> ShadowConst { ShadowConst::Undefined }
}
