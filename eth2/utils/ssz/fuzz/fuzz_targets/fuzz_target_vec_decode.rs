#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate ethereum_types;
extern crate ssz;

use ethereum_types::{Address, H256};
use ssz::{DecodeError, Decodable};

// Fuzz ssz_decode()
fuzz_target!(|data: &[u8]| {
    let _result: Result<(Vec<u8>, usize), DecodeError> = Decodable::ssz_decode(data, 0);
    /*
    let _result: Result<(Vec<u16>, usize), DecodeError> = Decodable::ssz_decode(data, 0);
    let _result: Result<(Vec<u32>, usize), DecodeError> = Decodable::ssz_decode(data, 0);
    let _result: Result<(Vec<u64>, usize), DecodeError> = Decodable::ssz_decode(data, 0);
    let _result: Result<(Vec<usize>, usize), DecodeError> = Decodable::ssz_decode(data, 0);
    let _result: Result<(Vec<Address>, usize), DecodeError> = Decodable::ssz_decode(data, 0);
    let _result: Result<(Vec<H256>, usize), DecodeError> = Decodable::ssz_decode(data, 0);
    let _result: Result<(Vec<bool>, usize), DecodeError> = Decodable::ssz_decode(data, 0);
    */
});
