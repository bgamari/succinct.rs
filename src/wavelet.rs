//! Wavelet trees

use super::bits::{BitIter};
use super::build;
use super::tree::binary::{Tree};
use super::tree::binary;
use std::fmt::Show;

fn bit_to_branch(bit: bool) -> binary::Branch {
    match bit {
        True => binary::Right,
        False => binary::Left,
    }
}

mod access {
    //! Access operation
    use super::super::dictionary::Access;
    use super::super::tree::binary::{Tree, Cursor};
    use super::super::build::{Builder, Buildable};
    use super::{Wavelet, bit_to_branch};

    /// Iterator over the bits of symbol `i`
    struct AccessIter<'a, BitV: 'a, Sym> {
        cursor: Cursor<'a, BitV>
    }

    impl<BuilderT: Builder<bool, Sym>, BitV: Access<bool>, Sym: Buildable<bool, BuilderT>>
        Access<Sym>
        for Wavelet<BitV, Sym> {

        fn get(&self, mut n: uint) -> Sym {
            let builder: BuilderT = Buildable::new_builder();
            let cursor = Cursor::new(&self.tree);
            loop {
                let bit = cursor.value.get(n);
                builder.push(bit);
                let branch = bit_to_branch(bit);
                match cursor.branch(branch) {
                    &None => break,
                    &Some(_) => {}
                }
            }
            builder.finish()
        }
    }
}

/// A wavelet tree over symbols of type `Sym`
#[deriving(Show)]
pub struct Wavelet<BitV, Sym> {
    tree: Tree<BitV>,
}

/// Build up a wavelet tree from a sequence of symbols.
///
/// We expect that the symbols are of homogenous bitwidth.
pub struct Builder<BitVBuilder, Sym> {
    tree: Wavelet<BitVBuilder, Sym>,
    new_bitvector: fn() -> BitVBuilder,
}

impl<BitV, BitVBuilder: build::Builder<bool, BitV> + Show,
     BI: Iterator<bool>, Sym: BitIter<BI>>
    build::Builder<Sym, Wavelet<BitV, Sym>>
    for Builder<BitVBuilder, Sym> {

        fn push(&mut self, element: Sym) {
            let new_bitvector = &self.new_bitvector;
            let mut cursor = binary::MutCursor::new(&mut self.tree.tree);
            for bit in element.bit_iter() {
                cursor.value.push(bit);
                let branch = bit_to_branch(bit);
                match cursor.branch_mut(branch) {
                    &Some(_) => {},
                    n => *n = Some(box Tree::singleton((*new_bitvector)())),
                }
                cursor.move(branch);
            }
        }

        fn finish(self) -> Wavelet<BitV, Sym> {
            Wavelet { tree: self.tree.tree.map_move(|b| b.finish()) }
        }
}

impl<BitVBuilder, Sym> Builder<BitVBuilder, Sym> {
    pub fn new(new_bitvector: fn() -> BitVBuilder)
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
