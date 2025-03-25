use crate::svst::aa;
use crate::svst::aa::node;
use crate::svst::aa::tree::{TreeStorage, RemovableTreeStorage};

#[derive(Debug)]
pub struct MapEntry<KeyType, MappedType>(KeyType, MappedType);

impl<KeyType, MappedType> node::Entry for MapEntry<KeyType, MappedType>
{
	type Key = KeyType;
	type Value = (KeyType, MappedType);
	fn key(&self) -> &Self::Key {&self.0}
	fn value(self) -> Self::Value {(self.0, self.1)}
}

pub type Map<KeyType, MappedType> = aa::tree::Tree<MapEntry<KeyType, MappedType>>;
pub type VecMap<KeyType, MappedType> = aa::tree::Tree<MapEntry<KeyType, MappedType>, Vec<node::Node<MapEntry<KeyType, MappedType>>>>;

impl<KeyType, MappedType, Storage> aa::tree::Tree<MapEntry<KeyType, MappedType>, Storage>
where
	Storage: TreeStorage<MapEntry<KeyType, MappedType>>,
{
	pub fn first_key_value(&self) -> Option<(&KeyType, &MappedType)> {self.impl_first().map(|v| (&v.0, &v.1))}
	pub fn last_key_value(&self) -> Option<(&KeyType, &MappedType)> {self.impl_last().map(|v| (&v.0, &v.1))}
	
	pub unsafe fn contains_key_with_comparator<Key, Compare>(&self, key: &Key, compare: Compare) -> bool
	where
		KeyType: std::borrow::Borrow<Key>,
		Key: ?Sized,
		Compare: crate::Comparator<Key>,
	{
		self.impl_find(key, compare) != usize::MAX
	}
	
	pub unsafe fn get_with_comparator<Key, Compare>(&self, key: &Key, compare: Compare) -> Option<&MappedType>
	where
		KeyType: std::borrow::Borrow<Key>,
		Key: ?Sized,
		Compare: crate::Comparator<Key>,
	{
		self.impl_get(key, compare).map(|v| &v.1)
	}
	
	pub unsafe fn get_mut_with_comparator<Key, Compare>(&mut self, key: &Key, compare: Compare) -> Option<&mut MappedType>
	where
		KeyType: std::borrow::Borrow<Key>,
		Key: ?Sized,
		Compare: crate::Comparator<Key>,
	{
		self.impl_get_mut(key, compare).map(|v| &mut v.1)
	}
	
	pub unsafe fn get_key_value_with_comparator<Key, Compare>(&self, key: &Key, compare: Compare) -> Option<(&KeyType, &MappedType)>
	where
		KeyType: std::borrow::Borrow<Key>,
		Key: ?Sized,
		Compare: crate::Comparator<Key>,
	{
		self.impl_get(key, compare).map(|v| (&v.0, &v.1))
	}
	
	pub unsafe fn insert_with_comparator<Compare>(&mut self, key: KeyType, mapped: MappedType, compare: Compare) -> Option<MappedType>
	where
		Compare: crate::Comparator<KeyType>,
	{
		self.try_insert(MapEntry {0: key, 1: mapped}, compare, |v| v.map(|v| v.1))
	}
	
	pub fn contains_key<Key>(&self, key: &Key) -> bool
	where
		KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
		Key: ?Sized + std::cmp::Ord,
	{
		unsafe {self.contains_key_with_comparator(key, crate::DefaultComparator::new())}
	}
	
	pub fn get<Key>(&self, key: &Key) -> Option<&MappedType>
	where
		KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
		Key: ?Sized + std::cmp::Ord,
	{
		unsafe {self.get_with_comparator(key, crate::DefaultComparator::new())}
	}
	
	pub fn get_mut<Key>(&mut self, key: &Key) -> Option<&mut MappedType>
	where
		KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
		Key: ?Sized + std::cmp::Ord,
	{
		unsafe {self.get_mut_with_comparator(key, crate::DefaultComparator::new())}
	}
	
	pub fn get_key_value<Key>(&self, key: &Key) -> Option<(&KeyType, &MappedType)>
	where
		KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
		Key: ?Sized + std::cmp::Ord,
	{
		unsafe {self.get_key_value_with_comparator(key, crate::DefaultComparator::new())}
	}
	
	pub fn insert(&mut self, key: KeyType, mapped: MappedType) -> Option<MappedType>
	where
		KeyType: std::cmp::Ord,
	{
		unsafe {self.insert_with_comparator(key, mapped, crate::DefaultComparator::new())}
	}
	
	pub unsafe fn get_at_unchecked(&self, position: usize) -> &MappedType
	{
		&self.impl_get_at_unchecked(position).1
	}
	
	pub unsafe fn get_at_unchecked_mut(&mut self, position: usize) -> &mut MappedType
	{
		&mut self.impl_get_at_unchecked_mut(position).1
	}
	
	pub unsafe fn get_key_value_at_unchecked(&self, position: usize) -> (&KeyType, &MappedType)
	{
		let result = self.impl_get_at_unchecked(position);
		return (&result.0, &result.1);
	}
	
	pub unsafe fn get_key_value_at_unchecked_mut(&mut self, position: usize) -> (&KeyType, &mut MappedType)
	{
		let result = self.impl_get_at_unchecked_mut(position);
		return (&result.0, &mut result.1);
	}
}

