pub enum Rose<T> {
    Leaves(Vec<T>),
    Nodes(Vec<Rose<T>>),
}

pub mod binary {
    /// A binary tree with nodes labelled with `T`
    #[deriving(Show)]
    pub struct Tree<T> {
        pub value: T,
        pub left: Option<Box<Tree<T>>>,
        pub right: Option<Box<Tree<T>>>,
    }

    impl<T> Tree<T> {
        pub fn singleton(value: T) -> Tree<T> {
            Tree {value: value, left: None, right: None}
        }

        pub fn map<V>(&self, f: |&T| -> V) -> Tree<V> {
            Tree {
                left: self.left.as_ref().map(|x| box x.map(|y| f(y))),
                right: self.right.as_ref().map(|x| box x.map(|y| f(y))),
                value: f(&self.value),
            }
        }

        pub fn map_move<V>(self, f: |T| -> V) -> Tree<V> {
            Tree {
                left: self.left.map(|x| box x.map_move(|y| f(y))),
                right: self.right.map(|x| box x.map_move(|y| f(y))),
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

    /// A child branch of a `Tree`
    #[deriving(Show)]
    pub enum Branch {Left, Right}

    /// A cursor allowing safe navigation of `Trees`
    pub struct Cursor<'a, T: 'a> {
        root: &'a mut Tree<T>,
        node: *mut Tree<T>,
    }

    impl<'a, T> Cursor<'a, T> {
        /// Create a new `Cursor` pointing to the root of the given `Tree`
        pub fn new(tree: &'a mut Tree<T>) -> Cursor<'a, T> {
            Cursor {
                root: tree,
                node: tree,
            }
        }

        /// Move the cursor back to the root
        pub fn back_to_root(&mut self) {
            self.node = self.root as *mut Tree<T>;
        }

        /// Descend down one of the branches
        pub fn move(&mut self, branch: Branch) {
            unsafe {
                let branch: &mut Option<Box<Tree<T>>> = match branch {
                    Left => &mut (*self.node).left,
                    Right => &mut (*self.node).right,
                };
                match branch {
                    &None => fail!("Attempted to move {} into empty branch"),
                    &Some(ref mut child) => {
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

    impl<'a, T> Deref<Tree<T>> for Cursor<'a, T> {
        fn deref<'b>(&'b self) -> &'b Tree<T> {
            unsafe{ &*self.node }
        }
    }

    impl<'a, T> DerefMut<Tree<T>> for Cursor<'a, T> {
        fn deref_mut<'b>(&'b mut self) -> &'b mut Tree<T> {
            unsafe{ &mut *self.node }
        }
    }
}
