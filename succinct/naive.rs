use super::dictionary::{BitAccess};

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
