//! Shadow memory for stack

extern crate ethereum_types;

use ethreum_types::{U256};
use super::Shadow;

pub mod const_detector {

	#[derive(Clone, Send)]
	pub enum ShadowConst {
		Undefined,
		Const(U256),
		CallData,
	}

	impl Shadow for ShadowConst {
		fn for_calldata() -> ShadowConst { ShadowConst::Calldata }
		fn for_const(v: U256) -> ShadowConst { ShadowConst::Const(v) }
	}

	impl Default for ShadowConst {
		fn default() -> ShadowConst { ShadowConst::Undefined }
	}
}
