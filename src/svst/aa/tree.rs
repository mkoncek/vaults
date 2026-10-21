use crate::svst::aa::node;
use crate::svst::bit_indexing;
use crate::svst::repository::Repository;

/// Storage of tree nodes.
/// The tree only follows links between nodes which are present in the storage,
/// therefore the slices returned by `nodes` and `nodes_mut` may contain dropped values at other indices.
pub trait TreeStorage<Type>
{
	type LinkType: node::LinkIndex;
	
	fn len(&self) -> usize;
	fn capacity(&self) -> usize;
	fn clear(&mut self);
	fn nodes(&self) -> &[node::Node<Type, Self::LinkType>];
	fn nodes_mut(&mut self) -> &mut [node::Node<Type, Self::LinkType>];
	fn contains(&self, index: usize) -> bool;
	
	/// Inserts a node, the indices of the other nodes must not change.
	fn push(&mut self, value: node::Node<Type, Self::LinkType>) -> usize;
}

/// Storage of tree nodes which supports removal.
pub trait RemovableTreeStorage<Type>: TreeStorage<Type>
{
	type Cursor;
	
	/// Removes the node at `index`, which must be already unlinked from the tree.
	/// Returns the removed node and the former index of the node which was moved to `index`, or `usize::MAX`.
	///
	/// Panics if there is no node at `index`.
	fn remove(&mut self, index: usize) -> (node::Node<Type, Self::LinkType>, usize);
	
	/// Returns a cursor for visiting the indices of all nodes.
	/// Removal of the most recently visited index must not cause any node to be skipped.
	fn cursor(&self) -> Self::Cursor;
	fn advance(&self, cursor: &mut Self::Cursor) -> Option<usize>;
}

impl<Type, LinkType> TreeStorage<Type> for Repository<node::Node<Type, LinkType>>
where
	LinkType: node::LinkIndex,
{
	type LinkType = LinkType;
	
	fn len(&self) -> usize {Repository::len(self)}
	fn capacity(&self) -> usize {Repository::capacity(self)}
	fn clear(&mut self) {Repository::clear(self)}
	fn nodes(&self) -> &[node::Node<Type, LinkType>] {unsafe {self.as_slice()}}
	fn nodes_mut(&mut self) -> &mut [node::Node<Type, LinkType>] {unsafe {self.as_mut_slice()}}
	fn contains(&self, index: usize) -> bool {self.get(index).is_some()}
	
	fn push(&mut self, value: node::Node<Type, LinkType>) -> usize
	{
		self.insert(value)
	}
}

