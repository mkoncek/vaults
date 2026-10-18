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

pub type Map<KeyType, MappedType, LinkType = usize> = aa::tree::Tree<MapEntry<KeyType, MappedType>, crate::svst::Repository<node::Node<MapEntry<KeyType, MappedType>, LinkType>>>;
pub type VecMap<KeyType, MappedType, LinkType = usize> = aa::tree::Tree<MapEntry<KeyType, MappedType>, Vec<node::Node<MapEntry<KeyType, MappedType>, LinkType>>>;

impl<KeyType, MappedType, Storage> aa::tree::Tree<MapEntry<KeyType, MappedType>, Storage>
where
	Storage: TreeStorage<MapEntry<KeyType, MappedType>>,
{
	pub fn first_key_value(&self) -> Option<(&KeyType, &MappedType)> {self.impl_first().map(|v| (&v.0, &v.1))}
	pub fn last_key_value(&self) -> Option<(&KeyType, &MappedType)> {self.impl_last().map(|v| (&v.0, &v.1))}
	
	/// Like [`contains_key`](Self::contains_key), but searches using `compare`,
	/// which returns the ordering of a key in the map relative to the searched one.
	/// 
	/// The ordering must be consistent with the one used for insertion, otherwise the result is unspecified.
	pub fn contains_key_by<Compare>(&self, compare: Compare) -> bool
	where
		Compare: FnMut(&KeyType) -> std::cmp::Ordering,
	{
		self.impl_find(compare) != node::Link::NIL
	}
	
	/// Like [`get`](Self::get), but searches using `compare`, see [`contains_key_by`](Self::contains_key_by).
	pub fn get_by<Compare>(&self, compare: Compare) -> Option<&MappedType>
	where
		Compare: FnMut(&KeyType) -> std::cmp::Ordering,
	{
		self.impl_get(compare).map(|v| &v.1)
	}
	
	/// Like [`get_mut`](Self::get_mut), but searches using `compare`, see [`contains_key_by`](Self::contains_key_by).
	pub fn get_mut_by<Compare>(&mut self, compare: Compare) -> Option<&mut MappedType>
	where
		Compare: FnMut(&KeyType) -> std::cmp::Ordering,
	{
		self.impl_get_mut(compare).map(|v| &mut v.1)
	}
	
	/// Like [`get_key_value`](Self::get_key_value), but searches using `compare`, see [`contains_key_by`](Self::contains_key_by).
	pub fn get_key_value_by<Compare>(&self, compare: Compare) -> Option<(&KeyType, &MappedType)>
	where
		Compare: FnMut(&KeyType) -> std::cmp::Ordering,
	{
		self.impl_get(compare).map(|v| (&v.0, &v.1))
	}
	
	/// Like [`insert`](Self::insert), but orders the keys using `compare`.
	/// 
	/// The ordering must be consistent for all operations on the map, otherwise their results are unspecified.
	pub fn insert_by<Compare>(&mut self, key: KeyType, mapped: MappedType, compare: Compare) -> Option<MappedType>
	where
		Compare: FnMut(&KeyType, &KeyType) -> std::cmp::Ordering,
	{
		self.try_insert(MapEntry {0: key, 1: mapped}, compare, |existing| existing.map(|(existing, value)| std::mem::replace(&mut existing.1, value.1)))
	}
	
	pub fn contains_key<Key>(&self, key: &Key) -> bool
	where
		KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
		Key: ?Sized + std::cmp::Ord,
	{
		self.contains_key_by(|k| k.borrow().cmp(key))
	}
	
	pub fn get<Key>(&self, key: &Key) -> Option<&MappedType>
	where
		KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
		Key: ?Sized + std::cmp::Ord,
	{
		self.get_by(|k| k.borrow().cmp(key))
	}
	
	pub fn get_mut<Key>(&mut self, key: &Key) -> Option<&mut MappedType>
	where
		KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
		Key: ?Sized + std::cmp::Ord,
	{
		self.get_mut_by(|k| k.borrow().cmp(key))
	}
	
	pub fn get_key_value<Key>(&self, key: &Key) -> Option<(&KeyType, &MappedType)>
	where
		KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
		Key: ?Sized + std::cmp::Ord,
	{
		self.get_key_value_by(|k| k.borrow().cmp(key))
	}
	
	/// Inserts a key-value pair into the map.
	/// If the map already contains an equal key, only its value is updated and the old value is returned.
	pub fn insert(&mut self, key: KeyType, mapped: MappedType) -> Option<MappedType>
	where
		KeyType: std::cmp::Ord,
	{
		self.insert_by(key, mapped, Ord::cmp)
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
	/// Like [`remove`](Self::remove), but searches using `compare`, see [`contains_key_by`](Self::contains_key_by).
	pub fn remove_by<Compare>(&mut self, compare: Compare) -> Option<MappedType>
	where
		Compare: FnMut(&KeyType) -> std::cmp::Ordering,
	{
		return self.remove_entry_by(compare).map(|v| v.1);
	}
	
	/// Like [`remove_entry`](Self::remove_entry), but searches using `compare`, see [`contains_key_by`](Self::contains_key_by).
	pub fn remove_entry_by<Compare>(&mut self, compare: Compare) -> Option<(KeyType, MappedType)>
	where
		Compare: FnMut(&KeyType) -> std::cmp::Ordering,
	{
		let index = self.impl_find(compare);
		
		if index != node::Link::NIL
		{
			return self.remove_at(index.index());
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
		self.remove_by(|k| k.borrow().cmp(key))
	}
	
	pub fn remove_entry<Key>(&mut self, key: &Key) -> Option<(KeyType, MappedType)>
	where
		KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
		Key: ?Sized + std::cmp::Ord,
	{
		self.remove_entry_by(|k| k.borrow().cmp(key))
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
	
	assert_eq!(Some((&"c".to_owned(), &3)), map.get_key_value_by(|k| k.as_str().cmp("c")));
	assert_eq!(None, map.insert_by("d".to_owned(), 4, |lhs, rhs| lhs.len().cmp(&rhs.len()).then(lhs.cmp(rhs))));
	assert_eq!(Some(4), map.remove_by(|k| k.as_str().cmp("d")));
	
	assert_eq!(Some(11), map.remove("a"));
	assert_eq!(Some(("c".to_owned(), 3)), map.remove_entry("c"));
	assert_eq!(None, map.remove("c"));
	assert_eq!(Some((&"b".to_owned(), &20)), map.first_key_value());
	assert_eq!(1, map.len());
}

#[test]
fn test_aa_map_insert_keeps_key()
{
	// Keys equal by the ordering but distinguishable, only the value is updated
	let by_lowercase = |lhs: &String, rhs: &String| lhs.to_lowercase().cmp(&rhs.to_lowercase());
	let mut map = Map::<String, i32>::new();
	assert_eq!(None, map.insert_by("Key".to_owned(), 1, by_lowercase));
	assert_eq!(Some(1), map.insert_by("KEY".to_owned(), 2, by_lowercase));
	assert_eq!(1, map.len());
	assert_eq!(Some((&"Key".to_owned(), &2)), map.first_key_value());
}
