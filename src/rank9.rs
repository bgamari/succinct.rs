//! Rank9 bitvector
//
// Rank9 supports efficient rank and select operations. This
// implementation uses `rank9`-proper for `rank` and an approximation
// of the `simple` algorithm described by Vigna for `select`.
//
// See Vigna 2014.

use std::cmp::{min, Ordering};
use std::num::Int;
use std::iter::range_step_inclusive;
use std::ops::Shr;
use super::dictionary::{Rank, BitRank, Select, Access};
use super::collection::Collection;

pub use rank9::build::Builder;

/// Counts for a basic block
#[derive(Copy)]
pub struct Counts {
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

    /// The number of matching bits in blocks up to but not including
    /// `block_idx`
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
            if n <= self.word_rank(bit, i + 1) {
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
    fn get(&self, n: uint) -> bool {
        let word = self.buffer[n / 64];
        (word >> (n % 64)) & 1 == 1
    }
}

impl Collection for Rank9 {
    fn len(&self) -> uint {
        self.bits as uint
    }
}

impl Rank9 {
    /// Search for the block that contains the `n`th matching bit
    fn select_block(&self, bit: bool, n: uint) -> uint {
        debug_assert!(n > 0);
        self.select_block_hlpr(bit, n, 0, self.counts.len())
        // todo LD: can we have default settings for method arguments?
    }


    fn select_block_hlpr(&self, bit:bool, n:uint, lower:uint, upper:uint) -> uint {
            let block_search: Result<uint,uint> =
                binary_search(|idx| self.counts[*idx].block_rank(bit, *idx).cmp(&(n as u64)),
                              lower, upper);
            let start_block = match block_search {
                Ok(block) => block,
                Err(i) => return i - 1,
            };

            // we found a block that is potentially surrounded by blocks
            // with the same block rank; we need to find the next matching
            // bit
            for block_idx in range_step_inclusive(start_block as int, 0, -1) {
                if self.counts[block_idx as uint].block_rank(bit, block_idx as uint) != n as u64 {
                    return block_idx as uint;
                }
            }
            self.counts.len() - 1
    }


    /// implements the recursion of select_all_blocks
    fn select_all_blocks_rec(&self, bit:bool, ns:Vec<uint>, lower:uint, upper:uint) -> Vec<uint> {
        let idx = ns.len()/2;
        let pos = self.select_block_hlpr(bit, ns[idx], lower, upper);

        let left_result: Vec<uint> = if idx > 0 {
                self.select_all_blocks_rec(bit, ns[0..idx].to_vec(), lower, pos)
            } else {vec!()};
        let right_result: Vec<uint> = if idx < ns.len()-1 {
                self.select_all_blocks_rec(bit, ns[idx+1..ns.len()].to_vec(), pos+1, upper)
            } else {vec!()};

        // leftResult ++ (pos) ++ rightResult
        let mut result = Vec::with_capacity(ns.len());
        result.push_all(left_result.as_slice());
        result.push(pos);
        result.push_all(right_result.as_slice());
        result
    }

    /// select_block for all n's at once.
    /// naive implementation of multiple selecs = O( ns.len() * log(bitv))
    /// this implementation = O(ns.len * log log(bitv) )
    /// = [ log(bitv) + 2 log(bitv/2) + 4 log(bitv/4) ...]
    fn select_all_blocks(&self, bit:bool, ns: Vec<uint>) -> Vec<uint> {
        self.select_all_blocks_rec(bit, ns, 0, self.counts.len())
    }

    pub fn from_vec<'a>(v: &'a Vec<u64>, length_in_bits: int) -> Rank9 {
        use super::build::Builder;
        let mut builder = build::CountsBuilder::with_capacity(v.len());
        for x in v.iter() {
            builder.push(*x);
        }
        return Rank9 {
            bits: length_in_bits,
            buffer: v.clone(), // TODO: no clone
            counts: builder.finish(),
        };
    }
}

impl Rank<bool> for Rank9 {
    fn rank(&self, el: bool, n: int) -> int {
        if el {self.rank1(n)} else {self.rank0(n)}
    }
}

