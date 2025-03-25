use std::borrow::Borrow;
use crate::svst::aa::node;
use crate::svst::bit_indexing;
use crate::svst::repository::Repository;

/// Storage of tree nodes.
/// The tree only follows links between nodes which are present in the storage,
/// therefore the slices returned by `nodes` and `nodes_mut` may contain dropped values at other indices.
pub trait TreeStorage<Type>
{
	fn len(&self) -> usize;
	fn capacity(&self) -> usize;
	fn clear(&mut self);
	fn nodes(&self) -> &[node::Node<Type>];
	fn nodes_mut(&mut self) -> &mut [node::Node<Type>];
	fn contains(&self, index: usize) -> bool;
	
	/// Inserts a node, the indices of the other nodes must not change.
	fn push(&mut self, value: node::Node<Type>) -> usize;
}

/// Storage of tree nodes which supports removal.
pub trait RemovableTreeStorage<Type>: TreeStorage<Type>
{
	type Cursor;
	
	/// Removes the node at `index`, which must be already unlinked from the tree.
	/// Returns the removed node and the former index of the node which was moved to `index`, or `usize::MAX`.
	///
	/// Panics if there is no node at `index`.
	fn remove(&mut self, index: usize) -> (node::Node<Type>, usize);
	
	/// Returns a cursor for visiting the indices of all nodes.
	/// Removal of the most recently visited index must not cause any node to be skipped.
	fn cursor(&self) -> Self::Cursor;
	fn advance(&self, cursor: &mut Self::Cursor) -> Option<usize>;
}

impl<Type> TreeStorage<Type> for Repository<node::Node<Type>>
{
	fn len(&self) -> usize {Repository::len(self)}
	fn capacity(&self) -> usize {Repository::capacity(self)}
	fn clear(&mut self) {Repository::clear(self)}
	fn nodes(&self) -> &[node::Node<Type>] {unsafe {self.as_slice()}}
	fn nodes_mut(&mut self) -> &mut [node::Node<Type>] {unsafe {self.as_mut_slice()}}
	fn contains(&self, index: usize) -> bool {self.get(index).is_some()}
	
	fn push(&mut self, value: node::Node<Type>) -> usize
	{
		self.insert(value)
	}
}

impl<Type> RemovableTreeStorage<Type> for Repository<node::Node<Type>>
{
	type Cursor = bit_indexing::TransientIndexSliceIterator;
	
	fn remove(&mut self, index: usize) -> (node::Node<Type>, usize)
	{
		(Repository::remove(self, index).expect("no node at index"), usize::MAX)
	}
	
	fn cursor(&self) -> Self::Cursor
	{
		bit_indexing::TransientIndexSliceIterator::new(self.index_header_leaf())
	}
	
	fn advance(&self, cursor: &mut Self::Cursor) -> Option<usize>
	{
		cursor.next(self.index_header_leaf())
	}
}

impl<Type> TreeStorage<Type> for Vec<node::Node<Type>>
{
	fn len(&self) -> usize {Vec::len(self)}
	fn capacity(&self) -> usize {Vec::capacity(self)}
	fn clear(&mut self) {Vec::clear(self)}
	fn nodes(&self) -> &[node::Node<Type>] {self.as_slice()}
	fn nodes_mut(&mut self) -> &mut [node::Node<Type>] {self.as_mut_slice()}
	fn contains(&self, index: usize) -> bool {index < Vec::len(self)}
	
	fn push(&mut self, value: node::Node<Type>) -> usize
	{
		Vec::push(self, value);
		Vec::len(self) - 1
	}
}

/// Removal moves the last node into the place of the removed one.
impl<Type> RemovableTreeStorage<Type> for Vec<node::Node<Type>>
{
	/// Visits the indices in descending order, so the node moved by a removal is one which was already visited.
	type Cursor = usize;
	
	fn remove(&mut self, index: usize) -> (node::Node<Type>, usize)
	{
		let last = Vec::len(self) - 1;
		(self.swap_remove(index), if index != last {last} else {usize::MAX})
	}
	
	fn cursor(&self) -> Self::Cursor {Vec::len(self)}
	
	fn advance(&self, cursor: &mut Self::Cursor) -> Option<usize>
	{
		if *cursor == 0
		{
			return None;
		}
		
		*cursor -= 1;
		return Some(*cursor);
	}
}

/// The bookkeeping of an AA tree, independent of the storage of its nodes.
#[derive(Debug)]
pub struct TreeBase
{
	pub root: usize,
	pub first: usize,
	pub last: usize,
}

