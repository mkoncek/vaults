use crate::svst::aa;
use crate::svst::aa::node;

#[derive(Debug)]
pub struct MapEntry<KeyType, MappedType>(KeyType, MappedType);

impl<KeyType, MappedType> node::Entry for MapEntry<KeyType, MappedType>
{
	type Key = KeyType;
	type Value = (KeyType, MappedType);
	fn key(&self) -> &Self::Key {&self.0}
	fn value(self) -> Self::Value {(self.0, self.1)}
}

pub type Map<KeyType, MappedType, Compare = crate::DefaultComparator> = aa::tree::Tree<MapEntry<KeyType, MappedType>, Compare>;

impl<KeyType, MappedType, Compare> Map<KeyType, MappedType, Compare>
{
	pub fn first_key_value(&self) -> Option<(&KeyType, &MappedType)> {self.impl_first().map(|v| (&v.0, &v.1))}
	pub fn last_key_value(&self) -> Option<(&KeyType, &MappedType)> {self.impl_last().map(|v| (&v.0, &v.1))}
	
	pub fn contains_key<Key>(&self, key: &Key) -> bool
	where
		KeyType: std::borrow::Borrow<Key>,
		Key: ?Sized,
		Compare: crate::Comparator<Key>,
	{
		node::AA::find(unsafe {self.repository.as_slice()}, self.root, key, &self.compare).0 != usize::MAX
	}
	
	pub fn get<Key>(&self, key: &Key) -> Option<&MappedType>
	where
		KeyType: std::borrow::Borrow<Key>,
		Key: ?Sized,
		Compare: crate::Comparator<Key>,
	{
		self.impl_get(key).map(|v| &v.1)
	}
	
	pub fn get_mut<Key>(&mut self, key: &Key) -> Option<&mut MappedType>
	where
		KeyType: std::borrow::Borrow<Key>,
		Key: ?Sized,
		Compare: crate::Comparator<Key>,
	{
		let index = node::AA::find(unsafe {self.repository.as_mut_slice()}, self.root, key, &self.compare).0;
		
		if index != usize::MAX
		{
			return Some(&mut self.repository[index].as_mut().1);
		}
		
		return None;
	}
	
	pub fn get_key_value<Key>(&self, key: &Key) -> Option<(&KeyType, &MappedType)>
	where
		KeyType: std::borrow::Borrow<Key>,
		Key: ?Sized,
		Compare: crate::Comparator<Key>,
	{
		self.impl_get(key).map(|v| (&v.0, &v.1))
	}
	
	pub fn insert(&mut self, key: KeyType, mapped: MappedType) -> Option<MappedType>
	where
		Compare: crate::Comparator<KeyType>,
	{
		self.try_insert(MapEntry {0: key, 1: mapped}, |v| v.map(|v| v.1))
	}
	
	pub fn remove<Key>(&mut self, key: &Key) -> Option<MappedType>
	where
		KeyType: std::borrow::Borrow<Key>,
		Key: ?Sized,
		Compare: crate::Comparator<Key>,
	{
		return self.remove_entry(key).map(|v| v.1);
	}
	
	pub fn remove_entry<Key>(&mut self, key: &Key) -> Option<(KeyType, MappedType)>
	where
		KeyType: std::borrow::Borrow<Key>,
		Key: ?Sized,
		Compare: crate::Comparator<Key>,
	{
		let index = aa::node::AA::find(unsafe {self.repository.as_slice()}, self.root, key, &self.compare).0;
		
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

impl<Key, KeyType, MappedType, Compare> std::ops::Index<&Key> for Map<KeyType, MappedType, Compare>
where
	KeyType: std::borrow::Borrow<Key>,
	Key: ?Sized,
	Compare: crate::Comparator<Key>,
{
	type Output = MappedType;
	
	fn index(&self, index: &Key) -> &Self::Output
	{
		&self.impl_get(index).expect("no entry found for key").1
	}
}