impl BitRank for Rank9 {
    fn rank1(&self, n: int) -> int {
        assert!(n <= self.bits);
        let n = min(n, self.bits);
        let word = n / 64;
        let bit_idx = n % 64;
        let block = word / 8;
        let block_word = word % 8;
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

        (counts._block_rank + word_rank + masked.count_ones() as u64) as int
    }

    fn rank0(&self, n: int) -> int {
        n - self.rank1(n)
    }
}

/// Perform a binary search in the range `[lower, upper)`.
/// If the predicate returns `Equal`, `Ok` with the matching index
/// is returned. Otherwise, `Err` is returned with the index of a
/// valid insertion point.
fn binary_search<T: Shr<T> + Ord + Int + Clone, F>(cmp: F, lower: T, upper: T) -> Result<T,T>
    where F: Fn(&T) -> Ordering
{
    let mut base : T = lower.clone();
    let mut lim : T = upper.clone();

    while lim > lower {
        let ix = base + (lim >> 1u);
        match cmp(&ix) {
            Ordering::Equal => return Ok(ix),
            Ordering::Less => {
                base = ix + Int::one();
                lim = lim - Int::one();
            }
            Ordering::Greater => ()
        }
        lim = lim >> 1u;
    }
    return Err(base);
}

impl Select<bool> for Rank9 {
    fn select(&self, bit: bool, n: int) -> int {
        // uses `laura-select`
        debug_assert!(n >= 0);

        if n == 0 { return 0; }
        let block_idx = self.select_block(bit, n as uint);
        let counts = &self.counts[block_idx];
        let mut remaining = n - counts.block_rank(bit, block_idx) as int;
        let word_idx = counts.select_word(bit, remaining as uint);
        let word: u64 = self.buffer[word_idx + 8*block_idx];
        remaining -= counts.word_rank(bit, word_idx) as int;
        (block_idx as int)*64*8 + (word_idx as int) * 64 + word.select(bit, remaining)
    }
}

mod build {
    use std::num::Int;
    use super::super::build;
    use super::{Counts, Rank9};
    use utils::div_ceil;

    /// Build up the counts metadata for rank-9 from a stream of `u64`s
    pub struct CountsBuilder {
        /// length in broadwords
        length: uint,
        counts: Vec<Counts>,
        /// accumulate `Counts` for the current block
        accum: Counts,
        /// accumulate number of ones in the current block
        block_accum: u64,
        /// accumulate number of ones total
        rank_accum: u64,
    }

    impl CountsBuilder {
        /// Create a `CountsBuilder` with capacity for `cap` broadwords.
        pub fn with_capacity(cap: uint) -> CountsBuilder {
            let n_blocks = div_ceil(cap, 64*8);
            CountsBuilder {
                length: 0,
                counts: Vec::with_capacity(n_blocks as uint),
                accum: Counts { _block_rank: 0, word_ranks: 0 },
                block_accum: 0,
                rank_accum: 0,
            }
        }

        fn push_block(&mut self) {
            self.counts.push(self.accum);
            self.block_accum = 0;
            self.accum._block_rank = self.rank_accum;
            self.accum.word_ranks = 0;
        }
    }

    impl build::Builder<u64, Vec<Counts>> for CountsBuilder {
        fn push(&mut self, word: u64) {
            let ones = word.count_ones() as u64;
            self.rank_accum += ones;
            self.block_accum += ones;
            if self.length % 8 == 7 {
                self.push_block();
            } else {
                self.accum.word_ranks >>= 9;
                self.accum.word_ranks |= self.block_accum << (9*6);
            }
            self.length += 1;
        }

        fn finish(mut self) -> Vec<Counts> {
            // Finish up final partial block
            while self.length % 8 != 0 {
                self.push(0);
            }
            self.counts
        }
    }

    /// Build a rank-9 bitvector from broadwords
    pub struct WordBuilder {
        builder: CountsBuilder,
        buffer: Vec<u64>,
    }

    impl WordBuilder {
        /// Create a `WordBuilder` with capacity for `cap` broadwords
        pub fn with_capacity(cap: uint) -> WordBuilder {
            WordBuilder {
                builder: CountsBuilder::with_capacity(cap),
                buffer: Vec::with_capacity(cap),
            }
        }
    }

    impl build::Builder<u64, Rank9> for WordBuilder {
        fn push(&mut self, word: u64) {
            self.builder.push(word);
            self.buffer.push(word);
        }
        fn finish(self) -> Rank9 {
            Rank9 {
                bits: 64*self.builder.length as int,
                buffer: self.buffer,
                counts: self.builder.finish(),
            }
        }
    }

