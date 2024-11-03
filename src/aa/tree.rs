use crate::aa::node;

#[derive(Debug)]
pub struct Tree<Type>
{
	pub(super) root: usize,
	pub(super) first: usize,
	pub(super) last: usize,
	pub(super) repository: crate::repository::Repository<node::Node<Type>>,
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
			repository: crate::repository::Repository::new(),
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
	
	pub(super) fn try_insert<Consumer, ResultType>(&mut self, value: Type, consumer: Consumer) -> ResultType
	where
		Type: node::Entry,
		Type::Key: std::cmp::Ord,
		Consumer: std::ops::FnOnce(Option<Type>) -> ResultType
	{
		if self.is_empty()
		{
			self.root = self.repository.insert(node::Node::new(value));
			self.first = self.root;
			self.last = self.root;
			return consumer(None);
		}
		
		let mut values = unsafe {self.repository.as_mut_slice()};
		let (mut position, parent, parent_index) = node::find(values, self.root, value.key());
		
		if position != usize::MAX
		{
			return consumer(Some(std::mem::replace(&mut values[position].as_mut(), value)));
		}
		
		position = self.repository.insert(node::Node::new(value));
		values = unsafe {self.repository.as_mut_slice()};
		
		if node::insert_rebalance(values, parent, parent_index, position)
		{
			self.root = node::skew(values, self.root);
			self.root = node::split(values, self.root);
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
	
	pub fn at(&self, position: usize) -> &Type
	{
		self.repository[position].as_ref()
	}
	
	pub fn at_mut(&mut self, position: usize) -> &mut Type
	{
		self.repository[position].as_mut()
	}
	
	pub fn remove_at(&mut self, position: usize) -> Type
	{
		let result = self.repository.remove(position).unwrap().value();
		let values = unsafe {self.repository.as_mut_slice()};
		let parent = values[position].parent;
		let rdes = values[position].descendants[1];
		let new_root = node::erase_rebalance(unsafe {self.repository.as_mut_slice()}, position);
		
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
		
		return result
	}
	
	pub(super) fn impl_retain<Function>(&mut self, mut function: Function)
	where
		Function: std::ops::FnMut(&mut Type) -> bool,
	{
		let mut it = crate::bit_indexing::TransientIndexSliceIterator::new(self.repository.index_header_leaf());
		while let Some(i) = it.next(self.repository.index_header_leaf())
		{
			if ! function(self.at_mut(i))
			{
				self.remove_at(i);
			}
		}
	}
	
	pub(super) fn impl_get<Key>(&self, key: &Key) -> Option<&Type>
	where
		Type: node::Entry,
		Type::Key: std::borrow::Borrow<Key> + std::cmp::Ord,
		Key: ?Sized + std::cmp::Ord,
	{
		let index = node::find(unsafe {self.repository.as_slice()}, self.root, key).0;
		
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
			Some(self.at(self.first))
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
			Some(self.at(self.last))
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
			Some(self.remove_at(self.first).value())
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
			Some(self.remove_at(self.last).value())
		}
		else
		{
			None
		}
	}
}

impl<Type> Tree<Type>
{
	fn to_dot_node(&self, index: usize, writer: &mut impl std::io::Write) -> std::io::Result<()>
	{
		writeln!(writer, "node{} [shape=record];", index)?;
		
		Ok(())
	}
	
	pub(crate) fn to_dot(&self, writer: &mut impl std::io::Write) -> std::io::Result<()>
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
