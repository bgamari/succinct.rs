//! A simple bit-vector

use super::dictionary::{Access, Rank, BitRank, Select};
use std::collections::Collection;
pub use bit_vector::build::Builder;

/// A simple bit vector
///
/// The first bit in the vector is the least-significant bit of the
/// first broadword
#[deriving(Show)]
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

impl Collection for BitVector {
    fn len(&self) -> uint {
        self.bits as uint
    }
}

impl Access<bool> for BitVector {
    fn get(&self, n: uint) -> bool {
        let word = self.buffer[n / 64];
        (word >> (n % 64)) & 1 == 1
    }
}

impl Rank<bool> for BitVector {
    fn rank(&self, el: bool, n: int) -> int {
        if el {self.rank1(n)} else {self.rank0(n)}
    }
}

impl BitRank for BitVector {
    fn rank0(&self, n: int) -> int {
        n - self.rank1(n)
    }

    fn rank1(&self, n: int) -> int {
        assert!(n < self.bits);
        let mut rank = 0;
        for i in self.buffer.iter().take(n as uint / 64) {
            rank += i.rank1(64);
        }

        if n < self.len() as int {
            rank += self.buffer[n as uint / 64].rank1(n % 64);
        }
        rank
    }
}

impl Select<bool> for BitVector {
    #[inline(always)]
    fn select(&self, bit: bool, n: int) -> int {
        debug_assert!(n >= 0);
        if n == 0 {
            return 0;
        }

        println!("{}",self);
        let mut cur: u64 = 0;
        let mut remain: int = n; // counting down from n
        let mut idx: int = 0;
        for word in self.buffer.iter() {
            cur = *word;
            let matches = if bit { word.count_ones() } else { word.count_zeros() } as int;
            if remain > matches {
                remain -= matches;
                idx += 64;
            } else {
                break
            }
        }
        idx + cur.select(bit, remain)
    }
}

mod build {
    use super::super::build;
    use super::super::utils::div_ceil;
    use super::BitVector;

    /// Build a `BitVector` from bits
    #[deriving(Show)]
    pub struct Builder {
        builder: build::BitBuilder<build::VecBuilder<u64>>,
    }

    impl Builder {
        /// Build a bitvector with capacity for `cap` bits
        pub fn with_capacity(cap: uint) -> Builder {
            let words = div_ceil(cap, 64);
            Builder {
                builder: build::BitBuilder::new(build::VecBuilder::with_capacity(words)),
            }
        }
    }

    impl build::Builder<bool, BitVector> for Builder {
        fn push(&mut self, bit: bool) {
            self.builder.push(bit)
        }
        fn finish(self) -> BitVector {
            match self.builder.finish() {
                (vec, bits) => BitVector { bits: bits as int, buffer: vec }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use quickcheck::TestResult;

    use super::BitVector;
    use super::super::dictionary::{BitRank, Select, Access};
    use super::super::naive;

    #[test]
    pub fn test_select0() {
        super::super::dictionary::test::test_select0(BitVector::from_vec)
    }

    #[test]
    pub fn test_select1() {
        super::super::dictionary::test::test_select1(BitVector::from_vec)
    }

    #[test]
    pub fn test_rank0() {
        super::super::dictionary::test::test_rank0(BitVector::from_vec)
    }

    #[test]
    pub fn test_rank1() {
        super::super::dictionary::test::test_rank1(BitVector::from_vec)
    }

    #[quickcheck]
    fn test_builder(bits: Vec<bool>) -> bool {
        use super::super::build::Builder;
        let b = super::Builder::with_capacity(8).from_iter(bits.clone().move_iter());
        for (i, bit) in bits.iter().enumerate() {
            if b.get(i) != *bit {
                return false;
            }
        }
        true
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

    #[quickcheck]
    fn rank_is_correct(bit: bool, v: Vec<u64>, n: uint) -> TestResult {
        let bits = v.len() * 64;
        if v.is_empty() || n >= bits {
            return TestResult::discard()
        }
        let bv = BitVector::from_vec(&v, bits as int);
        let ans = if bit { bv.rank1(n as int) } else { bv.rank0(n as int) };
        TestResult::from_bool(ans == naive::rank(&bv, bit, n as int))
    }

    #[quickcheck]
    fn select_is_correct(bit: bool, v: Vec<u64>, n: uint) -> TestResult {
        let bits = v.len() * 64;
        if v.is_empty() || n >= bits {
            return TestResult::discard()
        }
        let bv = BitVector::from_vec(&v, bits as int);
        match naive::select(&bv, bit, n as int) {
            None => TestResult::discard(),
            Some(ans) =>
                TestResult::from_bool(ans == bv.select(bit, n as int))
        }
    }
}
