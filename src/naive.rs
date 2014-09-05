//! Exceedingly naive implementations for dictionary operations

use super::dictionary::{Access};
use std::collections::Collection;
use std::option::{Option, Some, None};
use std::cmp::Eq;

/// A very simple rank implementation written to test against
pub fn rank<T: Eq, BitVec: Access<T>>(v: &BitVec, bit: T, n: int) -> int {
    let mut accum = 0;
    for i in range(0, n) {
        if v.get(i as uint) == bit {
            accum += 1;
        }
    }
    accum
}

pub fn select<T: Eq, BitVec: Access<T> + Collection>(v: &BitVec, bit: T, n: int) -> Option<int> {
    let mut n = n;
    if n == 0 {
        return Some(0);
    }
    for i in range(0, v.len()) {
        if v.get(i) == bit {
            n -= 1;
            if n == 0 {
                return Some(i as int + 1);
            }
        }
    }
    None
}
