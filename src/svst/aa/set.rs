use crate::svst::aa;
use crate::svst::aa::node;
use crate::svst::aa::tree::{TreeStorage, RemovableTreeStorage};

#[derive(Debug)]
pub struct SetEntry<KeyType>(KeyType);

impl<KeyType> node::Entry for SetEntry<KeyType>
{
	type Key = KeyType;
	type Value = KeyType;
	fn key(&self) -> &Self::Key {&self.0}
	fn value(self) -> Self::Value {self.0}
}

pub type Set<KeyType> = aa::tree::Tree<SetEntry<KeyType>>;
pub type VecSet<KeyType> = aa::tree::Tree<SetEntry<KeyType>, Vec<node::Node<SetEntry<KeyType>>>>;

impl<KeyType, Storage> aa::tree::Tree<SetEntry<KeyType>, Storage>
where
	Storage: TreeStorage<SetEntry<KeyType>>,
{
	pub fn first(&self) -> Option<&KeyType> {self.impl_first().map(|k| &k.0)}
	pub fn last(&self) -> Option<&KeyType> {self.impl_last().map(|k| &k.0)}
	
	pub unsafe fn contains_with_comparator<Key, Compare>(&self, key: &Key, compare: Compare) -> bool
	where
		KeyType: std::borrow::Borrow<Key>,
		Key: ?Sized,
		Compare: crate::Comparator<Key>,
	{
		self.impl_find(key, compare) != usize::MAX
	}
	
	pub unsafe fn get_with_comparator<Key, Compare>(&self, key: &Key, compare: Compare) -> Option<&KeyType>
	where
		KeyType: std::borrow::Borrow<Key>,
		Key: ?Sized,
		Compare: crate::Comparator<Key>,
	{
		self.impl_get(key, compare).map(|k| &k.0)
	}
	
	pub unsafe fn insert_with_comparator<Compare>(&mut self, value: KeyType, compare: Compare) -> bool
	where
		Compare: crate::Comparator<KeyType>,
	{
		self.try_insert(SetEntry {0: value}, compare, |v| v.is_none())
	}
	
	pub unsafe fn replace_with_comparator<Compare>(&mut self, value: KeyType, compare: Compare) -> Option<KeyType>
	where
		Compare: crate::Comparator<KeyType>,
	{
		self.try_insert(SetEntry {0: value}, compare, |v| v.map(|v| v.0))
	}
	
	pub fn iter<'t>(&'t self) -> node::Iterator<&'t [node::Node<SetEntry<KeyType>>]>
	{
		self.impl_iter()
	}
	
	pub fn contains<Key>(&self, key: &Key) -> bool
	where
		KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
		Key: ?Sized + std::cmp::Ord,
	{
		unsafe {self.contains_with_comparator(key, crate::DefaultComparator::new())}
	}
	
	pub fn get<Key>(&self, key: &Key) -> Option<&KeyType>
	where
		KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
		Key: ?Sized + std::cmp::Ord,
	{
		unsafe {self.get_with_comparator(key, crate::DefaultComparator::new())}
	}
	
	pub fn insert(&mut self, value: KeyType) -> bool
	where
		KeyType: std::cmp::Ord,
	{
		unsafe {self.insert_with_comparator(value, crate::DefaultComparator::new())}
	}
	
	pub fn replace(&mut self, value: KeyType) -> Option<KeyType>
	where
		KeyType: std::cmp::Ord,
	{
		unsafe {self.replace_with_comparator(value, crate::DefaultComparator::new())}
	}
	
	pub unsafe fn get_at_unchecked(&self, position: usize) -> &KeyType
	{
		&self.impl_get_at_unchecked(position).0
	}
}

