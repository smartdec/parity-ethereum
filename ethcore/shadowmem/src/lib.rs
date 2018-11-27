//! Shadow memory for Parity EVM

pub mod const_detector;
pub mod fake;

extern crate ethereum_types;
extern crate parity_bytes;

use ethereum_types::{U256, H256, Address};
use parity_bytes::Bytes;


pub trait Shadow: Default + Clone + Send {
	fn for_calldata(data: Bytes) -> Self;
	fn for_const(v: U256) -> Self;
	fn for_non_const_address(v: Address) -> Self;
	fn for_const_address(v: Address) -> Self;
	fn for_non_const_word(v: U256) -> Self;
	fn for_memory_size(v: U256) -> Self;
	fn for_env_variable(v: U256) -> Self;
	fn for_const_hash(h: H256) -> Self;
	fn for_non_const_hash(h: H256) -> Self;
	fn for_code(data: Bytes) -> Self;
	fn merge(left: &Self, right: &Self) -> Self;
	fn merge_slices(left: &[Self], right: &[Self]) -> Result<Vec<Self>, &'static str> where Self: Sized {
		if left.len() != right.len() {
			return Err("Merging slices have different length")
		}
		let mut result: Vec<Self> = Vec::with_capacity(left.len());
		for i in 0..left.len() {
			result.push(Self::merge(&left[i], &right[i]));
		}
		return Ok(result);
	}
	/// Aggregate values from slice
	fn aggregate(values: &[Self]) -> Self;
}

pub trait ShadowMemory<T: Shadow> {
	/// Retrieve current size of the memory
	fn size(&self) -> usize;
	/// Resize (shrink or expand) the memory to specified size (fills 0)
	fn resize(&mut self, new_size: usize);
	/// Resize the memory only if its smaller
	fn expand(&mut self, new_size: usize);
	/// Write single byte to memory
	fn write_byte(&mut self, offset: U256, value: T);
	/// Write a word to memory. Does not resize memory!
	fn write(&mut self, offset: U256, value: T);
	/// Read a word from memory
	fn read(&self, offset: U256) -> &[T];
	/// Read a word, then aggregate values
	fn read_aggregated(&self, offset: U256) -> T;
	/// Write slice of bytes to memory. Does not resize memory!
	fn write_slice(&mut self, offset: U256, &[T]);
	/// Retrieve part of the memory between offset and offset + size
	fn read_slice(&self, offset: U256, size: U256) -> &[T];
	/// Retrieve writeable part of memory
	fn writable_slice(&mut self, offset: U256, size: U256) -> &mut[T];
}

/// Checks whether offset and size is valid memory range
pub fn is_valid_range(off: usize, size: usize)  -> bool {
	// When size is zero we haven't actually expanded the memory
	let overflow = off.overflowing_add(size).1;
	size > 0 && !overflow
}

impl<T> ShadowMemory<T> for Vec<T> where T: Shadow {
	fn size(&self) -> usize {
		self.len()
	}

	fn resize(&mut self, new_size: usize) {
		self.resize(new_size, Default::default());
	}

	fn expand(&mut self, size: usize) {
		if size > self.len() {
			ShadowMemory::resize(self, size)
		}
	}

	fn write_byte(&mut self, offset: U256, value: T) {
		let off = offset.low_u64() as usize;
		self[off] = value;
	}

	fn write(&mut self, offset: U256, value: T) {
		let off = offset.low_u64() as usize;
		for val in &mut self[off..off+32] {
			*val = value.clone();
		}
	}

	fn read(&self, offset: U256) -> &[T] {
		let off = offset.low_u64() as usize;
		&self[off..off+32]
	}

	fn read_aggregated(&self, offset: U256) -> T {
		let v = self.read(offset);
		return T::aggregate(v);
	}

	fn write_slice(&mut self, offset: U256, slice: &[T]) {
		if !slice.is_empty() {
			let off = offset.low_u64() as usize;
			self[off..off+slice.len()].clone_from_slice(slice);
		}
	}

	fn read_slice(&self, init_off_u: U256, init_size_u: U256) -> &[T] {
		let off = init_off_u.low_u64() as usize;
		let size = init_size_u.low_u64() as usize;
		if !is_valid_range(off, size) {
			&self[0..0]
		} else {
			&self[off..off+size]
		}
	}

	fn writable_slice(&mut self, offset: U256, size: U256) -> &mut [T] {
		let off = offset.low_u64() as usize;
		let s = size.low_u64() as usize;
		if !is_valid_range(off, s) {
			&mut self[0..0]
		} else {
			&mut self[off..off+s]
		}
	}
}


/// Return data buffer. Holds memory from a previous call and a slice into that memory.
#[derive(Debug)]
pub struct ShadowReturnData<T: Shadow> {
	mem: Vec<T>,
	offset: usize,
	size: usize,
}

impl<T: Shadow + Sized> ::std::ops::Deref for ShadowReturnData<T> {
	type Target = [T];
	fn deref(&self) -> &[T] {
		&self.mem[self.offset..self.offset + self.size]
	}
}

impl<T: Shadow> ShadowReturnData<T> {
	/// Create empty `ShadowReturnData`.
	pub fn empty() -> Self {
		Self {
			mem: Vec::new(),
			offset: 0,
			size: 0,
		}
	}
	/// Create `ShadowReturnData` from give buffer and slice.
	pub fn new(mem: Vec<T>, offset: usize, size: usize) -> Self {
		Self {
			mem: mem,
			offset: offset,
			size: size,
		}
	}
}
