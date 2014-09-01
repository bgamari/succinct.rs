/// rank9 bitvector
///
/// See Vigna 2014.

use super::num::integer::Integer;
use std::num::{zero, One, one, Int};
use super::dictionary::{BitRank, Select, Access};
use std::collections::Collection;

use std::fmt::Show;

/// Counts for a basic block
struct Counts {
    /// first level count (rank up to p)
    _block_rank: u64,
    /// second level counts (rank up to each broadword)
    word_ranks: u64,
}

impl Counts {
    /// The rank within the block up to but not including the `i`th broadword
    fn word_rank(&self, bit:bool, i: uint) -> uint {
        debug_assert!(i < 8);
        match i {
            0 => 0,
            _ => {
                let ones = ((self.word_ranks >> (9*(i-1))) & 0x1ff) as uint;
                if bit {
                    ones
                } else {
                    i*64 - ones
                }
            }
        }
    }

    /// The number of matching bits in blocks up to but not including `block_idx`
    fn block_rank(&self, bit: bool, block_idx: uint) -> u64 {
        match bit {
            true => self._block_rank,
            false => 64*8*(block_idx as u64) - self._block_rank,
        }
    }

    /// Search for the word that contains the `n`th matching bit
    /// within this block
    fn select_word(&self, bit: bool, n: uint) -> uint {
        for i in range(0,7) {
            if n < self.word_rank(bit, i + 1) {
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
    /// Search for the block that contains the `n`th matching bit
    fn select_block(&self, bit: bool, n: uint) -> uint {
        let block_search =
            binary_search(|idx| self.counts[*idx].block_rank(bit, *idx).cmp(&(n as u64)),
                          0, self.counts.len());
        let start_block = match block_search {
            Found(block) => block,
            NotFound(i) => return i - 1,
        };

        // we found a block that is potentially surrounded by blocks
        // with the block rank; we need to find the next matching
        // bit
        for block_idx in range(start_block, self.counts.len()) {
            if self.counts[block_idx as uint].block_rank(bit, block_idx) != n as u64 {
                return block_idx - 1;
            }
        }
        self.counts.len() - 1
    }

    pub fn from_vec<'a>(v: &'a Vec<u64>, length_in_bits: int) -> Rank9 {
        let n_blocks = div_ceil(length_in_bits, 64*8);
        let mut v = v.clone(); // FIXME
        assert!(length_in_bits > 0);

        // add padding to end as necessary
        if length_in_bits % (64*8) != 0 {
            let padding = 8*n_blocks as uint - v.len();
            v.grow(padding, &0);
        }

        // compute counts
        let mut counts: Vec<Counts> = Vec::with_capacity(n_blocks as uint);
        let mut accum = Counts { _block_rank: 0, word_ranks: 0 };
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
                accum._block_rank = rank_accum;
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
            _ => assert_eq!(counts.word_rank(true, block_word as uint), word_rank as uint),
        };

        // within-word contribution
        let masked = self.buffer[word as uint] & ((1 << (bit_idx as uint)) - 1);

        (counts._block_rank + word_rank + masked.count_ones()) as int
    }

    fn rank0(&self, n: int) -> int {
        n - self.rank1(n)
    }
}

#[deriving(Show, PartialEq, Eq)]
enum BinarySearchResult<T> {
    Found(T),
    NotFound(T),
}

fn binary_search<T: Num + Shr<uint,T> + Ord + One + Clone + Show>(
        cmp: |&T| -> Ordering, lower: T, upper: T)
        -> BinarySearchResult<T> {
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
        let bit = *bit;
        // uses `laura-select`
        debug_assert!(n >= 0);

        let block_idx = self.select_block(bit, n as uint);
        let counts = &self.counts[block_idx];
        let mut remaining = n - counts.block_rank(bit, block_idx) as int;
        let word_idx = counts.select_word(bit, remaining as uint);
        let word: u64 = self.buffer[word_idx];
        remaining -= counts.word_rank(bit, word_idx) as int;
        (block_idx as int)*64*8 + (word_idx as int) * 64 + word.select(&bit, remaining)
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

    #[test]
    fn test_binary_search() {
        use super::{binary_search, Found, NotFound};
        let xs: Vec<int> = vec!(0, 3, 5, 8);
        assert_eq!(binary_search(|i| xs[*i].cmp(&5), 0, xs.len()), Found(2));
        assert_eq!(binary_search(|i| xs[*i].cmp(&4), 0, xs.len()), NotFound(2));
        assert_eq!(binary_search(|i| xs[*i].cmp(&0), 0, xs.len()), Found(0));
        assert_eq!(binary_search(|i| xs[*i].cmp(&9), 0, xs.len()), NotFound(4));
    }

    #[quickcheck]
    fn binary_search_works(v: Vec<int>, s: int) -> TestResult {
        use std::iter::FromIterator;
        use super::{binary_search, Found, NotFound};
        if v.len() < 2 {return TestResult::discard()}
        let xs: Vec<int> = FromIterator::from_iter(
            v.move_iter().scan(0, |acc, x| {*acc += x; Some(*acc+x)}));
        let res = match binary_search(|i| xs[*i].cmp(&s), 0, xs.len()) {
            Found(i) => xs[i] == s,
            NotFound(i) => xs[i-1] < s && xs[i] > s
        };
        TestResult::from_bool(res)
    }
}
