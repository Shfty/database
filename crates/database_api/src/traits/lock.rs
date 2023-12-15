use std::{
    cell::{Ref, RefCell, RefMut},
    ops::{Deref, DerefMut},
    sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

/// A interior mutable type that can hand out read and write guards to its underlying data.
pub trait Lock<'a, T>: Default + From<T> {
    type ReadGuard: Deref<Target = T>;
    type WriteGuard: DerefMut<Target = T>;

    fn read(&'a self) -> Self::ReadGuard;
    fn write(&'a self) -> Self::WriteGuard;
}

impl<'a, T> Lock<'a, T> for RefCell<T>
where
    T: Default + 'a,
{
    type ReadGuard = Ref<'a, T>;
    type WriteGuard = RefMut<'a, T>;

    fn read(&'a self) -> Self::ReadGuard {
        self.borrow()
    }

    fn write(&'a self) -> Self::WriteGuard {
        self.borrow_mut()
    }
}

impl<'a, T> Lock<'a, T> for Mutex<T>
where
    T: Default + 'a,
{
    type ReadGuard = MutexGuard<'a, T>;
    type WriteGuard = MutexGuard<'a, T>;

    fn read(&'a self) -> Self::ReadGuard {
        self.lock().expect("poisoned")
    }

    fn write(&'a self) -> Self::WriteGuard {
        self.lock().expect("poisoned")
    }
}

impl<'a, T> Lock<'a, T> for RwLock<T>
where
    T: Default + 'a,
{
    type ReadGuard = RwLockReadGuard<'a, T>;
    type WriteGuard = RwLockWriteGuard<'a, T>;

    fn read(&'a self) -> Self::ReadGuard {
        self.read().expect("poisoned")
    }

    fn write(&'a self) -> Self::WriteGuard {
        self.write().expect("poisoned")
    }
}

#[cfg(feature = "parking_lot")]
impl<'a, T> Lock<'a, T> for parking_lot::Mutex<T>
where
    T: Default + 'a,
{
    type ReadGuard = parking_lot::MutexGuard<'a, T>;
    type WriteGuard = parking_lot::MutexGuard<'a, T>;

    fn read(&'a self) -> Self::ReadGuard {
        self.lock()
    }

    fn write(&'a self) -> Self::WriteGuard {
        self.lock()
    }
}

#[cfg(feature = "parking_lot")]
impl<'a, T> Lock<'a, T> for parking_lot::RwLock<T>
where
    T: Default + 'a,
{
    type ReadGuard = parking_lot::RwLockReadGuard<'a, T>;
    type WriteGuard = parking_lot::RwLockWriteGuard<'a, T>;

    fn read(&'a self) -> Self::ReadGuard {
        self.read()
    }

    fn write(&'a self) -> Self::WriteGuard {
        self.write()
    }
}
