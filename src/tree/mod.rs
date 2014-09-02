//! Various trees

pub mod binary;

pub enum Rose<T> {
    Leaves(Vec<T>),
    Nodes(Vec<Rose<T>>),
}
