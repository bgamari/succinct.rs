//! Traits for building up objects incrementally

pub trait Builder<E, T> {
    fn push(&mut self, element: &E);
    fn finish(self) -> T;

    fn from_iter<Iter: Iterator<E>>(mut self, mut iter: Iter) -> T {
        for i in iter {
            self.push(&i);
        }
        self.finish()
    }
}

/// Build a stream of `u64`s from a stream of bits
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
    fn push(&mut self, element: &bool) {
        self.accum |= (*element as u64) << self.bit;
        self.bit += 1;
        self.size += 1;
        if self.bit == 64 {
            self.builder.push(&self.accum);
            self.bit = 0;
            self.accum = 0;
        }
    }

    #[inline(always)]
    fn finish(self) -> (T, uint) {
        (self.builder.finish(), self.size)
    }
}

/// Build up a `Vec` from elements
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
    fn push(&mut self, e: &T) {
        self.buffer.push((*e).clone());
    }
    fn finish(self) -> Vec<T> {
        self.buffer
    }
}

/// A pair of `Builder`s is also a `Builder`
impl<T, RA, RB, A: Builder<T, RA>, B: Builder<T, RB>> Builder<T, (RA, RB)> for (A, B) {
    fn push(&mut self, e: &T) {
        let (ref mut a, ref mut b) = *self;
        a.push(e);
        b.push(e);
    }
    fn finish(self) -> (RA, RB) {
        let (a, b) = self;
        (a.finish(), b.finish())
    }
}
