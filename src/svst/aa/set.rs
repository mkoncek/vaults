use crate::svst::aa;

#[derive(Debug)]
pub struct SetEntry<KeyType>(KeyType);

impl<KeyType> aa::node::Entry for SetEntry<KeyType>
{
	type Key = KeyType;
	type Value = KeyType;
	fn key(&self) -> &Self::Key {&self.0}
	fn value(self) -> Self::Value {self.0}
}

pub type Set<KeyType, Compare = crate::DefaultComparator> = aa::tree::Tree<SetEntry<KeyType>, Compare>;

impl<KeyType, Compare> Set<KeyType, Compare>
{
	pub fn first(&self) -> Option<&KeyType> {self.impl_first().map(|k| &k.0)}
	pub fn last(&self) -> Option<&KeyType> {self.impl_last().map(|k| &k.0)}
	
	pub fn contains<Key>(&self, key: &Key) -> bool
	where
		KeyType: std::borrow::Borrow<Key>,
		Key: ?Sized,
		Compare: crate::Comparator<Key>,
	{
		aa::node::find(unsafe {self.repository.as_slice()}, self.root, key, &self.compare).0 != usize::MAX
	}
	
	pub fn get<Key>(&self, key: &Key) -> Option<&KeyType>
	where
		KeyType: std::borrow::Borrow<Key>,
		Key: ?Sized,
		Compare: crate::Comparator<Key>,
	{
		self.impl_get(key).map(|k| &k.0)
	}
	
	pub fn insert(&mut self, value: KeyType) -> bool
	where
		Compare: crate::Comparator<KeyType>,
	{
		self.try_insert(SetEntry {0: value}, |v| v.is_none())
	}
	
	pub fn replace(&mut self, value: KeyType) -> Option<KeyType>
	where
		Compare: crate::Comparator<KeyType>,
	{
		self.try_insert(SetEntry {0: value}, |v| v.map(|v| v.0))
	}
	
	pub fn remove<Key>(&mut self, value: &Key) -> bool
	where
		KeyType: std::borrow::Borrow<Key>,
		Key: ?Sized,
		Compare: crate::Comparator<Key>,
	{
		let index = aa::node::find(unsafe {self.repository.as_slice()}, self.root, value, &self.compare).0;
		
		if index != usize::MAX
		{
			self.remove_at(index);
			return true;
		}
		
		return false;
	}
	
	pub fn retain<Function>(&mut self, mut function: Function)
	where
		KeyType: std::cmp::Ord,
		Function: std::ops::FnMut(&KeyType) -> bool,
	{
		self.impl_retain(move |k| function(&k.0));
	}
	
	pub fn iter<'t>(&'t self) -> aa::node::Iterator<&'t [aa::node::Node<SetEntry<KeyType>>]>
	{
		aa::node::Iterator::<&'t [aa::node::Node<SetEntry<KeyType>>]>
		{
			first: self.first,
			last: self.last,
			bounds: [self.first, self.last],
			nodes: unsafe {self.repository.as_slice()},
		}
	}
	
	pub unsafe fn get_at_unchecked(&self, position: usize) -> &KeyType
	{
		&self.impl_get_at_unchecked(position).0
	}
}

impl<'t, Type> std::iter::Iterator for aa::node::Iterator<&'t [aa::node::Node<SetEntry<Type>>]>
{
	type Item = &'t Type;
	
	fn next(&mut self) -> Option<Self::Item>
	{
		match aa::node::iter_impl!(self, 0)
		{
			usize::MAX => None,
			i => Some(&self.nodes[i].as_ref().0),
		}
	}
}

impl<'t, Type> std::iter::DoubleEndedIterator for aa::node::Iterator<&'t [aa::node::Node<SetEntry<Type>>]>
{
	fn next_back(&mut self) -> Option<Self::Item>
	{
		match aa::node::iter_impl!(self, 1)
		{
			usize::MAX => None,
			i => Some(&self.nodes[i].as_ref().0),
		}
	}
}

impl<'t, Type> std::iter::IntoIterator for &'t Set<Type>
{
	type Item = &'t Type;
	type IntoIter = aa::node::Iterator<&'t [aa::node::Node<SetEntry<Type>>]>;
	
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
