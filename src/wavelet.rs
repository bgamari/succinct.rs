//! Wavelet trees

use super::bit_vector;
use super::bits::{BitIter};
use super::build;
use arena::TypedArena;

/**
An unpacked wavelet tree.

This representation is primarily intended for building.
*/
struct Wavelet<'a, BitV: 'a, Sym> {
    bits: BitV,
    zeros: Option<&'a Wavelet<'a, BitV, Sym>>,
    ones: Option<&'a Wavelet<'a, BitV, Sym>>,
}

/*
impl<BitV: Access<bool>, Sym: FromIterator<>>
    Access<Sym> for Wavelet<BitV, Sym> {
        fn get(&self) -> Sym {
            let hi;
            for i in range() {
                bits
            }
        }
    }
}
*/

struct Builder<'a, BitVBuilder: 'a, Sym> {
    tree: Wavelet<'a, BitVBuilder, Sym>,
    arena: TypedArena<Wavelet<'a, BitVBuilder, Sym>>,
    new_bitvector: ||: 'a -> BitVBuilder,
}

impl<'a, BitV, BitVBuilder: build::Builder<bool, BitV>,
     Sym: BitIter<BI>, BI: Iterator<bool>>
    build::Builder<Sym, Wavelet<'a, BitV, Sym>>
    for Builder<'a, BitVBuilder, Sym> {

        fn push(&mut self, element: &Sym) {
            let mut node = &self.tree;
            for bit in element.bit_iter() {
                node.bits.push(&bit);
                let next = if bit { &node.ones } else { &node.zeros };
                match *next {
                    None => {
                        //let new = self.new_node();
                        //*next = Some(new);
                    },
                    _    => { },
                }
                node = next.unwrap();
            }
        }

        fn finish(self) -> Wavelet<'a, BitV, Sym> {
            fail!("FIXME");
        }
}

impl<'a, BitVBuilder, Sym> Builder<'a, BitVBuilder, Sym> {
    fn new_node(&'a self) -> &'a Wavelet<'a, BitVBuilder, Sym> {
        let node = Wavelet {
            bits: (self.new_bitvector)(),
            zeros: None,
            ones: None,
        };
        self.arena.alloc(node)
    }

    pub fn new(new_bitvector: ||: 'a -> BitVBuilder, depth: uint)
               -> Builder<'a, BitVBuilder, Sym> {
        Builder {
            tree: Wavelet {
                bits: (new_bitvector)(),
                zeros: None,
                ones: None,
            },
            arena: TypedArena::new(),
            new_bitvector: new_bitvector,
        }
    }
}

/**
A packed wavelet tree.

Here the node bitvectors are packed into a single bitvector, removing
the need for forwarding pointers.
*/
struct FlatWavelet<BitV, Sym> {
    bits: BitV,
}
/*
impl FlatWavelet<BitV, Sym> {
    fn from_tree(tree: Wavelet<BitV, Sym>) -> FlatWavelet<BitV, Sym> {
        // TODO: flatten tree
    }
}
*/
