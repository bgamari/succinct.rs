/// rank9 bitvector
///
/// See Vigna 2014.

use super::num::integer::Integer;
use std::num::{zero, One, one, Int};
use super::dictionary::{BitRank, Select, Access};
use std::collections::Collection;

/// Counts for a basic block
struct Counts {
    /// first level count (rank up to p)
    block_rank: u64,
    /// second level counts (rank up to each broadword)
    word_ranks: u64,
}

impl Counts {
    /// The rank within the block up to (and inclusive of) the `i`th broadword
    fn word_rank(&self, bit:bool, i: uint) -> uint {
        debug_assert!(i < 8);
        let ones = ((self.word_ranks >> (9*i)) & 0x1ff) as uint;
        if bit {
            ones
        } else {
            (i+1)*64 - ones
        }
    }

    fn block_rank0(&self, block_idx: uint) -> u64 {
        64*8*(block_idx as u64) - self.block_rank
    }

    /// Search for the word that contains the `n`th one within this
    /// block
    fn select_word(&self, n: uint, bit:bool) -> uint {
        for i in range(0,7) {
            if self.word_rank(bit, i) > n {
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

impl Access<bool> for Rank9 {
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
    pub fn from_vec<'a>(v: &'a Vec<u64>, length_in_bits: int) -> Rank9 {
        let n_blocks = div_ceil(length_in_bits, 64*8);
        let mut v = v.clone(); // FIXME

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
        assert!(n < self.bits);
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
            _ => assert_eq!(counts.word_rank(true, block_word as uint - 1), word_rank as uint),
        };

        // within-word contribution
        let masked = self.buffer[word as uint] & ((1 << (bit_idx as uint)) - 1);

        (counts.block_rank + word_rank + masked.count_ones()) as int
    }

    fn rank0(&self, n: int) -> int {
        n - self.rank1(n)
    }
}

enum BinarySearchResult<T> {
    Found(T),
    NotFound(T),
}

fn binary_search_with_idx<T: Num + Shr<uint,T> + Ord + One + Clone>(cmp: |&T| -> Ordering, lower: T, upper: T) -> BinarySearchResult<T> {
    let mut base : T = lower.clone();
    let mut lim : T = upper.clone();

    while lim > lower {
        let ix = base + (lim >> 1u);
        match cmp(&ix) {
            Equal => return Found(ix),
            Less => {
                base = ix + one();
                lim = lim - one();
            }
            Greater => ()
        }
        lim = lim >> 1u;
    }
    return NotFound(base);
}

impl Select<bool> for Rank9 {
    fn select(&self, bit: &bool, n: int) -> int {
        println!("select{}({})", bit, n)
        let bit = *bit;
        // uses `laura-select`
        debug_assert!(n >= 0);
        let block_search = if bit {
                binary_search_with_idx(|idx| self.counts[*idx].block_rank.cmp(&(n as u64)), 0, self.counts.len())
                //self.counts.as_slice().binary_search(|x| x.block_rank.cmp(&(n as u64)))
            } else {
                binary_search_with_idx(|idx| self.counts[*idx].block_rank0(*idx).cmp(&(n as u64)), 0, self.counts.len())
            };
        match block_search {
            Found(block) => {
                // We found a block beginning with exactly the right rank; We're done.
                println!("found block {}", block);
                (block as int)*64*8 + self.buffer[block*8].select(&bit, 0)
            },
            NotFound(i) => {
                // We found the block succceeding the block containing
                // the desired rank.
                let block = i - 1;
                let counts = &self.counts[block];
                assert!(n > counts.block_rank as int);
                let word_idx = counts.select_word(n as uint - counts.block_rank as uint, bit);

                let word: u64 = self.buffer[word_idx];
                let remain: int = if word_idx > 0 {
                    n - counts.block_rank as int - counts.word_rank(bit, word_idx - 1) as int
                } else {
                    n - counts.block_rank as int
                };
                println!("NotFound({}) remain={} block:{}/{} word:{}/{} bit:{}  bitvector:{:t}", i, remain, block, (block as int)*64*8, word_idx, (word_idx as int)*64, word.select(&bit, remain) as int, word);
                (block as int)*64*8 + (word_idx as int)*64 + word.select(&bit, remain) as int
            }
        }
    }
}

#[cfg(test)]
mod test {
    use quickcheck::TestResult;

    use super::Rank9;
    use super::super::dictionary::{BitRank, Select};
    use super::super::naive;

    #[test]
    fn test_rank0() {
        super::super::dictionary::test::test_rank0(Rank9::from_vec);
    }

    #[test]
    fn test_rank1() {
        super::super::dictionary::test::test_rank1(Rank9::from_vec);
    }

    #[test]
    fn test_select0() {
        super::super::dictionary::test::test_select0(Rank9::from_vec);
    }

    #[test]
    fn test_select1() {
        super::super::dictionary::test::test_select1(Rank9::from_vec);
    }

    #[quickcheck]
    fn rank_is_correct(bit: bool, v: Vec<u64>, n: uint) -> TestResult {
        let bits = v.len() * 64;
        if v.is_empty() || n >= bits {
            return TestResult::discard()
        }
        let bv = Rank9::from_vec(&v, bits as int);
        let ans = if bit { bv.rank1(n as int) } else { bv.rank0(n as int) };
        TestResult::from_bool(ans == naive::rank(&bv, bit, n as int))
    }

    #[quickcheck]
    fn select_is_correct(bit: bool, v: Vec<u64>, n: uint) -> TestResult {
        let bits = v.len() * 64;
        if v.is_empty() || n >= bits {
            return TestResult::discard()
        }
        let bv = Rank9::from_vec(&v, bits as int);
        match naive::select(&bv, bit, n as int) {
            None => TestResult::discard(),
            Some(ans) =>
                TestResult::from_bool(ans == bv.select(&bit, n as int))
        }
    }
}
