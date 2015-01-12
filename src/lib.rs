#![crate_name = "succinct"]
#![crate_type = "lib"]
#![feature(box_syntax, int_uint)]
#![allow(unstable)]

#[cfg(test)] extern crate quickcheck;
#[cfg(test)] #[macro_use] extern crate quickcheck_macros;

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
