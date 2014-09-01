/// Rank and select operations
///
/// Bit indices start at 0.

pub trait Access<T> {
    /// Retrieve the `n`th bit
    fn get(&self, n: Pos) -> T;
}

impl Access<bool> for u64 {
    fn get(&self, n: Pos) -> bool {
        if n < 64 {
            false
        } else {
            (*self >> (n as uint)) & 1 == 1
        }
    }
}

/// A bit position
pub type Pos = int;

/// A bit count
pub type Count = int;

/// Rank operation
pub trait Rank<T> {
    fn rank(&self, el: &T, n: Pos) -> Count;
}

impl<T: BitRank> Rank<bool> for T {
    fn rank(&self, el: &bool, n: Pos) -> Count {
        if *el {
            self.rank1(n)
        } else {
            self.rank0(n)
        }
    }
}

/// Select operation
pub trait Select<T> {
    /// Given a sequence, `select(n)` is the 0-based position
    /// of the `n`th zero.
    fn select(&self, el: &T, n: Count) -> Pos;
}

/// Rank operation on binary sequences.
pub trait BitRank {
    /// Given a sequence of bits, `rank0(n)` is the number of zeros
    /// the precede index n.
    fn rank0(&self, n: Pos) -> Count;

    /// Given a sequence of bits, `rank1(n)` is the number of ones
    /// the precede index n.
    fn rank1(&self, n: Pos) -> Count;
}