impl TreeBase
{
	pub const fn new() -> Self
	{
		Self
		{
			root: usize::MAX,
			first: usize::MAX,
			last: usize::MAX,
		}
	}
	
	pub fn is_empty(&self) -> bool {self.root == usize::MAX}
	
	pub fn clear(&mut self)
	{
		*self = Self::new();
	}
	
	pub fn find<Type, Key, Storage, Compare>(&self, storage: &Storage, key: &Key, compare: Compare) -> usize
	where
		Type: node::Entry,
		Key: ?Sized,
		Storage: ?Sized + TreeStorage<Type>,
		Compare: Fn(&Key, &Type::Key) -> std::cmp::Ordering,
	{
		node::AA::find(storage.nodes(), self.root, key, compare).0
	}
	
	pub fn get<'t, Type, Key, Storage, Compare>(&self, storage: &'t Storage, key: &Key, compare: Compare) -> Option<&'t Type>
	where
		Type: node::Entry,
		Key: ?Sized,
		Storage: ?Sized + TreeStorage<Type>,
		Compare: Fn(&Key, &Type::Key) -> std::cmp::Ordering,
	{
		let index = self.find(storage, key, compare);
		
		if index != usize::MAX
		{
			return Some(storage.nodes()[index].as_ref());
		}
		
		return None;
	}
	
	pub fn first<'t, Type, Storage>(&self, storage: &'t Storage) -> Option<&'t Type>
	where
		Storage: ?Sized + TreeStorage<Type>,
	{
		if self.first != usize::MAX
		{
			Some(storage.nodes()[self.first].as_ref())
		}
		else
		{
			None
		}
	}
	
	pub fn last<'t, Type, Storage>(&self, storage: &'t Storage) -> Option<&'t Type>
	where
		Storage: ?Sized + TreeStorage<Type>,
	{
		if self.last != usize::MAX
		{
			Some(storage.nodes()[self.last].as_ref())
		}
		else
		{
			None
		}
	}
	
	pub fn iter<'t, Type, Storage>(&self, storage: &'t Storage) -> node::Iterator<&'t [node::Node<Type>]>
	where
		Storage: ?Sized + TreeStorage<Type>,
	{
		node::Iterator
		{
			first: self.first,
			last: self.last,
			bounds: [self.first, self.last],
			nodes: storage.nodes(),
		}
	}
	
	pub fn try_insert<Type, Storage, Compare, Consumer, ResultType>(&mut self, storage: &mut Storage, value: Type, compare: Compare, consumer: Consumer) -> ResultType
	where
		Type: node::Entry,
		Storage: ?Sized + TreeStorage<Type>,
		Compare: Fn(&Type::Key, &Type::Key) -> std::cmp::Ordering,
		Consumer: std::ops::FnOnce(Option<Type>) -> ResultType,
	{
		if self.is_empty()
		{
			self.root = storage.push(node::Node::new(value));
			self.first = self.root;
			self.last = self.root;
			return consumer(None);
		}
		
		let (mut position, parent, parent_index) = node::AA::find(storage.nodes(), self.root, value.key(), compare);
		
		if position != usize::MAX
		{
			return consumer(Some(std::mem::replace(storage.nodes_mut()[position].as_mut(), value)));
		}
		
		position = storage.push(node::Node::new(value));
		let nodes = storage.nodes_mut();
		
		if node::AA::insert_rebalance(nodes, parent, parent_index, position)
		{
			self.root = node::AA::skew(nodes, self.root);
			self.root = node::AA::split(nodes, self.root);
			nodes[self.root].parent = usize::MAX;
		}
		
		if nodes[self.first].descendants[0] == position || nodes[position].descendants[1] == self.first
		{
			self.first = position;
		}
		
		if nodes[position].parent == self.last
		{
			self.last = position;
		}
		
		return consumer(None);
	}
	
	pub fn remove_at<Type, Storage>(&mut self, storage: &mut Storage, position: usize) -> Option<Type::Value>
	where
		Type: node::Entry,
		Storage: ?Sized + RemovableTreeStorage<Type>,
	{
		if ! storage.contains(position)
		{
			return None;
		}
		
		let nodes = storage.nodes_mut();
		let parent = nodes[position].parent;
		let rdes = nodes[position].descendants[1];
		
		// The node replacing the root if the root is removed
		let root_successor = if position == self.root {node::AA::find_successor(nodes, position)} else {usize::MAX};
		
		let new_root = node::AA::erase_rebalance(nodes, position);
		
		if new_root != usize::MAX
		{
			self.root = new_root;
		}
		else if position == self.root
		{
			self.root = root_successor;
		}
		
		if position == self.first
		{
			if rdes != usize::MAX
			{
				self.first = rdes;
			}
			else
			{
				self.first = parent;
			}
		}
		
		if position == self.last
		{
			self.last = parent;
		}
		
		// The node is unlinked now, but it stays at `position` until here
		let (result, moved) = storage.remove(position);
		
		if moved != usize::MAX
		{
			node::AA::relocate(storage.nodes_mut(), moved, position);
			
			for index in [&mut self.root, &mut self.first, &mut self.last]
			{
				if *index == moved
				{
					*index = position;
				}
			}
		}
		
		return Some(result.value().value());
	}
	
	pub fn pop_first<Type, Storage>(&mut self, storage: &mut Storage) -> Option<Type::Value>
	where
		Type: node::Entry,
		Storage: ?Sized + RemovableTreeStorage<Type>,
	{
		self.remove_at(storage, self.first)
	}
	
	pub fn pop_last<Type, Storage>(&mut self, storage: &mut Storage) -> Option<Type::Value>
	where
		Type: node::Entry,
		Storage: ?Sized + RemovableTreeStorage<Type>,
	{
		self.remove_at(storage, self.last)
	}
}

