/// A type that can convert itself from a [`Row`] type
pub trait FromRow<'a, T>: Sized {
    fn from_row(_: &'a mut T) -> Self;
}