    /// Build a `Rank9` bitvector from bits
    pub struct Builder {
        builder: build::BitBuilder<WordBuilder>,
    }

    impl Builder {
        /// Build a rank-9 bitvector with capacity for `cap` bits
        pub fn with_capacity(cap: uint) -> Builder {
            let b: WordBuilder = WordBuilder::with_capacity(64*cap);
            Builder {
                builder: build::BitBuilder::new(b)
            }
        }
    }

    impl build::Builder<bool, Rank9> for Builder {
        fn push(&mut self, bit: bool) {
            self.builder.push(bit)
        }
        fn finish(self) -> Rank9 {
            match self.builder.finish() {
                (mut rank9, bits) => {
                    rank9.bits = bits as int;
                    rank9
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::num::Int;
    use quickcheck::TestResult;

    use super::Rank9;
    use super::super::dictionary::{BitRank, Select};
    use super::super::naive;

    #[test]
    fn test_rank0() {
        super::super::dictionary::test::test_rank0(&Rank9::from_vec);
    }

    #[test]
    fn test_rank1() {
        super::super::dictionary::test::test_rank1(&Rank9::from_vec);
    }

    #[test]
    fn test_select0() {
        super::super::dictionary::test::test_select0(&Rank9::from_vec);
    }

    #[test]
    fn test_select1() {
        super::super::dictionary::test::test_select1(&Rank9::from_vec);
    }

    #[quickcheck]
    fn rank_is_correct(bit: bool, v: Vec<u64>, n: uint) -> TestResult {
        let bits = v.len() * 64;
        if n > bits {
            return TestResult::discard()
        }
        let bv = Rank9::from_vec(&v, bits as int);
        let ans = if bit { bv.rank1(n as int) } else { bv.rank0(n as int) };
        TestResult::from_bool(ans == naive::rank(&bv, bit, n as int))
    }

    #[quickcheck]
    fn select_is_correct(bit: bool, v: Vec<u64>, n: uint) -> TestResult {
        use std::iter::AdditiveIterator;
        if (v.iter().map(|x| x.count_ones()).sum() as uint) < n {
            return TestResult::discard()
        }

        let bits = v.len() * 64;
        if v.is_empty() || n >= bits {
            return TestResult::discard()
        }
        let bv = Rank9::from_vec(&v, bits as int);
        match naive::select(&bv, bit, n as int) {
            None => TestResult::discard(),
            Some(ans) =>
                TestResult::from_bool(ans == bv.select(bit, n as int))
        }
    }

    #[test]
    fn test_binary_search2() {
        use super::binary_search;
        let xs: Vec<int> = vec!(0, 22, 41, 63);
        assert_eq!(binary_search(|i| xs[*i].cmp(&63), 0, xs.len()), Ok(3));
    }

    #[test]
    fn test_binary_search() {
        use super::binary_search;
        let xs: Vec<int> = vec!(0, 3, 5, 8);
        assert_eq!(binary_search(|i| xs[*i].cmp(&5), 0, xs.len()), Ok(2));
        assert_eq!(binary_search(|i| xs[*i].cmp(&4), 0, xs.len()), Err(2));
        assert_eq!(binary_search(|i| xs[*i].cmp(&0), 0, xs.len()), Ok(0));
        assert_eq!(binary_search(|i| xs[*i].cmp(&9), 0, xs.len()), Err(4));
    }

    #[quickcheck]
    fn binary_search_works(v: Vec<int>, s: int) -> TestResult {
        use std::iter::FromIterator;
        use super::binary_search;
        if v.len() < 2 {return TestResult::discard()}
        let xs: Vec<int> = FromIterator::from_iter(
            v.clone().into_iter()
                .scan(0, |acc, x| {*acc += x; Some(*acc+x)}));
        let res = match binary_search(|i| xs[*i].cmp(&s), 0, xs.len()) {
            Ok(i) =>
                xs[i] == s,
            Err(i) if i == 0 || i == v.len() =>
                true,
            Err(i) =>
                xs[i-1] < s && xs[i] >= s
        };
        TestResult::from_bool(res)
    }
}
