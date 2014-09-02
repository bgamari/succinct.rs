pub trait Builder<E, T> {
    fn push(&mut self, element: &E);
    fn finish(self) -> T;
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
