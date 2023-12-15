use crate::traits::Lock;

use super::CellMap;

/// A type with OuterLock<CellMap<Key, InnerLock<Value>>> structure.
pub trait Column<'a, K, T>
where
    K: 'a,
{
    type OuterLock: Lock<'a, Self::CellMap> + 'a;
    type CellMap: CellMap<'a, K, Self::InnerLock, T> + 'a;
    type InnerLock: Lock<'a, T> + 'a;

    fn outer_lock(&'a self) -> &'a Self::OuterLock;

    fn read_cell_map(&'a self) -> <Self::OuterLock as Lock<'a, Self::CellMap>>::ReadGuard {
        self.outer_lock().read()
    }

    fn write_cell_map(&'a self) -> <Self::OuterLock as Lock<'a, Self::CellMap>>::WriteGuard {
        self.outer_lock().write()
    }
}
