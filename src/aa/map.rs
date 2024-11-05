use crate::aa;
use crate::aa::node;

pub struct MapEntry<KeyType, MappedType>(KeyType, MappedType);

impl<KeyType, MappedType> aa::node::Entry for MapEntry<KeyType, MappedType>
{
	type Key = KeyType;
	type Value = (KeyType, MappedType);
	fn key(&self) -> &Self::Key {&self.0}
	fn value(self) -> Self::Value {(self.0, self.1)}
}

pub type Map<KeyType, MappedType> = aa::tree::Tree<MapEntry<KeyType, MappedType>>;

impl<KeyType, MappedType> Map<KeyType, MappedType>
{
	pub fn first_key_value(&self) -> Option<(&KeyType, &MappedType)> {self.impl_first().map(|v| (&v.0, &v.1))}
	pub fn last_key_value(&self) -> Option<(&KeyType, &MappedType)> {self.impl_last().map(|v| (&v.0, &v.1))}
	
	pub fn contains_key<Key>(&self, key: &Key) -> bool
	where
		KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
		Key: ?Sized + std::cmp::Ord,
	{
		aa::node::find(unsafe {self.repository.as_slice()}, self.root, key).0 != usize::MAX
	}
	
	pub fn get<Key>(&self, key: &Key) -> Option<&MappedType>
	where
		KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
		Key: ?Sized + std::cmp::Ord,
	{
		self.impl_get(key).map(|v| &v.1)
	}
	
	pub fn get_mut<Key>(&mut self, key: &Key) -> Option<&mut MappedType>
	where
		KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
		Key: ?Sized + std::cmp::Ord,
	{
		let index = node::find(unsafe {self.repository.as_mut_slice()}, self.root, key).0;
		
		if index != usize::MAX
		{
			return Some(&mut self.repository[index].as_mut().1);
		}
		
		return None;
	}
	
	pub fn get_key_value<Key>(&self, key: &Key) -> Option<(&KeyType, &MappedType)>
	where
		KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
		Key: ?Sized + std::cmp::Ord,
	{
		self.impl_get(key).map(|v| (&v.0, &v.1))
	}
	
	pub fn insert(&mut self, key: KeyType, mapped: MappedType) -> Option<MappedType>
	where
		KeyType: std::cmp::Ord
	{
		self.try_insert(MapEntry {0: key, 1: mapped}, |v| v.map(|v| v.1))
	}
	
	pub fn remove<Key>(&mut self, key: &Key) -> Option<MappedType>
	where
		KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
		Key: ?Sized + std::cmp::Ord,
	{
		return self.remove_entry(key).map(|v| v.1);
	}
	
	pub fn remove_entry<Key>(&mut self, key: &Key) -> Option<(KeyType, MappedType)>
	where
		KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
		Key: ?Sized + std::cmp::Ord,
	{
		let index = aa::node::find(unsafe {self.repository.as_slice()}, self.root, key).0;
		
		if index != usize::MAX
		{
			return self.remove_at(index);
		}
		
		return None;
	}
	
	pub fn retain<Function>(&mut self, mut function: Function)
	where
		KeyType: std::cmp::Ord,
		Function: std::ops::FnMut(&KeyType, &mut MappedType) -> bool,
	{
		self.impl_retain(move |v| function(&v.0, &mut v.1));
	}
}
