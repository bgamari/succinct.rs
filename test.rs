use succinct::bit_vector::BitVector;
use succinct::dictionary::{BitRank, BitSelect};
mod succinct;

fn main() {
    let v = vec!(0x1, 0x10f);
    let bv = BitVector::from_vec(&v, 128);
    for i in range(0, 128) {
        println!("rank {} = {}", i, bv.rank1(i));
    }
}
