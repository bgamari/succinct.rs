use super::num::integer::Integer;
use std::num::{zero, one, Int};
use super::dictionary::BitRank;

struct Counts {
    /// first level count (rank up to p)
    first: u64,
    /// second level counts (rank up to each broadword)
    second: u64,
}

/// Bitvector supporting efficient rank and select
struct Rank9 {
    /// length of bitvector in bits
    bits: int,
    /// the bitvector data
    buffer: Vec<u64>,
    /// the basic block counts
    counts: Vec<Counts>,
}

fn div_ceil<T: Integer>(a: T, b: T) -> T {
    if a % b != zero() {
        a / b + one()
    } else {
        a / b
    }
}

impl Rank9 {
    fn from_vec(mut v: Vec<u64>, length_in_bits: int) -> Rank9 {
        let n_blocks = div_ceil(length_in_bits, 64*8);

        if length_in_bits % (64*8) != 0 {
            let padding = 8*n_blocks as uint - v.len();
            v.grow(padding, &0);
        }

        // Compute counts
        let mut counts: Vec<Counts> = Vec::with_capacity(n_blocks as uint);
        let mut accum = Counts { first: 0, second: 0 };
        // accumulate number of ones in this block
        let mut block_accum: u64 = 0;
        // accumulate number of ones
        let mut rank_accum: u64 = 0;
        for i in range(0, 8*n_blocks) {
            let word = v[i as uint];
            let ones = word.count_ones();
            rank_accum += ones;
            block_accum += ones;
            if i % 8 == 7 {
                // push new block
                counts.push(accum);
                block_accum = 0;
                accum.first = rank_accum;
                accum.second = 0;
            } else {
                accum.second >>= 9;
                accum.second |= block_accum << (9*6);
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
        let (word, bit) = n.div_mod_floor(&64); // w == word
        let (block, block_word) = word.div_mod_floor(&8);
        let counts = &self.counts[block as uint];
        let t = block_word - 1;

        // compute second-level contribution
        let shift = (t + ((t >> 60) & 8)) * 9;
        let second = (counts.second >> (shift as uint)) & 0x1ff;

        // within-word contribution
        let masked = self.buffer[word as uint] & ((1 << (bit as uint)) - 1);

        (counts.first + second + masked.count_ones()) as int
    }
    fn rank0(&self, n: int) -> int {
        0
    }
}

#[cfg(test)]
mod test {
    use super::Rank9;
    use super::super::dictionary::{BitRank, BitSelect};

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
}
