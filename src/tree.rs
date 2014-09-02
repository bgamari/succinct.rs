enum Rose<T> {
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
    }
}