impl<KeyType, Storage> aa::tree::Tree<SetEntry<KeyType>, Storage>
where
	Storage: RemovableTreeStorage<SetEntry<KeyType>>,
{
	pub unsafe fn remove_with_comparator<Key, Compare>(&mut self, value: &Key, compare: Compare) -> bool
	where
		KeyType: std::borrow::Borrow<Key>,
		Key: ?Sized,
		Compare: crate::Comparator<Key>,
	{
		let index = self.impl_find(value, compare);
		
		if index != usize::MAX
		{
			self.remove_at(index);
			return true;
		}
		
		return false;
	}
	
	pub fn retain<Function>(&mut self, mut function: Function)
	where
		Function: std::ops::FnMut(&KeyType) -> bool,
	{
		self.impl_retain(move |k| function(&k.0));
	}
	
	pub fn remove<Key>(&mut self, value: &Key) -> bool
	where
		KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
		Key: ?Sized + std::cmp::Ord,
	{
		unsafe {self.remove_with_comparator(value, crate::DefaultComparator::new())}
	}
}

impl<'t, Type> std::iter::Iterator for node::Iterator<&'t [node::Node<SetEntry<Type>>]>
{
	type Item = &'t Type;
	
	fn next(&mut self) -> Option<Self::Item>
	{
		match node::iter_impl!(self, 0)
		{
			usize::MAX => None,
			i => Some(&self.nodes[i].as_ref().0),
		}
	}
}

impl<'t, Type> std::iter::DoubleEndedIterator for node::Iterator<&'t [node::Node<SetEntry<Type>>]>
{
	fn next_back(&mut self) -> Option<Self::Item>
	{
		match node::iter_impl!(self, 1)
		{
			usize::MAX => None,
			i => Some(&self.nodes[i].as_ref().0),
		}
	}
}

impl<'t, Type, Storage> std::iter::IntoIterator for &'t aa::tree::Tree<SetEntry<Type>, Storage>
where
	Storage: TreeStorage<SetEntry<Type>>,
{
	type Item = &'t Type;
	type IntoIter = node::Iterator<&'t [node::Node<SetEntry<Type>>]>;
	
	fn into_iter(self) -> Self::IntoIter
	{
		self.iter()
	}
}

#[test]
fn test_aa_set_0()
{
	let set = Set::<i32>::new();
	
	assert_eq!(0, set.len());
	
	assert_eq!(0, set.iter().count());
	assert_eq!(None, set.iter().next());
	assert_eq!(None, set.iter().rev().next());
}

#[test]
fn test_aa_set_1()
{
	let mut set = Set::<i32>::new();
	set.insert(7);
	set.insert(8);
	set.insert(11);
	set.insert(12);
	set.insert(9);
	set.insert(10);
	set.insert(14);
	set.insert(14);
	
	assert_eq!(7, set.len());
	
	assert_eq!(false, set.contains(&6));
	assert_eq!(true, set.contains(&7));
	assert_eq!(true, set.contains(&8));
	assert_eq!(true, set.contains(&9));
	assert_eq!(true, set.contains(&10));
	assert_eq!(true, set.contains(&11));
	assert_eq!(true, set.contains(&12));
	assert_eq!(false, set.contains(&13));
	assert_eq!(true, set.contains(&14));
	assert_eq!(false, set.contains(&15));
	
	assert_eq!(7, set.iter().count());
	
	{
		let mut it = set.iter();
		assert_eq!(Some(7), it.next().copied());
		assert_eq!(Some(8), it.next().copied());
		assert_eq!(Some(9), it.next().copied());
		assert_eq!(Some(10), it.next().copied());
		assert_eq!(Some(11), it.next().copied());
		assert_eq!(Some(12), it.next().copied());
		assert_eq!(Some(14), it.next().copied());
		assert_eq!(None, it.next());
	}
	
	assert_eq!(7, set.iter().rev().count());
	
	{
		let mut rit = set.iter().rev();
		assert_eq!(Some(14), rit.next().copied());
		assert_eq!(Some(12), rit.next().copied());
		assert_eq!(Some(11), rit.next().copied());
		assert_eq!(Some(10), rit.next().copied());
		assert_eq!(Some(9), rit.next().copied());
		assert_eq!(Some(8), rit.next().copied());
		assert_eq!(Some(7), rit.next().copied());
		assert_eq!(None, rit.next());
	}
}

