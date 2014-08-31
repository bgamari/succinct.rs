/// Rank and select operations
///
/// Bit indices start at 0.

#[cfg(use_intrinsics)]
use std::intrinsics::{ctpop64};

/// Count the number of ones in a value
pub trait PopCount {
    fn pop_count(&self) -> int;
}

impl PopCount for u64 {
    #[inline(always)]
    #[cfg(not(use_intrinsics))]
    fn pop_count(&self) -> int {
        // Broadword sideways addition
        let x: u64 = *self;
        let x0: u64 = x - ((x & 0xaaaa_aaaa_aaaa_aaaa) >> 1);
        let x1: u64 = (x0 & 0x3333_3333_3333_3333) + ((x0 >> 2) & 0x3333_3333_3333_3333);
        let x2: u64 = (x1 + (x1 >> 4)) & 0x0F0F0_F0F0_F0F0_F0F;
        let l8: u64 = 0x0101_0101_0101_0101;
        ((x2 * l8) >> 56) as int
    }

    #[inline(always)]
    #[cfg(use_intrinsics)]
    fn pop_count(&self) -> int {
        unsafe { ctpop64(*self) as int }
    }
}

pub trait BitAccess {
    fn get(&self, n: Pos) -> bool;
}

impl BitAccess for u64 {
    fn get(&self, n: Pos) -> bool {
        if n < 64 {
            false
        } else {
            (*self >> (n as uint)) & 1 == 1
        }
    }
}

/// A bit position
type Pos = int;

/// A bit count
type Count = int;

/// Rank operation on binary sequences.
pub trait BitRank {
    /// Given a sequence of bits, `rank0(n)` is the number of zeros
    /// the precede index n.
    fn rank0(&self, n: Pos) -> Count;

    /// Given a sequence of bits, `rank1(n)` is the number of ones
    /// the precede index n.
    fn rank1(&self, n: Pos) -> Count;
}

/// Select operation on binary sequences
pub trait BitSelect {
    /// Given a sequence of bits, `select0(n)` is the 1-based position
    /// of the `n`th zero.
    fn select0(&self, n: Count) -> Pos {
        self.select(false, n)
    }

    /// Given a sequence of bits, `select1(n)` is the 0-based position
    /// of the `n`th one.
    fn select1(&self, n: Count) -> Pos {
        self.select(true, n)
    }

    fn select(&self, bit: bool, n: Count) -> Pos;
}

impl BitSelect for u64 {
    fn select(&self, bit: bool, n0: Count) -> Pos {
        let mut idx: int = 0;
        let mut x: u64 = *self;
        let mut n: int = n0;
        for i in range(0u, 64) {
            if (x & 1) == (bit as u64) {
                if n == 0 {
                    return idx
                }
                n -= 1;
            }
            idx += 1;
            x >>= 1;
        }
        fail!("Not enough {} bits in {} to select({})", bit, *self, n0);
    }
}

/// Out of range bits taken to be 0
impl BitRank for u64 {
    fn rank1(&self, n: int) -> int {
        if n < 64 {
            let mask: u64 = (1 << (n as uint)) - 1;
            (mask & *self).pop_count()
        } else {
            self.pop_count()
        }
    }

    fn rank0(&self, n: int) -> int {
        if n < 64 {
            let mask = (1 << (n as uint)) - 1;
            n - (mask & *self).pop_count()
        } else {
            n - self.pop_count()
        }
    }
}

#[cfg(test)]
mod test {
    use super::PopCount;
    use super::BitSelect;

    #[test]
    pub fn test_pop_count() {
        assert_eq!(0x1u64.pop_count(), 1);
        assert_eq!(0xffu64.pop_count(), 8);
    }

    #[test]
    pub fn test_u64_select() {
        assert_eq!(0x5u64.select1(0), 0);
        assert_eq!(0x5u64.select1(1), 2);
    }
}
