pub use tree::binary::cursor::Cursor;
pub use tree::binary::mut_cursor::MutCursor;
use std::fmt;

/// A child branch of a `Tree`
#[derive(Show)]
pub enum Branch {Left, Right}

/// A binary tree with nodes labelled with `T`
pub struct Tree<T> {
    pub value: T,
    pub left: Option<Box<Tree<T>>>,
    pub right: Option<Box<Tree<T>>>,
}

impl<T: fmt::Show> fmt::Show for Tree<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use std::fmt::Show;
        fn indent(level: uint, fmt: &mut fmt::Formatter) -> fmt::Result {
            for i in range(0, level) {try!(" ".fmt(fmt))}
            Ok(())
        }

        fn go<T: Show>(tree: &Tree<T>, fmt: &mut fmt::Formatter, level: uint) -> fmt::Result {
            try!(indent(2*level, fmt));
            try!(write!(fmt, "+ node {:p}    ", tree));
            try!(tree.value.fmt(fmt));
            try!("\n".fmt(fmt));

            try!(indent(2*level, fmt));
            try!("| left:\n".fmt(fmt));
            for subtree in tree.left.iter() {try!(go(&**subtree, fmt, level+1)); }

            try!(indent(2*level, fmt));
            try!("| right:\n".fmt(fmt));
            for subtree in tree.right.iter() {try!(go(&**subtree, fmt, level+1)); }
            try!("\n".fmt(fmt));
            Ok(())
        }
        go(self, fmt, 0)
    }
}

impl<T> Tree<T> {
    pub fn singleton(value: T) -> Tree<T> {
        Tree {value: value, left: None, right: None}
    }

    pub fn map<F, V>(&self, f: F) -> Tree<V>
        where F : Fn(&T) -> V {
        Tree {
            left: self.left.as_ref().map(|x| box x.map(|y| f(y))),
            right: self.right.as_ref().map(|x| box x.map(|y| f(y))),
            value: f(&self.value),
        }
    }

    pub fn map_step<F, V>(self, f: F) -> Tree<V>
        where F: Fn(T) -> V {
        Tree {
            left: self.left.map(|x| box x.map_step(|y| f(y))),
            right: self.right.map(|x| box x.map_step(|y| f(y))),
            value: f(self.value),
        }
    }

    pub fn branch(&self, branch: Branch) -> &Option<Box<Tree<T>>> {
        match branch {
            Left => &self.left,
            Right => &self.right,
        }
    }

    pub fn branch_mut(&mut self, branch: Branch) -> &mut Option<Box<Tree<T>>> {
        match branch {
            Left => &mut self.left,
            Right => &mut self.right,
        }
    }
}

mod mut_cursor {
    use std::ops::{Deref, DerefMut};
    use super::{Tree, Branch};

    /// A cursor allowing safe navigation and mutation of `Trees`
    pub struct MutCursor<'a, T: 'a> {
        root: &'a mut Tree<T>,
        node: *mut Tree<T>,
    }

    impl<'a, T> MutCursor<'a, T> {
        /// Create a new `Cursor` pointing to the root of the given `Tree`
        pub fn new(tree: &'a mut Tree<T>) -> MutCursor<'a, T> {
            MutCursor {
                root: tree,
                node: tree,
            }
        }

        /// Step the cursor back to the root
        pub fn back_to_root(&mut self) {
            self.node = self.root as *mut Tree<T>;
        }

        /// Descend down one of the branches
        pub fn step(&mut self, branch: Branch) {
            unsafe {
                use super::Branch::{Left, Right};
                let b: &mut Option<Box<Tree<T>>> = match branch {
                    Left => &mut (*self.node).left,
                    Right => &mut (*self.node).right,
                };
                match b {
                    &mut None => panic!("Attempted to step {:?} into empty branch", branch),
                    &mut Some(ref mut child) => {
                        self.node = &mut **child as *mut Tree<T>;
                    }
                }
            }
        }

        /// Reclaim the tree
        pub fn finish(self) -> &'a mut Tree<T> {
            self.root
        }
    }

    impl<'a, T> Deref for MutCursor<'a, T> {
        type Target = Tree<T>;
        fn deref<'b>(&'b self) -> &'b Tree<T> {
            unsafe{ &*self.node }
        }
    }

    impl<'a, T> DerefMut for MutCursor<'a, T> {
        fn deref_mut<'b>(&'b mut self) -> &'b mut Tree<T> {
            unsafe{ &mut *self.node }
        }
    }
}

mod cursor {
    use std::ops::Deref;
    use super::{Tree, Branch};

    /// A cursor allowing safe navigation of `Trees`
    pub struct Cursor<'a, T: 'a> {
        root: &'a Tree<T>,
        node: *const Tree<T>,
    }

    impl<'a, T> Clone for Cursor<'a, T> {
        fn clone(&self) -> Cursor<'a, T> {
            Cursor {
                root: self.root,
                node: self.node,
            }
        }
    }

    impl<'a, T> Cursor<'a, T> {
        /// Create a new `Cursor` pointing to the root of the given `Tree`
        pub fn new(tree: &'a Tree<T>) -> Cursor<'a, T> {
            Cursor {
                root: tree,
                node: tree,
            }
        }

        /// Step the cursor back to the root
        pub fn back_to_root(&mut self) {
            self.node = self.root as *const Tree<T>;
        }

        /// Descend down one of the branches
        pub fn step(&mut self, branch: Branch) {
            unsafe {
                use super::Branch::{Left, Right};
                let b: &Option<Box<Tree<T>>> = match branch {
                    Left => &(*self.node).left,
                    Right => &(*self.node).right,
                };
                match b {
                    &None => panic!("Attempted to step {:?} into empty branch", branch),
                    &Some(ref child) => {
                        self.node = &**child as *const Tree<T>;
                    }
                }
            }
        }
    }

    impl<'a, T> Deref for Cursor<'a, T> {
        type Target = Tree<T>;
        fn deref<'b>(&'b self) -> &'b Tree<T> {
            unsafe{ &*self.node }
        }
    }
}
