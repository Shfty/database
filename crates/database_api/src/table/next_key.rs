use std::marker::PhantomData;

/// A type that can provide an incrementing primary key.
pub trait NextKey<K> {
    fn next_key(&self) -> K;
}

/// An iterator that, given a reference to a [PrimaryKey] type, can provide an infinite stream of primary keys.
pub struct NextKeyIterator<'a, T, K>
where
    T: NextKey<K>,
{
    table: &'a T,
    _phantom: PhantomData<K>,
}

impl<'a, T, K> NextKeyIterator<'a, T, K>
where
    T: NextKey<K>,
{
    pub fn new(table: &'a T) -> Self {
        NextKeyIterator {
            table,
            _phantom: Default::default(),
        }
    }
}

impl<'a, T, K> Iterator for NextKeyIterator<'a, T, K>
where
    T: NextKey<K>,
{
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.table.next_key())
    }
}
