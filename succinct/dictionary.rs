/// Rank and select operations
///
/// Bit indices start at 0.
use std::intrinsics::{ctpop64};

type Pos = int;
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
    fn select0(&self, n: Count) -> Pos;

    /// Given a sequence of bits, `select1(n)` is the 0-based position
    /// of the `n`th one.
    fn select1(&self, n: Count) -> Pos;
}

//impl BitSelect for u64 {}

/// Out of range bits taken to be 0
impl BitRank for u64 {
    fn rank1(&self, n: int) -> int {
        if n < 64 {
            let mask: u64 = (1 << (n as uint)) - 1;
            unsafe { ctpop64(mask & *self) as int }
        } else {
            unsafe { ctpop64(*self) as int }
        }
    }

    fn rank0(&self, n: int) -> int {
        if n < 64 {
            let mask = (1 << (n as uint)) - 1;
            unsafe { ctpop64(mask & *self) as int }
        } else {
            unsafe { n - ctpop64(*self) as int }
        }
    }
}
