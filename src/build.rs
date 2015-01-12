//! Traits for building up objects incrementally

pub use build::buildable::{Buildable, PrimBuilder};

pub trait Builder<E, T> where Self: Sized {
    fn push(&mut self, element: E);
    fn finish(self) -> T;

    fn from_iter<Iter: Iterator<Item=E>>(mut self, mut iter: Iter) -> T {
        for i in iter {
            self.push(i);
        }
        self.finish()
    }
}

/// Build a stream of `u64`s from a stream of bits
#[deriving(Show)]
pub struct BitBuilder<B> {
    builder: B,
    accum: u64,
    bit: uint,
    size: uint,
}

impl<B> BitBuilder<B> {
    pub fn new(builder: B) -> BitBuilder<B> {
        BitBuilder {
            builder: builder,
            accum: 0,
            bit: 0,
            size: 0,
        }
    }
}

/// Returns both result and size in bits
impl<T, B: Builder<u64, T>> Builder<bool, (T, uint)> for BitBuilder<B> {
    #[inline(always)]
    fn push(&mut self, element: bool) {
        self.accum |= (element as u64) << self.bit;
        self.bit += 1;
        self.size += 1;
        if self.bit == 64 {
            self.builder.push(self.accum);
            self.bit = 0;
            self.accum = 0;
        }
    }

    #[inline(always)]
    fn finish(mut self) -> (T, uint) {
        // push partial word
        if self.bit % 64 != 0 {
            self.builder.push(self.accum);
        }
        (self.builder.finish(), self.size)
    }
}

/// Build up a `Vec` from elements
#[deriving(Show)]
pub struct VecBuilder<T> {
    buffer: Vec<T>,
}

impl<T> VecBuilder<T> {
    pub fn with_capacity(cap: uint) -> VecBuilder<T> {
        VecBuilder {
            buffer: Vec::with_capacity(cap),
        }
    }
}

impl<T: Clone> Builder<T, Vec<T>> for VecBuilder<T> {
    fn push(&mut self, e: T) {
        self.buffer.push(e);
    }
    fn finish(self) -> Vec<T> {
        self.buffer
    }
}

/// A pair of `Builder`s is also a `Builder`
impl<T: Clone, RA, RB, A: Builder<T, RA>, B: Builder<T, RB>> Builder<T, (RA, RB)> for (A, B) {
    fn push(&mut self, e: T) {
        let (ref mut a, ref mut b) = *self;
        a.push(e.clone());
        b.push(e);
    }
    fn finish(self) -> (RA, RB) {
        let (a, b) = self;
        (a.finish(), b.finish())
    }
}

mod buildable {
    use std::ops::{Shl, BitOr};
    use std::num::Int;
    use std::mem::size_of;
    use super::Builder;

    /// A trait for things that can be built from elements of type `E`
    pub trait Buildable<E, BuilderT: Builder<E, Self>> {
        // TODO: Associated type
        fn new_builder() -> BuilderT;
    }

    /// Build primitive values from their bits (least significant bit first)
    pub struct PrimBuilder<T> {
        prim: T,
        bit: uint,
    }

    impl<T: Int> PrimBuilder<T> {
        pub fn new() -> PrimBuilder<T> {
            PrimBuilder {
                prim: Int::zero(),
                bit: 0,
            }
        }
    }

    impl<T: Shl<usize> + BitOr<T,Output=T> + Int> Builder<bool, T> for PrimBuilder<T> {
        fn push(&mut self, e: bool) {
            debug_assert!(self.bit < size_of::<T>() * 8);
            if e {
                let one: T = Int::one();
                self.prim = self.prim | (one << self.bit);
            }
            self.bit += 1;
        }
        fn finish(self) -> T {
            self.prim
        }
    }

    impl Buildable<bool, PrimBuilder<u64>> for u64 {
        fn new_builder() -> PrimBuilder<u64> {
            PrimBuilder::new()
        }
    }
}
