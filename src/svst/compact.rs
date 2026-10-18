//! Variants of the collections whose nodes refer to each other with `u32` indices instead of `usize`.
//! They use less memory, but can hold fewer than `u32::MAX` values.

use crate::svst::aa;

pub type AATreeSet<KeyType> = aa::Set<KeyType, u32>;
pub type AATreeMap<KeyType, MappedType> = aa::Map<KeyType, MappedType, u32>;
pub type AAVecSet<KeyType> = aa::VecSet<KeyType, u32>;
pub type AAVecMap<KeyType, MappedType> = aa::VecMap<KeyType, MappedType, u32>;

#[test]
fn test_compact_map()
{
	let mut map = AAVecMap::<u64, &str>::new();
	assert_eq!(None, map.insert(2, "b"));
	assert_eq!(None, map.insert(1, "a"));
	assert_eq!(Some("b"), map.insert(2, "B"));
	assert_eq!(Some((&1, &"a")), map.first_key_value());
	assert_eq!(Some("B"), map.remove(&2));
	assert_eq!(1, map.len());
}
