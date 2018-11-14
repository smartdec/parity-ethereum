## evmbin

Simplified EVM implementation for Parity.

### Usage

```
Simplified EVM implementation for Parity.
  Copyright 2018 SmartDec (RU) Ltd.

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
```
### Example of transaction
```json
[
	{
		"data": "",
		"gasLimit": "0x2dc6c0",
		"gasPrice": "0x01",
		"nonce": "0x01",
		"secretKey": "c591464b3483d8754e4a3738dd7a70eacc1f3db717a6f114ac2f409361b738d4",
		"to": "1000000000000000000000000000000000000000",
		"value": "0x00"
	}
]
```
If `to` is empty string transaction interpreters as contract creation.

## Parity Ethereum toolchain
_This project is **not** a part of the Parity Ethereum toolchain. But uses its infrastructure_

