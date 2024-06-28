



pub type Mutex<T> = std::sync::Mutex<T>;
pub type MutexGruad<'a, T> = std::sync::MutexGuard<'a, T>;

pub type Condvar = std::sync::Condvar;

pub type RwLock<T> = std::sync::RwLock<T>;
pub type RwLockReadGuard<'a, T> = std::sync::RwLockReadGuard<'a, T>;
pub type RwLockWriteGuard<'a, T> = std::sync::RwLockWriteGuard<'a, T>;
