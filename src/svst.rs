//! Single vector-storage collections.

pub mod aa;
pub mod svec;
pub use repository::Repository;

mod repository;
mod vector_storage;
mod bit_indexing;

pub type SVec<Type, const SIZE: usize> = svec::SVec<Type, SIZE>;
pub type AATreeSet<KeyType, Compare = crate::DefaultComparator> = aa::Set<KeyType, Compare>;
pub type AATreeMap<KeyType, MappedType, Compare = crate::DefaultComparator> = aa::Map<KeyType, MappedType, Compare>;
