use crate::svst::aa::node;
use crate::svst::repository::Repository;

#[derive(Debug)]
pub struct Tree<Type>
{
	pub(super) root: usize,
	pub(super) first: usize,
	pub(super) last: usize,
	pub(super) repository: Repository<node::Node<Type>>,
}

impl<Type> Tree<Type>
{
	pub const fn new() -> Self
	{
		Self
		{
			root: usize::MAX,
			first: usize::MAX,
			last: usize::MAX,
			repository: Repository::new(),
		}
	}
	
	/// Returns the total number of values the collection can hold without reallocating.
	pub fn capacity(&self) -> usize {self.repository.capacity()}
	
	/// Returns the number of elements in the collection.
	pub fn len(&self) -> usize {self.repository.len()}
	
	/// Returns `true` if the collection contains no values.
	pub fn is_empty(&self) -> bool {self.len() == 0}
	
	pub fn clear(&mut self)
	{
		self.repository.clear();
		self.root = usize::MAX;
		self.first = usize::MAX;
		self.last = usize::MAX;
	}
	
	pub(super) fn try_insert<Consumer, ResultType, Compare>(&mut self, value: Type, compare: Compare, consumer: Consumer) -> ResultType
	where
		Type: node::Entry,
		Consumer: std::ops::FnOnce(Option<Type>) -> ResultType,
		Compare: crate::Comparator<Type::Key>,
	{
		if self.is_empty()
		{
			self.root = self.repository.insert(node::Node::new(value));
			self.first = self.root;
			self.last = self.root;
			return consumer(None);
		}
		
		let mut values = unsafe {self.repository.as_mut_slice()};
		let (mut position, parent, parent_index) = node::AA::find(values, self.root, value.key(), compare);
		
		if position != usize::MAX
		{
			return consumer(Some(std::mem::replace(&mut values[position].as_mut(), value)));
		}
		
		position = self.repository.insert(node::Node::new(value));
		values = unsafe {self.repository.as_mut_slice()};
		
		if node::AA::insert_rebalance(values, parent, parent_index, position)
		{
			self.root = node::AA::skew(values, self.root);
			self.root = node::AA::split(values, self.root);
			values[self.root].parent = usize::MAX;
		}
		
		if values[self.first].descendants[0] == position || values[position].descendants[1] == self.first
		{
			self.first = position;
		}
		
		if values[position].parent == self.last
		{
			self.last = position;
		}
		
		return consumer(None);
	}
	
	pub fn impl_get_at(&self, position: usize) -> Option<&Type>
	{
		self.repository.get(position).map(AsRef::as_ref)
	}
	
	pub fn impl_get_at_mut(&mut self, position: usize) -> Option<&mut Type>
	{
		self.repository.get_mut(position).map(AsMut::as_mut)
	}
	
	pub unsafe fn impl_get_at_unchecked(&self, position: usize) -> &Type
	{
		self.repository.get_unchecked(position).as_ref()
	}
	
	pub unsafe fn impl_get_at_unchecked_mut(&mut self, position: usize) -> &mut Type
	{
		self.repository.get_unchecked_mut(position).as_mut()
	}
	
	pub fn impl_at(&self, position: usize) -> &Type
	{
		self.repository[position].as_ref()
	}
	
	pub fn impl_at_mut(&mut self, position: usize) -> &mut Type
	{
		self.repository[position].as_mut()
	}
	
	pub fn remove_at(&mut self, position: usize) -> Option<Type::Value>
	where Type: node::Entry
	{
		let Some(result) = self.repository.remove(position) else
		{
			return None;
		};
		let values = unsafe {self.repository.as_mut_slice()};
		let parent = values[position].parent;
		let rdes = values[position].descendants[1];
		let new_root = node::AA::erase_rebalance(values, position);
		
		if new_root != usize::MAX
		{
			self.root = new_root;
		}
		else if self.is_empty()
		{
			self.root = usize::MAX;
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
		
		return Some(result.value().value());
	}
	
	pub(super) fn impl_retain(&mut self, mut function: impl std::ops::FnMut(&mut Type) -> bool)
	where Type: node::Entry
	{
		let mut it = crate::svst::bit_indexing::TransientIndexSliceIterator::new(self.repository.index_header_leaf());
		while let Some(i) = it.next(self.repository.index_header_leaf())
		{
			if ! function(self.impl_at_mut(i))
			{
				self.remove_at(i);
			}
		}
	}
	
	pub(super) fn impl_get<Key, Compare>(&self, key: &Key, compare: Compare) -> Option<&Type>
	where
		Type: node::Entry,
		Type::Key: std::borrow::Borrow<Key>,
		Key: ?Sized,
		Compare: crate::Comparator<Key>,
	{
		let index = node::AA::find(unsafe {self.repository.as_slice()}, self.root, key, compare).0;
		
		if index != usize::MAX
		{
			return Some(&self.repository[index].as_ref());
		}
		
		return None;
	}
	
	pub(super) fn impl_first(&self) -> Option<&Type>
	{
		if self.first != usize::MAX
		{
			Some(unsafe {self.impl_get_at_unchecked(self.first)})
		}
		else
		{
			None
		}
	}
	
	pub(super) fn impl_last(&self) -> Option<&Type>
	{
		if self.last != usize::MAX
		{
			Some(unsafe {self.impl_get_at_unchecked(self.last)})
		}
		else
		{
			None
		}
	}
	
	pub fn pop_first(&mut self) -> Option<Type::Value>
	where Type: node::Entry
	{
		if self.first != usize::MAX
		{
			self.remove_at(self.first)
		}
		else
		{
			None
		}
	}
	
	pub fn pop_last(&mut self) -> Option<Type::Value>
	where Type: node::Entry
	{
		if self.last != usize::MAX
		{
			self.remove_at(self.last)
		}
		else
		{
			None
		}
	}
}

impl<Type> Default for Tree<Type>
{
	fn default() -> Self {Self::new()}
}

impl<Type> Tree<Type>
{
	fn to_dot_node(&self, index: usize, writer: &mut impl std::io::Write) -> std::io::Result<()>
	{
		writeln!(writer, "node{} [shape=record];", index)?;
		
		Ok(())
	}
	
	pub fn _to_dot(&self, writer: &mut impl std::io::Write) -> std::io::Result<()>
	{
		writeln!(writer, "digraph tree {{")?;
		if self.root != usize::MAX
		{
			self.to_dot_node(self.root, writer)?;
		}
		for i in self.repository.index_iter()
		{
			if i != self.root
			{
				self.to_dot_node(i, writer)?;
			}
		}
		for i in self.repository.index_iter()
		{
			for d in self.repository[i].descendants
			{
				if d != usize::MAX
				{
					writeln!(writer, "node{} -> node{};", i, d)?;
				}
			}
			let p = self.repository[i].parent;
			if p != usize::MAX
			{
				writeln!(writer, "node{} -> node{} [style=dotted];", i, p)?;
			}
		}
		writeln!(writer, "}}")?;
		
		Ok(())
	}
}