impl Default for TreeBase
{
	fn default() -> Self {Self::new()}
}

/// An AA tree owning the storage of its nodes.
#[derive(Debug)]
pub struct Tree<Type, Storage = Repository<node::Node<Type>>>
{
	pub(super) base: TreeBase,
	pub(super) storage: Storage,
	_data: std::marker::PhantomData<Type>,
}

impl<Type> Tree<Type, Repository<node::Node<Type>>>
{
	pub const fn new() -> Self
	{
		Self
		{
			base: TreeBase::new(),
			storage: Repository::new(),
			_data: std::marker::PhantomData,
		}
	}
	
	pub fn with_capacity(capacity: usize) -> Self
	{
		Self
		{
			base: TreeBase::new(),
			storage: Repository::with_capacity(capacity),
			_data: std::marker::PhantomData,
		}
	}
}

impl<Type> Tree<Type, Vec<node::Node<Type>>>
{
	pub const fn new() -> Self
	{
		Self
		{
			base: TreeBase::new(),
			storage: Vec::new(),
			_data: std::marker::PhantomData,
		}
	}
	
	pub fn with_capacity(capacity: usize) -> Self
	{
		Self
		{
			base: TreeBase::new(),
			storage: Vec::with_capacity(capacity),
			_data: std::marker::PhantomData,
		}
	}
}

