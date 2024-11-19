//! Single vector-storage collections.

pub mod aa;
pub use repository::Repository;

mod repository;
mod vector_storage;
mod bit_indexing;

pub type AATreeSet<KeyType> = aa::Set<KeyType>;
pub type AATreeMap<KeyType, MappedType> = aa::Map<KeyType, MappedType>;
