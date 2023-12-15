use std::any::TypeId;

/// A type that can cache sets of keys by [TypeId].
pub trait Keys<'a, K>
where
    K: 'a,
{
    type Keys: Iterator<Item = K>;

    fn insert_key(&'a self, type_id: TypeId, key: K);
    fn extend_keys(&'a self, type_id: TypeId, keys: impl Iterator<Item = K>);
    fn remove_key(&'a self, type_id: &TypeId, key: &K);
    fn keys(&'a self, type_id: &TypeId) -> Self::Keys;
}
