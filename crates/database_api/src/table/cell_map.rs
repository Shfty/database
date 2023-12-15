use crate::traits::{KeyValueMap, Lock};

/// A [KeyValueMap] type containing a [`Lock`] value type
pub trait CellMap<'a, K, L, V>: KeyValueMap<'a, K, L>
where
    K: 'a,
    L: Lock<'a, V> + 'a,
{
    fn read_cell(&'a self, key: &K) -> Option<<L as Lock<'a, V>>::ReadGuard> {
        let cell = self.get(key);
        cell.map(Lock::read)
    }

    fn write_cell(&'a self, key: &K) -> Option<<L as Lock<'a, V>>::WriteGuard> {
        let cell = self.get(key);
        cell.map(Lock::write)
    }

    fn insert(&mut self, key: K, value: V) -> Option<L>
    where
        L: From<V>,
    {
        KeyValueMap::insert(self, key, value.into())
    }

    fn extend(&mut self, values: impl Iterator<Item = (K, V)>) {
        KeyValueMap::extend(self, values.map(|(key, value)| (key, value.into())))
    }

    fn remove(&mut self, key: &K) -> Option<L> {
        KeyValueMap::remove(self, key)
    }
}

impl<'a, K, L, V, T> CellMap<'a, K, L, V> for T
where
    K: 'a,
    T: KeyValueMap<'a, K, L>,
    L: Lock<'a, V> + 'a,
{
}

