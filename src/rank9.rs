/// rank9 bitvector
///
/// See Vigna 2014.

use super::num::integer::Integer;
use std::num::{zero, one, Int};
use super::dictionary::{BitRank, BitSelect, BitAccess};
use std::slice::{Found, NotFound};
use std::collections::Collection;

/// Counts for a basic block
struct Counts {
    /// first level count (rank up to p)
    block_rank: u64,
    /// second level counts (rank up to each broadword)
    word_ranks: u64,
}

impl Counts {
    /// The rank within the block up to the `i`th broadword
    fn word_rank(&self, i: uint) -> uint {
        debug_assert!(i < 8);
        ((self.word_ranks >> (9*i)) & 0x1ff) as uint
    }

    /// Search for the word that contains the `n`th one within this
    /// block
    fn select_word(&self, n: uint) -> uint {
        for i in range(0,7) {
            if self.word_rank(i) > n {
                return i;
            }
        }
        return 7;
    }
}

/// Bitvector supporting efficient rank and select
pub struct Rank9 {
    /// length of bitvector in bits
    bits: int,
    /// the bitvector data
    buffer: Vec<u64>,
    /// the basic block counts
    counts: Vec<Counts>,
}

impl BitAccess for Rank9 {
    fn get(&self, n: int) -> bool {
        let word = self.buffer[n as uint / 64];
        (word >> (n as uint % 64)) & 1 == 1
    }
}

impl Collection for Rank9 {
    fn len(&self) -> uint {
        self.bits as uint
    }
}

fn div_ceil<T: Integer>(a: T, b: T) -> T {
    if a % b != zero() {
        a / b + one()
    } else {
        a / b
    }
}

impl Rank9 {
    pub fn from_vec(mut v: Vec<u64>, length_in_bits: int) -> Rank9 {
        let n_blocks = div_ceil(length_in_bits, 64*8);

        // add padding to end as necessary
        if length_in_bits % (64*8) != 0 {
            let padding = 8*n_blocks as uint - v.len();
            v.grow(padding, &0);
        }

        // compute counts
        let mut counts: Vec<Counts> = Vec::with_capacity(n_blocks as uint);
        let mut accum = Counts { block_rank: 0, word_ranks: 0 };
        // accumulate number of ones in this block
        let mut block_accum: u64 = 0;
        // accumulate number of ones
        let mut rank_accum: u64 = 0;
        for (i, word) in v.iter().enumerate() {
            let ones = word.count_ones();
            rank_accum += ones;
            block_accum += ones;
            if i % 8 == 7 {
                // push new block
                counts.push(accum);
                block_accum = 0;
                accum.block_rank = rank_accum;
                accum.word_ranks = 0;
            } else {
                accum.word_ranks >>= 9;
                accum.word_ranks |= block_accum << (9*6);
            }
        }

        Rank9 {
            bits: length_in_bits,
            buffer: v,
            counts: counts,
        }
    }
}

impl BitRank for Rank9 {
    fn rank1(&self, n: int) -> int {
        let (word, bit_idx) = n.div_mod_floor(&64); // w == word
        let (block, block_word) = word.div_mod_floor(&8);
        let counts = &self.counts[block as uint];
        let t: int = block_word - 1;

        // compute second-level contribution
        // This is a bit tricky to avoid an unnecessary branch; functionally,
        //
        // ```
        // let word_rank = match block_word {
        //     0 => 0,
        //     _ => counts.word_rank(block_word - 1),
        // };
        // ```
        let shift = (t + ((t >> 60) & 8)) * 9;
        let word_rank = (counts.word_ranks >> (shift as uint)) & 0x1ff;

        // TODO: kill me
        match block_word {
            0 => {},
            _ => assert_eq!(counts.word_rank(block_word as uint - 1), word_rank as uint),
        };

        // within-word contribution
        let masked = self.buffer[word as uint] & ((1 << (bit_idx as uint)) - 1);

        (counts.block_rank + word_rank + masked.count_ones()) as int
    }
    fn rank0(&self, n: int) -> int {
        0
    }
}

impl BitSelect for Rank9 {
    fn select(&self, bit: bool, n: int) -> int {
        // uses `laura-select`
        debug_assert!(n >= 0);
        match self.counts.as_slice().binary_search(|x| x.block_rank.cmp(&(n as u64))) {
            Found(block) => {
                // We found a block beginning with exactly the right rank; We're done.
                (block as int)*64*8 + self.buffer[block*8].select(bit, 0)
            },
            NotFound(i) => {
                // We found the block succceeding the block containing
                // the desired rank.
                let block = i - 1;
                let counts = &self.counts[block];
                assert!(n > counts.block_rank as int);
                let word_idx = counts.select_word(n as uint - counts.block_rank as uint);

                let mut word: u64 = self.buffer[word_idx];
                let remain: int = if word_idx > 0 {
                    n - counts.block_rank as int - counts.word_rank(word_idx - 1) as int
                } else {
                    n - counts.block_rank as int
                };
                (block as int)*64*8 + (word_idx as int)*64 + word.select(bit, remain) as int
            }
        }
    }
}

#[cfg(test)]
mod test {
    use quickcheck::TestResult;

    use super::Rank9;
    use super::super::dictionary::{BitRank, BitSelect};
    use super::super::naive;

    #[test]
    fn test_rank1() {
        let v = vec!(0b0110, 0b1001, 0b1100);
        let bv = Rank9::from_vec(v, 64*3);
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

    #[test]
    fn test_select1() {
        let v = vec!(0b0110, 0b1001, 0b1100);
        let bv = Rank9::from_vec(v, 64*3);
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

    #[quickcheck]
    fn rank1_is_correct(v: Vec<u64>, n: uint) -> TestResult {
        let bits = v.len() * 64;
        if v.is_empty() || n >= bits {
            return TestResult::discard()
        }
        let bv = Rank9::from_vec(v, bits as int);
        TestResult::from_bool(bv.rank1(n as int) == naive::rank(&bv, true, n as int))
    }
}
