//! Vector-storage collections.

pub mod aa;
pub use repository::Repository;
pub type AATreeSet<KeyType> = aa::Set<KeyType>;
pub type AATreeMap<KeyType, MappedType> = aa::Map<KeyType, MappedType>;

mod repository;
mod vector_storage;
mod bit_indexing;
