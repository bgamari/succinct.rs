//! Wavelet trees

use super::bit_vector;
use super::bits::{BitIter};
use super::build;
use arena::TypedArena;

/**
An unpacked wavelet tree.

This representation is primarily intended for building.
*/

/// A binary tree with nodes labelled with `T`
struct BinaryTree<'a, T: 'a> {
    value: T,
    left: Option<&'a mut BinaryTree<'a, T>>,
    right: Option<&'a mut BinaryTree<'a, T>>,
}

impl<'a, T: 'a> BinaryTree<'a, T> {
    pub fn singleton(value: T) -> BinaryTree<'a, T> {
        BinaryTree {value: value, left: None, right: None}
    }
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

struct Wavelet<'a, BitV: 'a, Sym> {
    tree: BinaryTree<'a, BitV>,
}

struct Builder<'a, BitVBuilder: 'a, Sym> {
    tree: Wavelet<'a, BitVBuilder, Sym>,
    arena: TypedArena<BinaryTree<'a, BitVBuilder>>,
    new_bitvector: fn() -> BitVBuilder,
}

impl<'a, BitV, BitVBuilder: build::Builder<bool, BitV>,
     BI: Iterator<bool>, Sym: BitIter<BI>>
    build::Builder<Sym, Wavelet<'a, BitV, Sym>>
    for Builder<'a, BitVBuilder, Sym> {

        fn push(&'a mut self, element: Sym) {
            let mut node: &mut BinaryTree<'a, BitVBuilder> = &mut self.tree.tree;
            for bit in element.bit_iter() {
                node.value.push(bit);
                let next = if bit { &node.right } else { &node.left };
                node = match *next {
                    None => {
                        let new = self.new_node();
                        *next = Some(new);
                        new
                    },
                    Some(n) => n
                };
            }
        }

        fn finish(self) -> Wavelet<'a, BitV, Sym> {
            fail!("FIXME");
        }
}

impl<'a, BitVBuilder, Sym> Builder<'a, BitVBuilder, Sym> {
    fn new_node(&'a self) -> &'a mut BinaryTree<'a, BitVBuilder> {
        let node = BinaryTree::singleton((self.new_bitvector)());
        self.arena.alloc(node)
    }

    pub fn new(new_bitvector: fn() -> BitVBuilder, depth: uint)
               -> Builder<'a, BitVBuilder, Sym> {
        Builder {
            tree: Wavelet {
                tree: BinaryTree::singleton((new_bitvector)()),
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