impl<Type, LinkType> RemovableTreeStorage<Type> for Repository<node::Node<Type, LinkType>>
where
	LinkType: node::LinkIndex,
{
	type Cursor = bit_indexing::TransientIndexSliceIterator;
	
	fn remove(&mut self, index: usize) -> (node::Node<Type, LinkType>, usize)
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

impl<Type, LinkType> TreeStorage<Type> for Vec<node::Node<Type, LinkType>>
where
	LinkType: node::LinkIndex,
{
	type LinkType = LinkType;
	
	fn len(&self) -> usize {Vec::len(self)}
	fn capacity(&self) -> usize {Vec::capacity(self)}
	fn clear(&mut self) {Vec::clear(self)}
	fn nodes(&self) -> &[node::Node<Type, LinkType>] {self.as_slice()}
	fn nodes_mut(&mut self) -> &mut [node::Node<Type, LinkType>] {self.as_mut_slice()}
	fn contains(&self, index: usize) -> bool {index < Vec::len(self)}
	
	fn push(&mut self, value: node::Node<Type, LinkType>) -> usize
	{
		Vec::push(self, value);
		Vec::len(self) - 1
	}
}

/// Removal moves the last node into the place of the removed one.
impl<Type, LinkType> RemovableTreeStorage<Type> for Vec<node::Node<Type, LinkType>>
where
	LinkType: node::LinkIndex,
{
	/// Visits the indices in descending order, so the node moved by a removal is one which was already visited.
	type Cursor = usize;
	
	fn remove(&mut self, index: usize) -> (node::Node<Type, LinkType>, usize)
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
pub struct TreeBase<LinkType = usize>
{
	pub root: node::Link<LinkType>,
	pub first: node::Link<LinkType>,
	pub last: node::Link<LinkType>,
}

impl<LinkType> TreeBase<LinkType>
where
	LinkType: node::LinkIndex,
{
	pub const fn new() -> Self
	{
		Self
		{
			root: node::Link::NIL,
			first: node::Link::NIL,
			last: node::Link::NIL,
		}
	}
	
	pub fn is_empty(&self) -> bool {self.root == node::Link::NIL}
	
	pub fn clear(&mut self)
	{
		*self = Self::new();
	}
	
	/// `compare` returns the ordering of a key in the tree relative to the searched one.
	pub fn find<Type, Storage, Compare>(&self, storage: &Storage, mut compare: Compare) -> node::Link<LinkType>
	where
		Type: node::Entry,
		Storage: ?Sized + TreeStorage<Type, LinkType = LinkType>,
		Compare: FnMut(&Type::Key) -> std::cmp::Ordering,
	{
		node::AA::find(storage.nodes(), self.root, |key| compare(key).reverse()).0
	}
	
	pub fn get<'t, Type, Storage, Compare>(&self, storage: &'t Storage, compare: Compare) -> Option<&'t Type>
	where
		Type: node::Entry,
		Storage: ?Sized + TreeStorage<Type, LinkType = LinkType>,
		Compare: FnMut(&Type::Key) -> std::cmp::Ordering,
	{
		let index = self.find(storage, compare);
		
		if index != node::Link::NIL
		{
			return Some(storage.nodes()[index].as_ref());
		}
		
		return None;
	}
	
	pub fn first<'t, Type, Storage>(&self, storage: &'t Storage) -> Option<&'t Type>
	where
		Storage: ?Sized + TreeStorage<Type, LinkType = LinkType>,
	{
		if self.first != node::Link::NIL
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
		Storage: ?Sized + TreeStorage<Type, LinkType = LinkType>,
	{
		if self.last != node::Link::NIL
		{
			Some(storage.nodes()[self.last].as_ref())
		}
		else
		{
			None
		}
	}
	
	pub fn iter<'t, Type, Storage>(&self, storage: &'t Storage) -> node::Iterator<&'t [node::Node<Type, LinkType>], LinkType>
	where
		Storage: ?Sized + TreeStorage<Type, LinkType = LinkType>,
	{
		node::Iterator
		{
			first: self.first,
			last: self.last,
			bounds: [self.first, self.last],
			nodes: storage.nodes(),
		}
	}
	
	/// Inserts `value` unless there already is an equal one.
	/// `consumer` receives the existing value together with `value` if there is one, or `None` if `value` was inserted.
	/// 
	/// `compare` returns the ordering of its first argument relative to the second one.
	pub fn try_insert<Type, Storage, Compare, Consumer, ResultType>(&mut self, storage: &mut Storage, value: Type, mut compare: Compare, consumer: Consumer) -> ResultType
	where
		Type: node::Entry,
		Storage: ?Sized + TreeStorage<Type, LinkType = LinkType>,
		Compare: FnMut(&Type::Key, &Type::Key) -> std::cmp::Ordering,
		Consumer: std::ops::FnOnce(Option<(&mut Type, Type)>) -> ResultType,
	{
		assert!(storage.len() < LinkType::MAX_LEN, "the number of nodes exceeds {}", LinkType::MAX_LEN);
		
		if self.is_empty()
		{
			self.root = node::Link::new(storage.push(node::Node::new(value)));
			self.first = self.root;
			self.last = self.root;
			return consumer(None);
		}
		
		let (mut position, parent, parent_index) = node::AA::find(storage.nodes(), self.root, |key| compare(value.key(), key));
		
		if position != node::Link::NIL
		{
			return consumer(Some((storage.nodes_mut()[position].as_mut(), value)));
		}
		
		position = node::Link::new(storage.push(node::Node::new(value)));
		let nodes = storage.nodes_mut();
		
		if node::AA::insert_rebalance(nodes, parent, parent_index, position)
		{
			self.root = node::AA::skew(nodes, self.root);
			self.root = node::AA::split(nodes, self.root);
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
		Storage: ?Sized + RemovableTreeStorage<Type, LinkType = LinkType>,
	{
		if ! storage.contains(position)
		{
			return None;
		}
		
		let position = node::Link::new(position);
		let nodes = storage.nodes_mut();
		let parent = nodes[position].parent;
		let rdes = nodes[position].descendants[1];
		
		// The node replacing the root if the root is removed
		let root_successor = if position == self.root {node::AA::find_successor(nodes, position)} else {node::Link::NIL};
		
		let new_root = node::AA::erase_rebalance(nodes, position);
		
		if new_root != node::Link::NIL
		{
			self.root = new_root;
		}
		else if position == self.root
		{
			self.root = root_successor;
		}
		
		if position == self.first
		{
			if rdes != node::Link::NIL
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
		let (result, moved) = storage.remove(position.index());
		
		if moved != usize::MAX
		{
			let moved = node::Link::new(moved);
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
		Storage: ?Sized + RemovableTreeStorage<Type, LinkType = LinkType>,
	{
		self.remove_at(storage, self.first.index())
	}
	
	pub fn pop_last<Type, Storage>(&mut self, storage: &mut Storage) -> Option<Type::Value>
	where
		Type: node::Entry,
		Storage: ?Sized + RemovableTreeStorage<Type, LinkType = LinkType>,
	{
		self.remove_at(storage, self.last.index())
	}
}

impl<LinkType> Default for TreeBase<LinkType>
where
	LinkType: node::LinkIndex,
{
	fn default() -> Self {Self::new()}
}

/// An AA tree owning the storage of its nodes.
#[derive(Debug)]
pub struct Tree<Type, Storage = Repository<node::Node<Type>>>
where
	Storage: TreeStorage<Type>,
{
	pub(super) base: TreeBase<Storage::LinkType>,
	pub(super) storage: Storage,
	_data: std::marker::PhantomData<Type>,
}

impl<Type, LinkType> Tree<Type, Repository<node::Node<Type, LinkType>>>
where
	LinkType: node::LinkIndex,
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

impl<Type, LinkType> Tree<Type, Vec<node::Node<Type, LinkType>>>
where
	LinkType: node::LinkIndex,
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
		Consumer: std::ops::FnOnce(Option<(&mut Type, Type)>) -> ResultType,
		Compare: FnMut(&Type::Key, &Type::Key) -> std::cmp::Ordering,
	{
		self.base.try_insert(&mut self.storage, value, compare, consumer)
	}
	
	pub(super) fn impl_find<Compare>(&self, compare: Compare) -> node::Link<Storage::LinkType>
	where
		Type: node::Entry,
		Compare: FnMut(&Type::Key) -> std::cmp::Ordering,
	{
		self.base.find(&self.storage, compare)
	}
	
	pub(super) fn impl_get<Compare>(&self, compare: Compare) -> Option<&Type>
	where
		Type: node::Entry,
		Compare: FnMut(&Type::Key) -> std::cmp::Ordering,
	{
		self.base.get(&self.storage, compare)
	}
	
	pub(super) fn impl_get_mut<Compare>(&mut self, compare: Compare) -> Option<&mut Type>
	where
		Type: node::Entry,
		Compare: FnMut(&Type::Key) -> std::cmp::Ordering,
	{
		let index = self.impl_find(compare);
		
		if index != node::Link::NIL
		{
			return Some(self.storage.nodes_mut()[index].as_mut());
		}
		
		return None;
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
	
	pub(super) fn impl_iter(&self) -> node::Iterator<&[node::Node<Type, Storage::LinkType>], Storage::LinkType>
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
	Storage: TreeStorage<Type> + Default,
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
	fn to_dot_node(&self, index: node::Link<Storage::LinkType>, writer: &mut impl std::io::Write) -> std::io::Result<()>
	{
		writeln!(writer, "node{} [shape=record];", index.index())?;
		
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
			
			if i == node::Link::NIL
			{
				break;
			}
			
			self.to_dot_node(i, writer)?;
			
			for d in nodes[i].descendants
			{
				if d != node::Link::NIL
				{
					writeln!(writer, "node{} -> node{};", i.index(), d.index())?;
				}
			}
			let p = nodes[i].parent;
			if p != node::Link::NIL
			{
				writeln!(writer, "node{} -> node{} [style=dotted];", i.index(), p.index())?;
			}
		}
		writeln!(writer, "}}")?;
		
		Ok(())
	}
}
