use std::{
    cell::{Ref, RefCell, RefMut},
    ops::{Deref, DerefMut},
    sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

/// A interior mutable type that can hand out read and write guards to its underlying data.
#[async_trait::async_trait]
pub trait LockAsync<'a, T>: Default + From<T> {
    type ReadGuard: Deref<Target = T>;
    type WriteGuard: DerefMut<Target = T>;

    async fn read(&'a self) -> Self::ReadGuard;
    async fn write(&'a self) -> Self::WriteGuard;
}

#[cfg(feature = "async_std")]
#[async_trait::async_trait]
impl<'a, T> LockAsync<'a, T> for async_std::sync::Mutex<T>
where
    T: Default + 'a,
{
    type ReadGuard = async_std::sync::MutexGuard<'a, T>;
    type WriteGuard = async_std::sync::MutexGuard<'a, T>;

    async fn read(&'a self) -> Self::ReadGuard {
        self.lock()
    }

    async fn write(&'a self) -> Self::WriteGuard {
        self.lock()
    }
}

#[cfg(feature = "async_std")]
#[async_trait::async_trait]
impl<'a, T> LockAsync<'a, T> for async_std::sync::RwLock<T>
where
    T: Default + 'a,
{
    type ReadGuard = async_std::sync::RwLockReadGuard<'a, T>;
    type WriteGuard = async_std::sync::RwLockWriteGuard<'a, T>;

    async fn read(&'a self) -> Self::ReadGuard {
        self.lock()
    }

    async fn write(&'a self) -> Self::WriteGuard {
        self.lock()
    }
}

