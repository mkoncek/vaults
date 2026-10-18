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

pub type Set<KeyType, LinkType = usize> = aa::tree::Tree<SetEntry<KeyType>, crate::svst::Repository<node::Node<SetEntry<KeyType>, LinkType>>>;
pub type VecSet<KeyType, LinkType = usize> = aa::tree::Tree<SetEntry<KeyType>, Vec<node::Node<SetEntry<KeyType>, LinkType>>>;

impl<KeyType, Storage> aa::tree::Tree<SetEntry<KeyType>, Storage>
where
	Storage: TreeStorage<SetEntry<KeyType>>,
{
	pub fn first(&self) -> Option<&KeyType> {self.impl_first().map(|k| &k.0)}
	pub fn last(&self) -> Option<&KeyType> {self.impl_last().map(|k| &k.0)}
	
	/// Like [`contains`](Self::contains), but searches using `compare`,
	/// which returns the ordering of a value in the set relative to the searched one.
	/// 
	/// The ordering must be consistent with the one used for insertion, otherwise the result is unspecified.
	pub fn contains_by<Compare>(&self, compare: Compare) -> bool
	where
		Compare: FnMut(&KeyType) -> std::cmp::Ordering,
	{
		self.impl_find(compare) != node::Link::NIL
	}
	
	/// Like [`get`](Self::get), but searches using `compare`, see [`contains_by`](Self::contains_by).
	pub fn get_by<Compare>(&self, compare: Compare) -> Option<&KeyType>
	where
		Compare: FnMut(&KeyType) -> std::cmp::Ordering,
	{
		self.impl_get(compare).map(|k| &k.0)
	}
	
	/// Like [`insert`](Self::insert), but orders the values using `compare`.
	/// 
	/// The ordering must be consistent for all operations on the set, otherwise their results are unspecified.
	pub fn insert_by<Compare>(&mut self, value: KeyType, compare: Compare) -> bool
	where
		Compare: FnMut(&KeyType, &KeyType) -> std::cmp::Ordering,
	{
		self.try_insert(SetEntry {0: value}, compare, |existing| existing.is_none())
	}
	
	/// Like [`replace`](Self::replace), but orders the values using `compare`, see [`insert_by`](Self::insert_by).
	pub fn replace_by<Compare>(&mut self, value: KeyType, compare: Compare) -> Option<KeyType>
	where
		Compare: FnMut(&KeyType, &KeyType) -> std::cmp::Ordering,
	{
		self.try_insert(SetEntry {0: value}, compare, |existing| existing.map(|(existing, value)| std::mem::replace(existing, value).0))
	}
	
	pub fn iter<'t>(&'t self) -> node::Iterator<&'t [node::Node<SetEntry<KeyType>, Storage::LinkType>], Storage::LinkType>
	{
		self.impl_iter()
	}
	
	pub fn contains<Key>(&self, key: &Key) -> bool
	where
		KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
		Key: ?Sized + std::cmp::Ord,
	{
		self.contains_by(|k| k.borrow().cmp(key))
	}
	
	pub fn get<Key>(&self, key: &Key) -> Option<&KeyType>
	where
		KeyType: std::borrow::Borrow<Key> + std::cmp::Ord,
		Key: ?Sized + std::cmp::Ord,
	{
		self.get_by(|k| k.borrow().cmp(key))
	}
	
	/// Adds a value to the set, returns whether it was newly inserted.
	/// If the set already contains an equal value, the set is not modified and `value` is dropped.
	pub fn insert(&mut self, value: KeyType) -> bool
	where
		KeyType: std::cmp::Ord,
	{
		self.insert_by(value, Ord::cmp)
	}
	
	/// Adds a value to the set, replacing the existing equal value, if any, which is returned.
	pub fn replace(&mut self, value: KeyType) -> Option<KeyType>
	where
		KeyType: std::cmp::Ord,
	{
		self.replace_by(value, Ord::cmp)
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
	/// Like [`remove`](Self::remove), but searches using `compare`, see [`contains_by`](Self::contains_by).
	pub fn remove_by<Compare>(&mut self, compare: Compare) -> bool
	where
		Compare: FnMut(&KeyType) -> std::cmp::Ordering,
	{
		let index = self.impl_find(compare);
		
		if index != node::Link::NIL
		{
			self.remove_at(index.index());
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
		self.remove_by(|k| k.borrow().cmp(value))
	}
}

impl<'t, Type, LinkType> std::iter::Iterator for node::Iterator<&'t [node::Node<SetEntry<Type>, LinkType>], LinkType>
where
	LinkType: node::LinkIndex,
{
	type Item = &'t Type;
	
	fn next(&mut self) -> Option<Self::Item>
	{
		let i = node::iter_impl!(self, 0);
		return if i != node::Link::NIL {Some(&self.nodes[i].as_ref().0)} else {None};
	}
}

impl<'t, Type, LinkType> std::iter::DoubleEndedIterator for node::Iterator<&'t [node::Node<SetEntry<Type>, LinkType>], LinkType>
where
	LinkType: node::LinkIndex,
{
	fn next_back(&mut self) -> Option<Self::Item>
	{
		let i = node::iter_impl!(self, 1);
		return if i != node::Link::NIL {Some(&self.nodes[i].as_ref().0)} else {None};
	}
}

impl<'t, Type, Storage> std::iter::IntoIterator for &'t aa::tree::Tree<SetEntry<Type>, Storage>
where
	Storage: TreeStorage<SetEntry<Type>>,
{
	type Item = &'t Type;
	type IntoIter = node::Iterator<&'t [node::Node<SetEntry<Type>, Storage::LinkType>], Storage::LinkType>;
	
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

#[test]
fn test_aa_set_by()
{
	// Ordered by the second field only, looked up by it without constructing a whole value
	let mut set = Set::<(&str, u32)>::new();
	let by_number = |lhs: &(&str, u32), rhs: &(&str, u32)| lhs.1.cmp(&rhs.1);
	assert!(set.insert_by(("c", 3), by_number));
	assert!(set.insert_by(("a", 1), by_number));
	assert!(set.insert_by(("b", 2), by_number));
	
	// Insertion of an equal value keeps the existing one, replacement swaps it
	assert!(! set.insert_by(("x", 2), by_number));
	assert_eq!(Some(&("b", 2)), set.get_by(|v| v.1.cmp(&2)));
	assert_eq!(Some(("b", 2)), set.replace_by(("y", 2), by_number));
	assert_eq!(3, set.len());
	
	assert_eq!(vec![("a", 1), ("y", 2), ("c", 3)], set.iter().copied().collect::<Vec<_>>());
	assert_eq!(Some(&("y", 2)), set.get_by(|v| v.1.cmp(&2)));
	assert!(! set.contains_by(|v| v.1.cmp(&4)));
	assert!(set.remove_by(|v| v.1.cmp(&1)));
	assert_eq!(vec![("y", 2), ("c", 3)], set.iter().copied().collect::<Vec<_>>());
}

#[test]
fn test_aa_set_inconsistent_ordering()
{
	// An inconsistent ordering gives unspecified results, but the tree stays structurally valid
	use rand::{RngExt, SeedableRng};
	let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
	let mut set = VecSet::<u32>::new();
	
	for _ in 0 .. 5000
	{
		let value = rng.random_range(0 .. 100);
		let ordering = [std::cmp::Ordering::Less, std::cmp::Ordering::Equal, std::cmp::Ordering::Greater][rng.random_range(0 .. 3)];
		
		if rng.random_bool(0.6)
		{
			set.insert_by(value, |_, _| ordering);
		}
		else
		{
			set.remove_by(|_| ordering);
		}
		
		assert_eq!(set.len(), node::check_invariants(set.storage.nodes(), set.base.root));
		assert_eq!(set.len(), set.iter().count());
		
		// The first and last nodes are the leftmost and rightmost ones
		for (bound, side) in [(set.base.first, 0), (set.base.last, 1)]
		{
			let mut index = set.base.root;
			while index != node::Link::NIL && set.storage.nodes()[index].descendants[side] != node::Link::NIL
			{
				index = set.storage.nodes()[index].descendants[side];
			}
			assert_eq!(index, bound);
		}
	}
}

#[cfg(test)]
fn check_against_btree_set<Storage>(mut set: aa::tree::Tree<SetEntry<u32>, Storage>, range: u32, iterations: usize)
where
	Storage: RemovableTreeStorage<SetEntry<u32>>,
{
	use rand::{RngExt, SeedableRng};
	let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
	let mut reference = std::collections::BTreeSet::<u32>::new();
	
	for iteration in 0 .. iterations
	{
		let value = rng.random_range(0 .. range);
		
		match rng.random_range(0 .. 8)
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

#[test]
fn test_aa_set_u32_links_random()
{
	check_against_btree_set(Set::<u32, u32>::new(), 5000, 20000);
	check_against_btree_set(VecSet::<u32, u32>::new(), 5000, 20000);
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
