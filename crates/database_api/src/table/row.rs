use super::{Keys, NextKey};

/// A type used to read/write sets of [Column]s
pub trait Row<'a, Tbl, K>: Sized
where
    Tbl: Keys<'a, K> + NextKey<K>,
    K: 'a,
    <Self as Row<'a, Tbl, K>>::Insert: 'static,
{
    type Insert;
    type Result;

    type OuterReadGuards;
    type OuterWriteGuards;
    type InnerGuards;

    fn keys(tbl: &'a Tbl) -> <Tbl as Keys<K>>::Keys
    where
        Tbl: Keys<'a, K>,
    {
        tbl.keys(&std::any::TypeId::of::<Self::Insert>())
    }

    fn read_columns(tbl: &'a Tbl) -> Self::OuterReadGuards;

    fn get_row(tbl: &'a Tbl, read_columns: &'a Self::OuterReadGuards, key: &K)
        -> Self::InnerGuards;

    fn write_columns(tbl: &'a Tbl) -> Self::OuterWriteGuards;

    fn insert(
        tbl: &'a Tbl,
        write_columns: &mut Self::OuterWriteGuards,
        key: K,
        values: Self::Insert,
    ) -> Self::Result;

    fn extend(
        tbl: &'a Tbl,
        write_columns: &mut Self::OuterWriteGuards,
        values: impl Iterator<Item = (K, Self::Insert)>,
    );

    fn remove(tbl: &'a Tbl, write_columns: &mut Self::OuterWriteGuards, key: &K) -> Self::Result;

    fn key_cache_id(_tbl: &Tbl) -> std::any::TypeId {
        std::any::TypeId::of::<Self::Insert>()
    }
}
