//! Wavelet trees

use super::bit_vector;
use super::bits::{BitIter};
use super::build;
use super::tree::binary::{Tree};
use alloc::boxed::Box;
use std::fmt::Show;

/*
impl<BitV: Access<bool>, Sym: FromIterator<bool>>
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

#[deriving(Show)]
pub struct Wavelet<BitV, Sym> {
    tree: Tree<BitV>,
}

pub struct Builder<BitVBuilder, Sym> {
    tree: Wavelet<BitVBuilder, Sym>,
    new_bitvector: fn() -> BitVBuilder,
}

impl<BitV, BitVBuilder: build::Builder<bool, BitV> + Show,
     BI: Iterator<bool>, Sym: BitIter<BI>>
    build::Builder<Sym, Wavelet<BitV, Sym>>
    for Builder<BitVBuilder, Sym> {

        fn push(&mut self, element: Sym) {
            unsafe {
                let mut node: *mut Tree<BitVBuilder> = &mut self.tree.tree;
                for bit in element.bit_iter() {
                    (*node).value.push(bit);
                    println!("pushed {} to {:p}: {}", bit, node, (*node).value);
                    let branch = if bit { &mut (*node).right } else { &mut (*node).left };
                    let next = match branch {
                        &Some(ref mut n) =>
                            &mut **n as *mut Tree<BitVBuilder>,
                        &None => {
                            let mut new = box self.new_node();
                            let ptr = &mut *new as *mut Tree<BitVBuilder>;
                            *branch = Some(new);
                            ptr
                        },
                    };
                    node = next;
                }
            }
        }

        fn finish(self) -> Wavelet<BitV, Sym> {
            Wavelet { tree: self.tree.tree.map_move(|b| b.finish()) }
        }
}

impl<BitVBuilder, Sym> Builder<BitVBuilder, Sym> {
    fn new_node(&self) -> Tree<BitVBuilder> {
        Tree::singleton((self.new_bitvector)())
    }

    pub fn new(new_bitvector: fn() -> BitVBuilder, depth: uint)
               -> Builder<BitVBuilder, Sym> {
        Builder {
            tree: Wavelet {tree: Tree::singleton(new_bitvector())},
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
