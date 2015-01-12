//! Various traits for working with bits and objects composed of them

use std::ops::{Shr, BitAnd};
use std::iter::Iterator;
use std::num::Int;
use std::mem::size_of;

/// An iterator over the bits of a primitive type
/// The least significant bit is produced first.
pub struct BitIterator<T> {
    bit: uint,
    x: T,
}

impl<T> BitIterator<T> {
    pub fn new(x: T) -> BitIterator<T> {
        BitIterator {
            bit: 8*size_of::<T>(),
            x: x,
        }
    }

    pub fn with_width(bits: uint, x: T) -> BitIterator<T> {
        BitIterator {
            bit: bits,
            x: x
        }
    }
}

impl<T: Shr<uint> + BitAnd<T> + Int> Iterator for BitIterator<T> {
    type Item = bool;
    fn next(&mut self) -> Option<bool> {
        match self.bit {
            0 => None,
            _ => {
                let res = Some(!(self.x & Int::one()) == Int::zero());
                self.bit -= 1;
                self.x = self.x >> 1;
                res
            }
        }
    }
}

/// A trait for types for which one can get an iterator over bits
pub trait BitIter {
    type Iter: Iterator<Item=bool>;
    fn bit_iter(self) -> <Self as BitIter>::Iter;
}

impl BitIter for u64 {
    type Iter = BitIterator<u64>;
    fn bit_iter(self) -> BitIterator<u64> {BitIterator::new(self)}
}

impl BitIter for u32 {
    type Iter = BitIterator<u32>;
    fn bit_iter(self) -> BitIterator<u32> {BitIterator::new(self)}
}

impl BitIter for u16 {
    type Iter = BitIterator<u16>;
    fn bit_iter(self) -> BitIterator<u16> {BitIterator::new(self)}
}

impl BitIter for u8 {
    type Iter = BitIterator<u8>;
    fn bit_iter(self) -> BitIterator<u8> {BitIterator::new(self)}
}

/// A trait for types for which one can extract arbitrary bits
trait Bitwise {
    fn width(&self) -> uint;
    fn bit(&self, n: uint) -> bool;
}

impl Bitwise for u64 {
    fn width(&self) -> uint {64}
    fn bit(&self, n: uint) -> bool {(*self >> n) & 1 == 1}
}
