//! Various traits for working with bits and objects composed of them

use std::ops::{Shr, BitAnd};
use std::iter::Iterator;
use std::num::{One, Zero};
use std::mem::size_of;

/// An iterator over the bits of a primitive type
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

impl<T: Shr<uint, T> + BitAnd<T, T> + One + Zero> Iterator<bool> for BitIterator<T> {
    fn next(&mut self) -> Option<bool> {
        match self.bit {
            0 => None,
            _ => {
                let res = Some(!(self.x & One::one()).is_zero());
                self.bit -= 1;
                self.x = self.x >> 1;
                res
            }
        }
    }
}

/// A trait for types for which one can get an iterator over bits
pub trait BitIter<Iter: Iterator<bool>> {
    // TODO: associated type here
    fn bit_iter(self) -> Iter;
}

impl BitIter<BitIterator<u64>> for u64 {
    fn bit_iter(self) -> BitIterator<u64> {BitIterator::new(self)}
}

impl BitIter<BitIterator<u32>> for u32 {
    fn bit_iter(self) -> BitIterator<u32> {BitIterator::new(self)}
}

impl BitIter<BitIterator<u16>> for u16 {
    fn bit_iter(self) -> BitIterator<u16> {BitIterator::new(self)}
}

impl BitIter<BitIterator<u8>> for u8 {
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
