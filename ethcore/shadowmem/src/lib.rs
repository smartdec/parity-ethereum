//! Shadow memory for Parity EVM

pub mod const_detector;

extern crate ethereum_types;
extern crate parity_bytes;

use ethereum_types::{U256, U512, H256, Address};
use parity_bytes::Bytes;


pub trait Shadow: Default + Clone + Send {
	fn for_calldata(data: Bytes) -> Self;
	fn for_const(v: U256) -> Self;
	fn for_no_const_address(v: Address) -> Self;
	fn for_const_address(v: Address) -> Self;
	fn for_no_const_word(v: U256) -> Self;
	fn for_memory_size(v: U256) -> Self;
	fn for_env_variable(v: U256) -> Self;
	fn for_const_hash(h: H256) -> Self;
	fn for_no_const_hash(h: H256) -> Self;
	fn for_external_code(data: Bytes) -> Self;
	fn merge(left: &Self, right: &Self) -> Self;
}
