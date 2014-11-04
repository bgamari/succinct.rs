//! Wavelet trees

use super::bits::{BitIter};
use super::dictionary::{Rank, Select, Access};
use super::build;
use super::tree::binary::{Tree};
use super::tree::binary;

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

impl<Iter: Iterator<bool>, BitV: Rank<bool> + Access<bool>, Sym: BitIter<Iter>> Wavelet<BitV, Sym> {
    /// Efficiently test whether the `n`th position is the given
    /// symbol.
    ///
    /// `wavelet.symbol_eq(sym, n)` is Functionally equivalent to
    /// `wavelet.access(n) == sym` but avoids traversing the entire
    /// depth of the tree in the unequal case.
    pub fn symbol_eq(&self, sym: Sym, mut n: uint) -> bool {
        let mut cursor = binary::Cursor::new(&self.tree);
        for bit in sym.bit_iter() {
            let branch = bit_to_branch(bit);
            match cursor.branch(branch) {
                &None => return false,
                &Some(_) => if bit != cursor.value.get(n) {
                    return false;
                } else {
                    n = cursor.value.rank(bit, n as int) as uint;
                    cursor.move(branch);
                }
            }
        }
        true
    }
}

impl<BitV: Rank<bool> + Access<bool>, Sym: Ord> Wavelet<BitV, Sym> {
    pub fn range_next_value(i: uint, j: uint, sym: Sym) {}
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

impl<BitV, BitVBuilder: build::Builder<bool, BitV>,
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

impl<Iter: Iterator<bool>, BitV: Collection+Access<bool>+Select<bool>, Sym: BitIter<Iter>>
    Select<Sym> for Wavelet<BitV, Sym> {
    fn select(&self, sym: Sym, n: int) -> int {
        if n == 0 { return 0; }
        let mut stack: Vec<(bool, binary::Cursor<BitV>)> = Vec::new();
        let mut cursor = binary::Cursor::new(&self.tree);
        for bit in sym.bit_iter() {
            match cursor.branch(bit_to_branch(bit)) {
                &None    => panic!(),
                &Some(_) => {
                    stack.push((bit, cursor.clone()));
                    cursor.move(bit_to_branch(bit))
                },
            }
        }

        let mut n = n;
        for (bit,cursor) in stack.move_iter().rev() {
            n = cursor.value.select(bit, n);
        }
        n
    }
}

impl<Iter: Iterator<bool>, BitV: Collection+Access<bool>+Rank<bool>, Sym: BitIter<Iter>>
    Rank<Sym> for Wavelet<BitV, Sym> {
    fn rank(&self, sym: Sym, mut idx: int) -> int {
        let mut cursor = binary::Cursor::new(&self.tree);
        for bit in sym.bit_iter() {
            idx = cursor.value.rank(bit, idx);
            match cursor.branch(bit_to_branch(bit)) {
                &None    => return 0,
                &Some(_) => cursor.move(bit_to_branch(bit)),
            }
        }
        idx
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
pub struct FlatWavelet<BitV, Sym> {
    bits: BitV,
}
/*
impl FlatWavelet<BitV, Sym> {
    fn from_tree(tree: Wavelet<BitV, Sym>) -> FlatWavelet<BitV, Sym> {
        // TODO: flatten tree
    }
}
*/

#[cfg(test)]
mod test {
    use quickcheck::TestResult;
    use super::super::dictionary::{Rank, Select};
    use super::super::build::Builder;

    #[quickcheck]
    fn rank_is_correct(el: u8, v: Vec<u8>, n: uint) -> TestResult {
        use super::super::rank9;
        fn new_bitvector() -> rank9::Builder {
           rank9::Builder::with_capacity(128)
        }

        if n > v.len() {
            return TestResult::discard()
        }

        let wavelet = super::Builder::new(new_bitvector).from_iter(v.clone().move_iter());
        let ans = wavelet.rank(el, n as int);
        TestResult::from_bool(ans == v.rank(el, n as int))
    }

    #[quickcheck]
    fn select_is_correct(el: u8, v: Vec<u8>, n: uint) -> TestResult {
        use super::super::bit_vector;
        fn new_bitvector() -> bit_vector::Builder {
           bit_vector::Builder::with_capacity(128)
        }

        if v.iter().filter(|x| *x == &el).count() < n {
            return TestResult::discard()
        }

        let wavelet = super::Builder::new(new_bitvector).from_iter(v.clone().move_iter());
        let ans = wavelet.select(el, n as int);
        TestResult::from_bool(ans == v.select(el, n as int))
    }

    #[test]
    pub fn test_select() {
        use super::super::bit_vector;
        fn new_bitvector() -> bit_vector::Builder {
           bit_vector::Builder::with_capacity(128)
        }

        let v: Vec<u8> = vec!(4, 6, 2, 7, 5, 1, 6, 2);
        let wavelet = super::Builder::new(new_bitvector).from_iter(v.clone().move_iter());
        assert_eq!(wavelet.select(2, 2), 8);
    }

    #[test]
    pub fn test_symbol_eq() {
        use super::super::bit_vector;
        fn new_bitvector() -> bit_vector::Builder {
           bit_vector::Builder::with_capacity(128)
        }
        let v: Vec<u8> = vec!(4, 6, 2, 7, 5, 1, 6, 2);
        let wavelet = super::Builder::new(new_bitvector).from_iter(v.clone().move_iter());
        assert!(wavelet.symbol_eq(7, 3))
        assert!(!wavelet.symbol_eq(7, 2))
    }
}