#[test]
fn test_aa_set_retain()
{
	let mut set = Set::<i32>::new();
	for i in 0 .. 100
	{
		set.insert(i);
	}
	set.retain(|i| i % 2 == 0);
	let mut it = set.iter();
	for i in 0 .. 100
	{
		if i % 2 == 0
		{
			assert_eq!(Some(i), it.next().copied());
		}
	}
	assert_eq!(None, it.next());
}

#[test]
fn test_aa_vec_set()
{
	let mut set = VecSet::<i32>::new();
	assert_eq!(None, set.first());
	assert_eq!(None, set.iter().next());
	
	for i in [7, 8, 11, 12, 9, 10, 14, 14]
	{
		set.insert(i);
	}
	
	assert_eq!(7, set.len());
	assert_eq!(Some(&7), set.first());
	assert_eq!(Some(&14), set.last());
	assert_eq!(false, set.contains(&13));
	assert_eq!(true, set.contains(&14));
	assert_eq!(vec![7, 8, 9, 10, 11, 12, 14], set.iter().copied().collect::<Vec<_>>());
	assert_eq!(vec![14, 12, 11, 10, 9, 8, 7], set.iter().rev().copied().collect::<Vec<_>>());
	
	set.retain(|i| i % 2 == 0);
	assert_eq!(vec![8, 10, 12, 14], set.iter().copied().collect::<Vec<_>>());
	assert_eq!(4, set.len());
}

#[cfg(test)]
fn check_against_btree_set<Storage>(mut set: aa::tree::Tree<SetEntry<u32>, Storage>, range: u32, iterations: usize)
where
	Storage: RemovableTreeStorage<SetEntry<u32>>,
{
	use rand::{Rng, SeedableRng};
	let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
	let mut reference = std::collections::BTreeSet::<u32>::new();
	
	for iteration in 0 .. iterations
	{
		let value = rng.gen_range(0 .. range);
		
		match rng.gen_range(0 .. 8)
		{
			0 => assert_eq!(reference.pop_first(), set.pop_first()),
			1 => assert_eq!(reference.pop_last(), set.pop_last()),
			2 | 3 | 4 => assert_eq!(reference.remove(&value), set.remove(&value)),
			_ => assert_eq!(reference.insert(value), set.insert(value)),
		}
		
		assert_eq!(set.len(), node::check_invariants(set.storage.nodes(), set.base.root));
		assert_eq!(reference.len(), set.len());
		assert_eq!(reference.first(), set.first());
		assert_eq!(reference.last(), set.last());
		
		if iteration % 64 == 0
		{
			assert!(reference.iter().eq(set.iter()));
			assert!(reference.iter().rev().eq(set.iter().rev()));
		}
	}
	
	set.retain(|i| i % 3 == 0);
	reference.retain(|i| i % 3 == 0);
	assert_eq!(set.len(), node::check_invariants(set.storage.nodes(), set.base.root));
	assert!(reference.iter().eq(set.iter()));
	
	while let Some(value) = set.pop_first()
	{
		assert_eq!(reference.pop_first(), Some(value));
	}
	assert!(reference.is_empty());
	assert!(set.base.is_empty());
}

#[test]
fn test_aa_set_random()
{
	check_against_btree_set(Set::<u32>::new(), 500, 20000);
	check_against_btree_set(Set::<u32>::new(), 5000, 20000);
}

#[test]
fn test_aa_vec_set_random()
{
	check_against_btree_set(VecSet::<u32>::new(), 500, 20000);
	check_against_btree_set(VecSet::<u32>::new(), 5000, 20000);
}

/*
#[test]
fn test_to_dot()
{
	use std::io::Write;
	let mut set = Set::<i32>::new();
	for i in 0 .. 10
	{
		set.insert(i);
	}
	set.to_dot(&mut std::fs::File::create("tree.dot").unwrap()).unwrap();
	let mut command = std::process::Command::new("dot");
	command.args(["-Tpng", "tree.dot", "-o", "tree.png"].iter());
	let output = command.output().unwrap();
	std::io::stdout().write(output.stdout.as_slice()).unwrap();
	std::io::stderr().write(output.stderr.as_slice()).unwrap();
	if ! output.status.success()
	{
		panic!();
	}
}
*/
