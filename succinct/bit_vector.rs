use super::dictionary::{BitAccess, PopCount};
use super::dictionary as dict;

/// A bit vector
///
/// The first bit in the vector is the least-significant bit of the
/// first broadword
pub struct BitVector {
    /// length in bits
    bits: int,
    /// the bits
    buffer: Vec<u64>
}

impl BitVector {
    pub fn zero(length_in_bits: int) -> BitVector {
        let len = if length_in_bits % 64 == 0 {
            length_in_bits / 64
        } else {
            length_in_bits / 64 + 1
        };
        BitVector {
            bits: length_in_bits,
            buffer: Vec::with_capacity(len as uint),
        }
    }

    pub fn from_vec(vec: &Vec<u64>, length_in_bits: int) -> BitVector {
        BitVector {
            bits: length_in_bits,
            buffer: vec.clone()
        }
    }
}

impl BitAccess for BitVector {
    fn get(&self, n: int) -> bool {
        let word = self.buffer[n as uint / 64];
        (word >> (n as uint % 64)) & 1 == 1
    }
}

impl dict::BitRank for BitVector {
    fn rank0(&self, n: int) -> int {
        n - self.rank1(n)
    }

    fn rank1(&self, n: int) -> int {
        let mut rank = 0;
        assert!(n < self.bits);
        for i in self.buffer.iter().take(n as uint / 64) {
            rank += i.rank1(64);
        }

        rank += self.buffer[n as uint / 64].rank1(n % 64);
        rank
    }
}

/*
impl dict::BitSelect for BitVector {
    fn select1(&self, n: int) -> int {
        let mut n: int = n;
        let mut it = self.buffer.iter();
        let mut cur: u64 = 0;
        while n > 0 {
            cur = match it.next() {
                Some(n) => *n,
                None    => fail!();
            };
            n -= cur.pop_count();
        }

        let count = cur.pop_count();
        if count < n {
            fail!();
        } else {
            // TODO
        }
    }
}
*/

impl dict::BitSelect for BitVector {
    //#[inline(always)]
    fn select(&self, bit: bool, n: int) -> int {
        let mut cur: u64 = 0;
        let mut remain: int = n+1; // counting down from n+1
        let mut idx: int = 0;
        for i in self.buffer.iter() {
            cur = *i;
            let ones = i.pop_count();
            let matches = if bit { ones } else { 64 - ones };
            if remain - matches > 0 {
                remain -= matches;
                idx += 64;
            } else {
                break
            }
        }
        idx + cur.select(bit, remain - 1)
    }
}

#[cfg(test)]
mod test {
    use super::BitVector;
    use super::super::dictionary::{BitRank, BitSelect, BitAccess};

    #[test]
    pub fn test_select0() {
        let v = vec!(0b0110, 0b1001, 0b1100);
        let bv = BitVector::from_vec(&v, 64*3);
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
            let a = bv.select0(rank);
            if a != select {
                fail!("select0({}) failed: expected {}, saw {}", rank, select, a);
            }
        }
    }

    #[test]
    pub fn test_select1() {
        let v = vec!(0b0110, 0b1001, 0b1100);
        let bv = BitVector::from_vec(&v, 64*3);
        let select1: Vec<(int,int)> = vec!(
            (0, (1+0*64)), // rank is non exclusive rank of zero is always 0
            (1, (2+0*64)),
            (2, (0+1*64)),
            (3, (3+1*64)),
            (4, (2+2*64)),
            (5, (3+2*64)),
        );
        for &(rank, select) in select1.iter() {
            let a = bv.select1(rank);
            if a != select {
                fail!("select1({}) failed: expected {}, saw {}", rank, select, a);
            }
        }
    }

    #[test]
    pub fn test_rank0() {
        let v = vec!(0b0110, 0b1001, 0b1100);
        let bv = BitVector::from_vec(&v, 64*3);
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

    #[test]
    pub fn test_get() {
        let v = vec!(0b0110, 0b1001, 0b1100);
        let bv = BitVector::from_vec(&v, 64*3);
        assert_eq!(bv.get(0),  false);
        assert_eq!(bv.get(1),  true);
        assert_eq!(bv.get(2),  true);
        assert_eq!(bv.get(3),  false);
        assert_eq!(bv.get(64), true);
    }

    #[test]
    pub fn test_rank1() {
        let v = vec!(0b0110, 0b1001, 0b1100);
        let bv = BitVector::from_vec(&v, 64*3);
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
