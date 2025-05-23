pub mod builtins;
pub mod formats;
pub mod runner;
// pub mod state;
pub mod traits;
// pub mod logging;

#[cfg(feature = "opt")]
type Hasher = std::hash::BuildHasherDefault<rustc_hash::FxHasher>;

#[cfg(not(feature = "opt"))]
type Hasher = std::collections::hash_map::RandomState;

// #[cfg(feature = "opt")]
// pub type BTreeMap<K, V> = btree_slab::BTreeMap<K, V>;
//
// #[cfg(not(feature = "opt"))]
pub type BTreeMap<K, V> = std::collections::BTreeMap<K, V>;

pub type HashMap<K, V> = std::collections::HashMap<K, V, Hasher>;

#[cfg(feature = "opt")]
pub type SmolHashMap<K, V> = halfbrown::HashMap<K, V, Hasher>;
#[cfg(not(feature = "opt"))]
pub type SmolHashMap<K, V> = std::collections::HashMap<K, V, Hasher>;

#[cfg(feature = "opt")]
pub type SmolVec<T> = smallvec::SmallVec<T>;
#[cfg(not(feature = "opt"))]
pub type SmolVec<T> = std::vec::Vec<T>;

#[cfg(feature = "opt")]
pub type SmolStr = smol_str::SmolStr;
#[cfg(not(feature = "opt"))]
pub type SmolStr = std::string::String;

#[cfg(feature = "opt")]
pub type ArrayVec<T, const N: usize> = arrayvec::ArrayVec<T, N>;
#[cfg(not(feature = "opt"))]
pub type ArrayVec<T, const N: usize> = std::vec::Vec<T>;

pub mod prelude {
    pub use macros::data;
    pub use macros::unwrap;
    pub use macros::DeepClone;
    pub use macros::New;
    pub use macros::Send;
    pub use macros::Sync;
    pub use macros::Timestamp;
    pub use macros::Unpin;

    pub use crate::builtins::duration::Duration;
    pub use crate::builtins::format::Format;
    pub use crate::builtins::window::Window;
    pub use crate::builtins::keyed_stream::KeyedStream;
    pub use crate::builtins::reader::Reader;
    pub use crate::builtins::stream::Stream;
    pub use crate::builtins::time::Time;
    pub use crate::builtins::writer::Writer;
    pub use crate::traits::Data;
    pub use crate::traits::DeepClone;

    pub use crate::runner::context::Context;
    pub use crate::runner::current_thread::CurrentThreadRunner;
    pub use crate::runner::data_parallel::DataParallelRunner;
    pub use crate::runner::task_parallel::TaskParallelRunner;

    pub use serde;
    pub use tokio;

    pub use crate::builtins::stream;
    pub use crate::formats;
}
