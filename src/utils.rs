//! Utilities

use std::num::{zero, one};
use num::integer::Integer;

pub fn div_ceil<T: Integer>(a: T, b: T) -> T {
    if a % b != zero() {
        a / b + one()
    } else {
        a / b
    }
}

