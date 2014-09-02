//! Wavelet trees

use super::bits::{BitIter};
use super::dictionary::{Rank, Access};
use super::build;
use super::tree::binary::{Tree};
use super::tree::binary;
use std::fmt::Show;

fn bit_to_branch(bit: bool) -> binary::Branch {
    match bit {
        true => binary::Right,
        false => binary::Left,
    }
}

/// A wavelet tree over symbols of type `Sym`
#[deriving(Show)]
pub struct Wavelet<BitV, Sym> {
    tree: Tree<BitV>,
}

impl<BitV: Rank<bool> + Access<bool>, Sym> Wavelet<BitV, Sym> {
    /// TODO: This needs to turn into an `Access` impl once
    /// `Buildable` has an associated `Builder` type
    pub fn access<SymBuilder: build::Builder<bool, Sym>>(&self, mut builder: SymBuilder, mut n: uint) -> Sym {
        let mut cursor = binary::Cursor::new(&self.tree);
        loop {
            if cursor.branch(binary::Left).is_none() {  // HACK: encode the leaf
                break;
            }
            let bit = cursor.value.get(n);
            builder.push(bit);
            let branch = bit_to_branch(bit);
            println!("on node {:p}", &*cursor);
            match cursor.branch(branch) {
                &None => break,
                &Some(_) => {
                    n = cursor.value.rank(bit, n as int) as uint;
                    cursor.move(branch)
                },
            }
        }
        builder.finish()
    }
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
