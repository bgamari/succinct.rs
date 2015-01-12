//! Utilities

use std::num::{Int};

pub fn div_ceil<T: Int>(a: T, b: T) -> T {
    if a % b != Int::zero() {
        a / b + Int::one()
    } else {
        a / b
    }
}
