use std::{
    any::TypeId,
    array::IntoIter as ArrayIter,
    borrow::Cow,
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashMap},
    ops::{Deref, DerefMut},
    sync::{atomic::AtomicUsize, Mutex, RwLock},
};

use crate as database_api;
use crate::{FromRow, KeySet, Keys, Lock, NextKey, NextKeyIterator, Row};

// Test Code
// TODO: Fix Row derive when used in foreign crates
//       Probably better to nix the CellMap-specific guards
//       Newtype seems like the idiomatic pattern here
//       Worst-case, move those methods into Row (could be useful for API helpers)

// TODO: Fix key caching - currently only works correctly if components are inserted and queried by the same row
//       i.e. Will break for systems that share components, need to update all interested row caches on insert

// TODO: Integrate with ecs_bench_suite
// TODO: Derive-friendly generics-driven key caching implementation
// TODO: Can cloning the inner key cache for iteration be avoided?
// TODO: Investigate async compatibility
//       Looks like it would run very deep - probably better to try without it for now

#[derive(Debug, Default, crate::macros::Column)]
pub struct Table {
    primary_key: AtomicUsize,

    #[skip_column]
    key_cache: RefCell<BTreeMap<TypeId, RefCell<BTreeSet<usize>>>>,

    ints: RefCell<BTreeMap<usize, RefCell<u32>>>,
    floats: parking_lot::RwLock<HashMap<usize, parking_lot::RwLock<f32>>>,
    chars: RwLock<BTreeMap<usize, RwLock<char>>>,
    strs: Mutex<HashMap<usize, Mutex<Cow<'static, str>>, fnv::FnvBuildHasher>>,
}

impl NextKey<usize> for Table {
    fn next_key(&self) -> usize {
        self.primary_key
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

impl<'a> Keys<'a, usize> for Table {
    type Keys = std::collections::btree_set::IntoIter<usize>;

    fn insert_key(&'a self, type_id: TypeId, key: usize) {
        KeySet::insert(
            self.key_cache
                .write()
                .entry(type_id)
                .or_default()
                .write()
                .deref_mut(),
            key,
        );
    }

    fn extend_keys(&'a self, type_id: TypeId, keys: impl Iterator<Item = usize>) {
        KeySet::extend(
            self.key_cache
                .write()
                .entry(type_id)
                .or_default()
                .write()
                .deref_mut(),
            keys,
        );
    }

    fn remove_key(&'a self, type_id: &TypeId, key: &usize) {
        KeySet::remove(
            self.key_cache
                .write()
                .get(type_id)
                .expect("Tried to uncache a non-cached key.")
                .write()
                .deref_mut(),
            key,
        );
    }

    fn keys(&'a self, type_id: &TypeId) -> Self::Keys {
        self.key_cache
            .read()
            .get(type_id)
            .unwrap()
            .read()
            .clone()
            .into_iter()
    }
}

#[derive(Debug, crate::macros::Row)]
pub struct IntFloatRow<'a> {
    int: &'a u32,
    float: &'a mut f32,
}

impl<'a, T1, T2> FromRow<'a, (T1, T2)> for IntFloatRow<'a>
where
    T1: Deref<Target = u32>,
    T2: DerefMut<Target = f32>,
{
    fn from_row((int, float): &'a mut (T1, T2)) -> Self {
        IntFloatRow {
            int: Deref::deref(int),
            float: DerefMut::deref_mut(float),
        }
    }
}

#[derive(Debug, crate::macros::Row)]
pub struct CharStrRow<'a> {
    char: &'a char,
    str: &'a mut Cow<'static, str>,
}

impl<'a, T1, T2> FromRow<'a, (T1, T2)> for CharStrRow<'a>
where
    T1: Deref<Target = char>,
    T2: DerefMut<Target = Cow<'static, str>>,
{
    fn from_row((char, str): &'a mut (T1, T2)) -> Self {
        CharStrRow {
            char: Deref::deref(char),
            str: DerefMut::deref_mut(str),
        }
    }
}

#[test]
fn test_database_api() {
    // Create table
    let table = Table::default();

    // Take a mutable view over int / float columns
    let mut columns = IntFloatRow::write_columns(&table);

    // Insert values
    IntFloatRow::extend(
        &table,
        &mut columns,
        NextKeyIterator::new(&table).zip(ArrayIter::new([
            (10, 10.0),
            (20, 20.0),
            (30, 30.0),
            (40, 40.0),
            (50, 50.0),
            (60, 60.0),
        ])),
    );

    // Remove values
    IntFloatRow::remove(&table, &mut columns, &1);
    IntFloatRow::remove(&table, &mut columns, &3);
    IntFloatRow::remove(&table, &mut columns, &5);

    drop(columns);

    // Take a mutable view over char / str columns
    let mut columns = CharStrRow::write_columns(&table);

    // Insert values
    CharStrRow::extend(
        &table,
        &mut columns,
        NextKeyIterator::new(&table).zip(ArrayIter::new([
            ('a', "Foo".into()),
            ('b', "Bar".into()),
            ('c', "Baz".into()),
            ('d', "Decafisbad".into()),
        ])),
    );

    drop(columns);

    // Iterate over int / float columns and print
    let columns = IntFloatRow::read_columns(&table);
    for key in IntFloatRow::keys(&table) {
        let mut row = IntFloatRow::get_row(&table, &columns, &key);
        let int_float_row = IntFloatRow::from_row(&mut row);
        println!("Key {}: {:#?}", key, int_float_row);
    }
    drop(columns);

    // Iterate over int / float columns and print
    let columns = CharStrRow::read_columns(&table);
    for key in CharStrRow::keys(&table) {
        let mut row = CharStrRow::get_row(&table, &columns, &key);
        let char_str_row = CharStrRow::from_row(&mut row);
        println!("Key {}: {:#?}", key, char_str_row);
    }
}
