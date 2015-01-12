#![crate_name = "succinct"]
#![crate_type = "lib"]
#![feature(int_uint)]

extern crate core;
extern crate alloc;
#[cfg(test)] extern crate quickcheck;
#[cfg(test)] #[phase(plugin)] extern crate quickcheck_macros;

pub mod collection;
pub mod dictionary;
pub mod bit_vector;
pub mod rank9;
pub mod naive;
pub mod bits;
pub mod utils;
pub mod tree;
pub mod build;
pub mod wavelet;