impl<Type, Storage> Tree<Type, Storage>
where
	Storage: TreeStorage<Type>,
{
	/// Returns the total number of values the collection can hold without reallocating.
	pub fn capacity(&self) -> usize {self.storage.capacity()}
	
	/// Returns the number of elements in the collection.
	pub fn len(&self) -> usize {self.storage.len()}
	
	/// Returns `true` if the collection contains no values.
	pub fn is_empty(&self) -> bool {self.len() == 0}
	
	pub fn clear(&mut self)
	{
		self.storage.clear();
		self.base.clear();
	}
	
	pub(super) fn try_insert<Consumer, ResultType, Compare>(&mut self, value: Type, compare: Compare, consumer: Consumer) -> ResultType
	where
		Type: node::Entry,
		Consumer: std::ops::FnOnce(Option<Type>) -> ResultType,
		Compare: crate::Comparator<Type::Key>,
	{
		self.base.try_insert(&mut self.storage, value, |l, r| compare.compare(l, r), consumer)
	}
	
	pub(super) fn impl_find<Key, Compare>(&self, key: &Key, compare: Compare) -> usize
	where
		Type: node::Entry,
		Type::Key: Borrow<Key>,
		Key: ?Sized,
		Compare: crate::Comparator<Key>,
	{
		self.base.find(&self.storage, key, |l, r| compare.compare(l, r.borrow()))
	}
	
	pub(super) fn impl_get<Key, Compare>(&self, key: &Key, compare: Compare) -> Option<&Type>
	where
		Type: node::Entry,
		Type::Key: Borrow<Key>,
		Key: ?Sized,
		Compare: crate::Comparator<Key>,
	{
		self.base.get(&self.storage, key, |l, r| compare.compare(l, r.borrow()))
	}
	
	pub(super) fn impl_get_mut<Key, Compare>(&mut self, key: &Key, compare: Compare) -> Option<&mut Type>
	where
		Type: node::Entry,
		Type::Key: Borrow<Key>,
		Key: ?Sized,
		Compare: crate::Comparator<Key>,
	{
		match self.impl_find(key, compare)
		{
			usize::MAX => None,
			index => Some(self.storage.nodes_mut()[index].as_mut()),
		}
	}
	
	pub fn impl_get_at(&self, position: usize) -> Option<&Type>
	{
		if self.storage.contains(position)
		{
			Some(self.storage.nodes()[position].as_ref())
		}
		else
		{
			None
		}
	}
	
	pub fn impl_get_at_mut(&mut self, position: usize) -> Option<&mut Type>
	{
		if self.storage.contains(position)
		{
			Some(self.storage.nodes_mut()[position].as_mut())
		}
		else
		{
			None
		}
	}
	
	pub unsafe fn impl_get_at_unchecked(&self, position: usize) -> &Type
	{
		self.storage.nodes().get_unchecked(position).as_ref()
	}
	
	pub unsafe fn impl_get_at_unchecked_mut(&mut self, position: usize) -> &mut Type
	{
		self.storage.nodes_mut().get_unchecked_mut(position).as_mut()
	}
	
	pub fn impl_at(&self, position: usize) -> &Type
	{
		self.impl_get_at(position).unwrap_or_else(|| panic!("index {} contains an invalid value", position))
	}
	
	pub fn impl_at_mut(&mut self, position: usize) -> &mut Type
	{
		self.impl_get_at_mut(position).unwrap_or_else(|| panic!("index {} contains an invalid value", position))
	}
	
	pub(super) fn impl_first(&self) -> Option<&Type> {self.base.first(&self.storage)}
	pub(super) fn impl_last(&self) -> Option<&Type> {self.base.last(&self.storage)}
	
	pub(super) fn impl_iter(&self) -> node::Iterator<&[node::Node<Type>]>
	{
		self.base.iter(&self.storage)
	}
}

impl<Type, Storage> Tree<Type, Storage>
where
	Storage: RemovableTreeStorage<Type>,
{
	pub fn remove_at(&mut self, position: usize) -> Option<Type::Value>
	where
		Type: node::Entry,
	{
		self.base.remove_at(&mut self.storage, position)
	}
	
	pub(super) fn impl_retain(&mut self, mut function: impl std::ops::FnMut(&mut Type) -> bool)
	where
		Type: node::Entry,
	{
		let mut cursor = self.storage.cursor();
		while let Some(i) = self.storage.advance(&mut cursor)
		{
			if ! function(self.storage.nodes_mut()[i].as_mut())
			{
				self.remove_at(i);
			}
		}
	}
	
	pub fn pop_first(&mut self) -> Option<Type::Value>
	where
		Type: node::Entry,
	{
		self.base.pop_first(&mut self.storage)
	}
	
	pub fn pop_last(&mut self) -> Option<Type::Value>
	where
		Type: node::Entry,
	{
		self.base.pop_last(&mut self.storage)
	}
}

impl<Type, Storage> Default for Tree<Type, Storage>
where
	Storage: Default,
{
	fn default() -> Self
	{
		Self
		{
			base: TreeBase::new(),
			storage: Storage::default(),
			_data: std::marker::PhantomData,
		}
	}
}

impl<Type, Storage> Tree<Type, Storage>
where
	Storage: TreeStorage<Type>,
{
	fn to_dot_node(&self, index: usize, writer: &mut impl std::io::Write) -> std::io::Result<()>
	{
		writeln!(writer, "node{} [shape=record];", index)?;
		
		Ok(())
	}
	
	pub fn _to_dot(&self, writer: &mut impl std::io::Write) -> std::io::Result<()>
	{
		writeln!(writer, "digraph tree {{")?;
		let nodes = self.storage.nodes();
		let mut it = self.impl_iter();
		loop
		{
			let i = node::iter_impl!(it, 0);
			
			if i == usize::MAX
			{
				break;
			}
			
			self.to_dot_node(i, writer)?;
			
			for d in nodes[i].descendants
			{
				if d != usize::MAX
				{
					writeln!(writer, "node{} -> node{};", i, d)?;
				}
			}
			let p = nodes[i].parent;
			if p != usize::MAX
			{
				writeln!(writer, "node{} -> node{} [style=dotted];", i, p)?;
			}
		}
		writeln!(writer, "}}")?;
		
		Ok(())
	}
}
