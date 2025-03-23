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

pub type Map<KeyType, MappedType> = aa::tree::Tree<MapEntry<KeyType, MappedType>>;

impl<KeyType, MappedType> Map<KeyType, MappedType>
{
	pub fn first_key_value(&self) -> Option<(&KeyType, &MappedType)> {self.impl_first().map(|v| (&v.0, &v.1))}
	pub fn last_key_value(&self) -> Option<(&KeyType, &MappedType)> {self.impl_last().map(|v| (&v.0, &v.1))}
	
	pub unsafe fn contains_key_with_comparator<Key, Compare>(&self, key: &Key, compare: Compare) -> bool
	where
		KeyType: std::borrow::Borrow<Key>,
		Key: ?Sized,
		Compare: crate::Comparator<Key>,
	{
		node::AA::find(unsafe {self.repository.as_slice()}, self.root, key, compare).0 != usize::MAX
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
		let index = node::AA::find(unsafe {self.repository.as_mut_slice()}, self.root, key, compare).0;
		
		if index != usize::MAX
		{
			return Some(&mut self.repository[index].as_mut().1);
		}
		
		return None;
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
		let index = aa::node::AA::find(unsafe {self.repository.as_slice()}, self.root, key, compare).0;
		
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

impl<Key, KeyType, MappedType> std::ops::Index<&Key> for Map<KeyType, MappedType>
where
	KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
	Key: ?Sized + std::cmp::Ord,
{
	type Output = MappedType;
	
	fn index(&self, index: &Key) -> &Self::Output
	{
		&self.impl_get(index, crate::DefaultComparator::new()).expect("no entry found for key").1
	}
}
