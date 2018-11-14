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

//! Parity EVM interpreter binary.

#![warn(missing_docs)]

extern crate ethcore;
extern crate ethjson;
extern crate rustc_hex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate docopt;
extern crate ethcore_transaction as transaction;
extern crate parity_bytes as bytes;
extern crate ethereum_types;
extern crate vm;
extern crate evm;
extern crate panic_hook;
extern crate env_logger;
extern crate rlp;
extern crate serde_json;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[cfg(test)]
extern crate tempdir;

use std::sync::Arc;
use std::{fmt, fs};
use std::path::PathBuf;
use docopt::Docopt;
use rustc_hex::FromHex;
use ethereum_types::{U256, Address};
use bytes::Bytes;
use ethcore::{spec, json_tests};
use vm::{ActionParams, CallType};
use ethcore::client::{self, EvmTestClient, EvmTestError, TransactResult};
use ethjson::state::Transaction;

use std::io::{self, BufRead};

mod info;
mod display;

use info::Informant;

const USAGE: &'static str = r#"
Simplified EVM implementation for Parity.
  Copyright 2015-2018 Parity Technologies (UK) Ltd.

Usage:
    parity-patched-evm [options] [array of json transactions]
    parity-patched-evm [-h | --help]

State test options:
    --chain CHAIN      Run only tests from specific chain.

General options:
    --json             Display verbose results in JSON.
    --std-json         Display results in standardized JSON format.
    --chain CHAIN      Chain spec file path.
    -h, --help         Display this message and exit.
"#;

fn main() {
	panic_hook::set_abort();
	env_logger::init();

	let args: Args = Docopt::new(USAGE).and_then(|d| d.deserialize()).unwrap_or_else(|e| e.exit());

	let spec = match args.spec() {
		Ok(spec) => spec,
		Err(err) => die(err)
	};
	let genesis = spec.genesis_header();
	let mut env_info = evm::EnvInfo {
		number: genesis.number(),
		author: *genesis.author(),
		timestamp: genesis.timestamp(),
		difficulty: *genesis.difficulty(),
		gas_limit: U256::max_value(),
		last_hashes: Arc::new(Vec::new()),
		gas_used: 0.into(),
	};

	let mut test_client = match EvmTestClient::from_pod_state(&spec, spec.genesis_state().clone()) {
		Ok(test_client) => test_client,
		Err(err) => die(err)
	};

	let stdin = io::stdin();
	let transactions: Vec<serde_json::Value> = serde_json::from_reader(stdin).unwrap();
	for tx_value in transactions.into_iter() {

		let tx: ethjson::state::Transaction = serde_json::from_value(tx_value).unwrap();
		let tx = transaction::SignedTransaction::from(tx);

		if args.flag_json {
			run_tx(tx, display::json::Informant::default(), &mut env_info, &mut test_client);
		} else if args.flag_std_json {
			run_tx(tx, display::std_json::Informant::default(), &mut env_info, &mut test_client);
		} else {
			run_tx(tx, display::simple::Informant::default(), &mut env_info, &mut test_client);
		}
	}
}

fn run_tx<T: Informant>(tx: transaction::SignedTransaction, mut informant: T, env_info: &mut client::EnvInfo, client: &mut EvmTestClient) {


	let result = info::run_transaction(env_info, tx, informant, client);
	match result{
		TransactResult::Ok{gas_left, contract_address, ..} => {
			eprintln!("gas left: {}", gas_left);
			if contract_address.is_some() {
				eprintln!("contact on address: {:?}", contract_address.unwrap());
			}
		},
		TransactResult::Err{error: err, ..} => die(err)
	};
	//T::finish(result);
}

#[derive(Serialize, Deserialize)]
struct Args {
	arg_file: Option<PathBuf>,
	flag_chain: Option<String>,
	flag_json: bool,
	flag_std_json: bool,
}

impl Args {
		pub fn spec(&self) -> Result<spec::Spec, String> {
		Ok(match self.flag_chain {
			Some(ref filename) =>  {
				let file = fs::File::open(filename).map_err(|e| format!("{}", e))?;
				spec::Spec::load(&::std::env::temp_dir(), file)?
			},
			None => {
				ethcore::ethereum::new_foundation(&::std::env::temp_dir())
			},
		})
	}
}

fn arg<T>(v: Result<T, String>, param: &str) -> T {
	v.unwrap_or_else(|e| die(format!("Invalid {}: {}", param, e)))
}

fn to_string<T: fmt::Display>(msg: T) -> String {
	format!("{}", msg)
}

fn die<T: fmt::Display>(msg: T) -> ! {
	println!("{}", msg);
	::std::process::exit(-1)
}

#[cfg(test)]
mod tests {
	use docopt::Docopt;
	use super::{Args, USAGE};

	fn run<T: AsRef<str>>(args: &[T]) -> Args {
		Docopt::new(USAGE).and_then(|d| d.argv(args.into_iter()).deserialize()).unwrap()
	}

	#[test]
	fn should_parse_all_the_options() {
		let args = run(&[
			"parity-evm",
			"--json",
			"--std-json",
			"--gas", "1",
			"--gas-price", "2",
			"--from", "0000000000000000000000000000000000000003",
			"--to", "0000000000000000000000000000000000000004",
			"--code", "05",
			"--input", "06",
			"--chain", "./testfile",
		]);

		assert_eq!(args.flag_json, true);
		assert_eq!(args.flag_std_json, true);
		assert_eq!(args.gas(), Ok(1.into()));
		assert_eq!(args.gas_price(), Ok(2.into()));
		assert_eq!(args.from(), Ok(3.into()));
		assert_eq!(args.to(), Ok(4.into()));
		assert_eq!(args.code(), Ok(Some(vec![05])));
		assert_eq!(args.data(), Ok(Some(vec![06])));
		assert_eq!(args.flag_chain, Some("./testfile".to_owned()));
	}

	#[test]
	fn should_parse_state_test_command() {
		let args = run(&[
			"parity-evm",
			"state-test",
			"./file.json",
			"--chain", "homestead",
			"--only=add11",
			"--json",
			"--std-json"
		]);

		assert_eq!(args.cmd_state_test, true);
		assert!(args.arg_file.is_some());
		assert_eq!(args.flag_json, true);
		assert_eq!(args.flag_std_json, true);
		assert_eq!(args.flag_chain, Some("homestead".to_owned()));
		assert_eq!(args.flag_only, Some("add11".to_owned()));
	}
}
