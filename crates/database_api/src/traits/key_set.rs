use std::{
    collections::{BTreeSet, HashSet},
    hash::Hash,
};

/// A set collection type.
pub trait KeySet<'a, K>
where
    K: 'a,
{
    type Iter: Iterator<Item = &'a K>;

    fn insert(&mut self, key: K) -> bool;
    fn extend(&mut self, values: impl Iterator<Item = K>);

    fn contains(&self, key: &K) -> bool;

    fn remove(&mut self, key: &K) -> bool;

    fn iter(&'a self) -> Self::Iter;
}

impl<'a, K> KeySet<'a, K> for BTreeSet<K>
where
    K: Ord + 'a,
{
    type Iter = std::collections::btree_set::Iter<'a, K>;

    fn insert(&mut self, key: K) -> bool {
        BTreeSet::insert(self, key)
    }

    fn extend(&mut self, values: impl Iterator<Item = K>) {
        std::iter::Extend::extend(self, values)
    }

    fn contains(&self, key: &K) -> bool {
        BTreeSet::contains(self, key)
    }

    fn remove(&mut self, key: &K) -> bool {
        BTreeSet::remove(self, key)
    }

    fn iter(&'a self) -> Self::Iter {
        BTreeSet::iter(self)
    }
}

impl<'a, K> KeySet<'a, K> for HashSet<K>
where
    K: Hash + Eq + 'a,
{
    type Iter = std::collections::hash_set::Iter<'a, K>;

    fn insert(&mut self, key: K) -> bool {
        HashSet::insert(self, key)
    }

    fn extend(&mut self, values: impl Iterator<Item = K>) {
        std::iter::Extend::extend(self, values)
    }

    fn contains(&self, key: &K) -> bool {
        HashSet::contains(self, key)
    }

    fn remove(&mut self, key: &K) -> bool {
        HashSet::remove(self, key)
    }

    fn iter(&'a self) -> Self::Iter {
        HashSet::iter(self)
    }
}
