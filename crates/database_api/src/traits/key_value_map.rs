use std::{
    collections::{BTreeMap, HashMap},
    hash::{BuildHasher, Hash},
};

/// A key-value map type.
pub trait KeyValueMap<'a, K, V>
where
    K: 'a,
{
    type Keys: Iterator<Item = &'a K>;

    fn insert(&mut self, key: K, value: V) -> Option<V>;
    fn extend(&mut self, values: impl Iterator<Item = (K, V)>);

    fn get(&self, key: &K) -> Option<&V>;
    fn get_mut(&mut self, key: &K) -> Option<&mut V>;

    fn remove(&mut self, key: &K) -> Option<V>;

    fn keys(&'a self) -> Self::Keys;
}

impl<'a, K, V> KeyValueMap<'a, K, V> for BTreeMap<K, V>
where
    K: Ord + 'a,
    V: 'a,
{
    type Keys = std::collections::btree_map::Keys<'a, K, V>;

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        BTreeMap::insert(self, key, value)
    }

    fn extend(&mut self, values: impl Iterator<Item = (K, V)>) {
        std::iter::Extend::extend(self, values)
    }

    fn get(&self, key: &K) -> Option<&V> {
        BTreeMap::get(self, key)
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        BTreeMap::get_mut(self, key)
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        BTreeMap::remove(self, key)
    }

    fn keys(&'a self) -> Self::Keys {
        BTreeMap::keys(self)
    }
}

impl<'a, K, V, S> KeyValueMap<'a, K, V> for HashMap<K, V, S>
where
    K: Hash + Eq + 'a,
    V: 'a,
    S: BuildHasher,
{
    type Keys = std::collections::hash_map::Keys<'a, K, V>;

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        HashMap::insert(self, key, value)
    }

    fn extend(&mut self, values: impl Iterator<Item = (K, V)>) {
        std::iter::Extend::extend(self, values)
    }

    fn get(&self, key: &K) -> Option<&V> {
        HashMap::get(self, key)
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        HashMap::get_mut(self, key)
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        HashMap::remove(self, key)
    }

    fn keys(&'a self) -> Self::Keys {
        HashMap::keys(self)
    }
}

