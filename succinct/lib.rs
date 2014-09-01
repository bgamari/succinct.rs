#![crate_name = "succinct"]
#![crate_type = "lib"]

extern crate num;
#[cfg(test)] extern crate quickcheck;

pub mod dictionary;
pub mod bit_vector;
pub mod rank9;
pub mod wavelet;
