use super::dictionary::{BitAccess};
use std::collections::Collection;

/// A very simple rank implementation written to test against
pub fn rank<T: BitAccess>(v: &T, bit: bool, n: int) -> int {
    let mut accum = 0;
    for i in range(0, n) {
        if v.get(i) == bit {
            accum += 1;
        }
    }
    accum
}

pub fn select<T: BitAccess + Collection>(v: &T, bit: bool, n: int) -> int {
    let mut n = n;
    for i in range(0, v.len()) {
        if v.get(i as int) == bit {
            if n == 0 {
                return i as int;
            }
            n -= 1;
        }
    }
    fail!("There aren't that many");
}
