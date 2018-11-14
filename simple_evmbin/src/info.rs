// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! VM runner.

use std::time::{Instant, Duration};
use ethereum_types::{H256, U256};
use ethcore::client::{self, EvmTestClient, EvmTestError, TransactResult};
use ethcore::{trace, spec, pod_state};
use ethjson;
use transaction;
use vm::ActionParams;
use rlp::{self, RlpStream, Rlp, DecoderError, Encodable};

use rustc_hex::FromHex;

/// VM execution informant
pub trait Informant: trace::VMTracer {
	/// Display a single run init message
	fn before_test(&mut self, test: &str, action: &str);
	/// Set initial gas.
	fn set_gas(&mut self, _gas: U256) {}
	/// Display final result.
	fn finish(result: RunResult<Self::Output>);
}

/// Execution finished correctly
#[derive(Debug)]
pub struct Success<T> {
	/// State root
	pub state_root: H256,
	/// Used gas
	pub gas_used: U256,
	/// Output as bytes
	pub output: Vec<u8>,
	/// Time Taken
	pub time: Duration,
	/// Traces
	pub traces: Option<T>,
}

/// Execution failed
#[derive(Debug)]
pub struct Failure<T> {
	/// Used gas
	pub gas_used: U256,
	/// Internal error
	pub error: EvmTestError,
	/// Duration
	pub time: Duration,
	/// Traces
	pub traces: Option<T>,
}

/// EVM Execution result
pub type RunResult<T> = Result<Success<T>, Failure<T>>;

pub fn run_transaction<T: Informant>(
	env_info: &mut client::EnvInfo,
	tx: transaction::SignedTransaction,
	mut informant: T,
	client: &mut EvmTestClient
) -> TransactResult<trace::FlatTrace, T::Output>{

	eprintln!("Running transaction with hash {:#x}", tx.hash());

	return client.transact(env_info, tx, trace::NoopTracer, informant);
}



