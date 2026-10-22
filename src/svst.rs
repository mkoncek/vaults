//! Collections backed by contiguous storage.
//!
//! All structures in this module store their elements in flat arrays rather than
//! pointer-linked nodes, improving cache locality and reducing allocation overhead.

pub mod aa;
pub mod svec;
pub use repository::Repository;

mod repository;
mod vector_storage;
mod bit_indexing;

pub type SVec<Type, const SIZE: usize> = svec::SVec<Type, SIZE>;
pub type AATreeSet<KeyType> = aa::Set<KeyType>;
pub type AATreeMap<KeyType, MappedType> = aa::Map<KeyType, MappedType>;
pub type AAVecSet<KeyType> = aa::VecSet<KeyType>;
pub type AAVecMap<KeyType, MappedType> = aa::VecMap<KeyType, MappedType>;
