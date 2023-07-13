script;

use std::context::balance_of;
use std::token::*;
use test_fuel_coin_abi::*;

struct Opts {
    gas: u64,
    coins: u64,
    id: ContractId,
}

fn main() -> bool {
    let default_gas = 1_000_000_000_000;

    // the deployed fuel_coin Contract_Id:
    let fuelcoin_id = ContractId::from(0x4667ae5f1acd06f545f5d6a795fefa1df778d36531757b228c7e4b18656020b9);

    // contract ID for sway/test/src/e2e_vm_tests/test_programs/should_pass/test_contracts/balance_test_contract/
    let balance_test_id = ContractId::from(0x46d24c009cb468ef822ff7570eabc8d6636ac2a651d9743754803bbf84f36f85);

    // todo: use correct type ContractId
    let fuel_coin = abi(TestFuelCoin, fuelcoin_id.into());

    // Get the initial balances which can be non-zero
    // since we can't be sure if the contracts are fresh
    let fuelcoin_initial_balance = balance_of(fuelcoin_id, fuelcoin_id);
    let balance_test_initial_balance = balance_of(fuelcoin_id, balance_test_id);

    fuel_coin.mint {
        gas: default_gas
    }
    (11);

    // check that the mint was successful
    let fuelcoin_balance = balance_of(fuelcoin_id, fuelcoin_id) - fuelcoin_initial_balance;
    assert(fuelcoin_balance == 11);

    fuel_coin.burn {
        gas: default_gas
    }
    (7);

    // check that the burn was successful
    let fuelcoin_balance = balance_of(fuelcoin_id, fuelcoin_id) - fuelcoin_initial_balance;
    assert(fuelcoin_balance == 4);

    // force transfer coins
    fuel_coin.force_transfer {
        gas: default_gas
    }
    (3, fuelcoin_id, balance_test_id);

    // check that the transfer was successful
    let fuelcoin_balance = balance_of(fuelcoin_id, fuelcoin_id) - fuelcoin_initial_balance;
    let balance_test_contract_balance = balance_of(fuelcoin_id, balance_test_id) - balance_test_initial_balance;
    assert(fuelcoin_balance == 1);
    assert(balance_test_contract_balance == 3);

    true
}