impl<KeyType, MappedType, Storage> aa::tree::Tree<MapEntry<KeyType, MappedType>, Storage>
where
	Storage: RemovableTreeStorage<MapEntry<KeyType, MappedType>>,
{
	pub unsafe fn remove_with_comparator<Key, Compare>(&mut self, key: &Key, compare: Compare) -> Option<MappedType>
	where
		KeyType: std::borrow::Borrow<Key>,
		Key: ?Sized,
		Compare: crate::Comparator<Key>,
	{
		return self.remove_entry_with_comparator(key, compare).map(|v| v.1);
	}
	
	pub unsafe fn remove_entry_with_comparator<Key, Compare>(&mut self, key: &Key, compare: Compare) -> Option<(KeyType, MappedType)>
	where
		KeyType: std::borrow::Borrow<Key>,
		Key: ?Sized,
		Compare: crate::Comparator<Key>,
	{
		let index = self.impl_find(key, compare);
		
		if index != usize::MAX
		{
			return self.remove_at(index);
		}
		
		return None;
	}
	
	pub fn retain<Function>(&mut self, mut function: Function)
	where
		Function: std::ops::FnMut(&KeyType, &mut MappedType) -> bool,
	{
		self.impl_retain(move |v| function(&v.0, &mut v.1));
	}
	
	pub fn remove<Key>(&mut self, key: &Key) -> Option<MappedType>
	where
		KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
		Key: ?Sized + std::cmp::Ord,
	{
		unsafe {self.remove_with_comparator(key, crate::DefaultComparator::new())}
	}
	
	pub fn remove_entry<Key>(&mut self, key: &Key) -> Option<(KeyType, MappedType)>
	where
		KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
		Key: ?Sized + std::cmp::Ord,
	{
		unsafe {self.remove_entry_with_comparator(key, crate::DefaultComparator::new())}
	}
}

impl<Key, KeyType, MappedType, Storage> std::ops::Index<&Key> for aa::tree::Tree<MapEntry<KeyType, MappedType>, Storage>
where
	KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
	Key: ?Sized + std::cmp::Ord,
	Storage: TreeStorage<MapEntry<KeyType, MappedType>>,
{
	type Output = MappedType;
	
	fn index(&self, index: &Key) -> &Self::Output
	{
		self.get(index).expect("no entry found for key")
	}
}

#[test]
fn test_aa_vec_map()
{
	let mut map = VecMap::<String, i32>::new();
	assert_eq!(None, map.insert("b".to_owned(), 2));
	assert_eq!(None, map.insert("a".to_owned(), 1));
	assert_eq!(None, map.insert("c".to_owned(), 3));
	assert_eq!(Some(2), map.insert("b".to_owned(), 20));
	
	assert_eq!(3, map.len());
	assert_eq!(20, map["b"]);
	assert_eq!(None, map.get("d"));
	*map.get_mut("a").unwrap() += 10;
	assert_eq!(Some((&"a".to_owned(), &11)), map.first_key_value());
	assert_eq!(Some((&"c".to_owned(), &3)), map.last_key_value());
	
	assert_eq!(Some(11), map.remove("a"));
	assert_eq!(Some(("c".to_owned(), 3)), map.remove_entry("c"));
	assert_eq!(None, map.remove("c"));
	assert_eq!(Some((&"b".to_owned(), &20)), map.first_key_value());
	assert_eq!(1, map.len());
}
