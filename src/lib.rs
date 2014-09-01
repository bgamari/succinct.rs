#![crate_name = "succinct"]
#![crate_type = "lib"]

#![feature(phase)]

extern crate num;
#[cfg(test)] extern crate quickcheck;
#[cfg(test)] #[phase(plugin)] extern crate quickcheck_macros;

pub mod dictionary;
pub mod bit_vector;
pub mod rank9;
pub mod wavelet;
pub mod naive;
