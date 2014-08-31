use std::intrinsics::{ctpop64};

pub trait BitRank {
    fn rank0(&self, n: int) -> int;
    fn rank1(&self, n: int) -> int;
}

pub trait BitSelect {
    fn select0(&self, n: int) -> int;
    fn select1(&self, n: int) -> int;
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
