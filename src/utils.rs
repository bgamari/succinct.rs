//! Utilities

use std::num::{Int, zero, one};

pub fn div_ceil<T: Int>(a: T, b: T) -> T {
    if a % b != zero() {
        a / b + one()
    } else {
        a / b
    }
}