impl Select<bool> for u64 {
    fn select(&self, bit: &bool, n0: Count) -> Pos {
        let bit: bool = *bit;
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

/*
fn pop_count(x: u64) -> int {
    // Broadword sideways addition
    let x0: u64 = x - ((x & 0xaaaa_aaaa_aaaa_aaaa) >> 1);
    let x1: u64 = (x0 & 0x3333_3333_3333_3333) + ((x0 >> 2) & 0x3333_3333_3333_3333);
    let x2: u64 = (x1 + (x1 >> 4)) & 0x0F0F0_F0F0_F0F0_F0F;
    let l8: u64 = 0x0101_0101_0101_0101;
    ((x2 * l8) >> 56) as int
}

/// Find the index of the `i`th one in `x`
/// Based on Algorithm 2 from Vigna 2014
fn bit_search(i: uint, x: u64) -> uint {
    fn lt8(x: u64, y: u64) -> u64 {
        let h8 = 0x8080808080808080;
        (((x | h8) - (y & !h8)) ^ x ^ !y) & h8
    }
    fn gt8(x: u64, y: u64) -> u64 {}

    let l8: u64 = 0x0101_0101_0101_0101;
    let s0: u64 = x - ((x & 0xaaaa_aaaa_aaaa_aaaa) >> 1);
    let s1: u64 = (x0 & 0x3333_3333_3333_3333) + ((x0 >> 2) & 0x3333_3333_3333_3333);
    let s2: u64 = (x1 + (x1 >> 4)) & 0x0F0F0_F0F0_F0F0_F0F;
    let s3: u64 = x2 * l8;
    let b = (((lt8(s, r*l8) >> 7) * l8) >> 53) & !7;
    let l = r - (((s << 8) >> b) & 0xff);
    let s4: u64 = ((((x >> b) & 0xff) * l8 & gt8(0x8040201008040201, 0)) >> 7) * l8;
    let res = b + (((lt8(s, l*l8) >> 7) * l8) >> 56);
    res as uint
}
*/

/// Out of range bits taken to be 0
impl BitRank for u64 {
    fn rank1(&self, n: int) -> int {
        if n < 64 {
            let mask: u64 = (1 << (n as uint)) - 1;
            (mask & *self).count_ones() as int
        } else {
            self.count_ones() as int
        }
    }

    fn rank0(&self, n: int) -> int {
        if n < 64 {
            let mask = (1 << (n as uint)) - 1;
            (mask | *self).count_zeros() as int
        } else {
            self.count_zeros() as int
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::{BitRank, Select};

    #[test]
    pub fn test_u64_select() {
        assert_eq!(0x5u64.select(&true, 0), 0);
        assert_eq!(0x5u64.select(&true, 1), 2);
    }

    pub fn test_select0<T: Select<bool>>(from_vec: |&Vec<u64>, int| -> T) {
        let v = vec!(0b0110, 0b1001, 0b1100);
        let bv = from_vec(&v, 64*3);
        let select0: Vec<(int, int)> = vec!(
            (0,   0+0*64),
            (1,   3+0*64),
            (2,   4+0*64),

            (62,  1+1*64),
            (63,  2+1*64),
            (64,  4+1*64),
            (65,  5+1*64),

            (124, 0+2*64),
            (125, 1+2*64),
            (126, 4+2*64),
            (127, 5+2*64),
        );
        for &(rank, select) in select0.iter() {
            let a = bv.select(&false, rank);
            if a != select {
                fail!("select0({}) failed: expected {}, saw {}", rank, select, a);
            }
        }
    }

    pub fn test_select1<T: Select<bool>>(from_vec: |&Vec<u64>, int| -> T) {
        let v = vec!(0b0110, 0b1001, 0b1100);
        let bv = from_vec(&v, 64*3);
        let select1: Vec<(int,int)> = vec!(
            (0, (1+0*64)), // rank is non exclusive rank of zero is always 0
            (1, (2+0*64)),
            (2, (0+1*64)),
            (3, (3+1*64)),
            (4, (2+2*64)),
            (5, (3+2*64)),
        );
        for &(rank, select) in select1.iter() {
            let a = bv.select(&true, rank);
            if a != select {
                fail!("select1({}) failed: expected {}, saw {}", rank, select, a);
            }
        }
    }

    pub fn test_rank0<T: BitRank>(from_vec: |&Vec<u64>, int| -> T) {
        let v = vec!(0b0110, 0b1001, 0b1100);
        let bv = from_vec(&v, 64*3);
        let rank0: Vec<(int, int)> = vec!(
            ((0+0*64), 0), // rank is non exclusive rank of zero is always 0
            ((1+0*64), 1),
            ((2+0*64), 1),
            ((3+0*64), 1),
            ((4+0*64), 2),

            ((0+1*64), 62), // second broadword
            ((1+1*64), 62),
            ((2+1*64), 63),
            ((3+1*64), 64),
            ((4+1*64), 64),

            ((0+2*64), 124),
            ((1+2*64), 125),
            ((2+2*64), 126),
            ((3+2*64), 126),
            ((4+2*64), 126),
        );
        for &(select, rank) in rank0.iter() {
            let a = bv.rank0(select);
            if a != rank {
                fail!("rank0({}) failed: expected {}, saw {}", select, rank, a);
            }
        }
    }

    pub fn test_rank1<T: BitRank>(from_vec: |&Vec<u64>, int| -> T) {
        let v = vec!(0b0110, 0b1001, 0b1100);
        let bv = from_vec(&v.clone(), 64*3);
        let rank1: Vec<(int, int)> = vec!(
            ((0+0*64), 0), // rank is non exclusive rank of zero is always 0
            ((1+0*64), 0),
            ((2+0*64), 1),
            ((3+0*64), 2),
            ((4+0*64), 2),

            ((0+1*64), 2), // second broadword
            ((1+1*64), 3),
            ((2+1*64), 3),
            ((3+1*64), 3),
            ((4+1*64), 4),

            ((0+2*64), 4),
            ((1+2*64), 4),
            ((2+2*64), 4),
            ((3+2*64), 5),
            ((4+2*64), 6),
        );

        for &(select, rank) in rank1.iter() {
            let a = bv.rank1(select);
            if a != rank {
                fail!("rank1({}) failed: expected {}, saw {}", select, rank, a);
            }
        }
    }
}
