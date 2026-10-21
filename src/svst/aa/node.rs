pub trait Entry
{
	type Key;
	type Value;
	fn key(&self) -> &Self::Key;
	fn value(self) -> Self::Value;
}

/// The index of a node in its storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Link(usize);

impl Link
{
	/// The link to no node.
	pub const NIL: Self = Self(usize::MAX);
	
	pub fn new(index: usize) -> Self {Self(index)}
	pub fn index(self) -> usize {self.0}
}

#[derive(Debug)]
pub struct Node<Type>
{
	pub(super) parent: Link,
	pub(super) descendants: [Link; 2],
	pub(super) level: i16,
	value: Type,
}

impl<Type> std::ops::Index<Link> for [Node<Type>]
{
	type Output = Node<Type>;
	fn index(&self, index: Link) -> &Self::Output {&self[index.index()]}
}

impl<Type> std::ops::IndexMut<Link> for [Node<Type>]
{
	fn index_mut(&mut self, index: Link) -> &mut Self::Output {&mut self[index.index()]}
}

impl<Type> AsRef<Type> for Node<Type>
{
	fn as_ref(&self) -> &Type {&self.value}
}

impl<Type> AsMut<Type> for Node<Type>
{
	fn as_mut(&mut self) -> &mut Type {&mut self.value}
}

impl<Type> Node<Type>
{
	pub fn new(value: Type) -> Self
	{
		Self
		{
			parent: Link::NIL,
			descendants: [Link::NIL, Link::NIL],
			level: 0,
			value,
		}
	}
	
	pub fn value(self) -> Type {self.value}
}

pub(super) trait AA<Type>: std::ops::Index<Link, Output = Node<Type>>
{
	fn get_parent_index(&self, index: Link, parent: Link) -> u8
	{
		(self[parent].descendants[1] == index) as u8
	}
	
	fn skew(&mut self, index: Link) -> Link
	where
		Self: std::ops::IndexMut<Link, Output = Node<Type>>,
	{
		let l_index = self[index].descendants[0];
		
		if l_index == Link::NIL
		{
			return index;
		}
		else if self[index].level == self[l_index].level
		{
			let lrdesc = self[l_index].descendants[1];
			
			if lrdesc != Link::NIL
			{
				self[lrdesc].parent = index;
			}
			
			self[l_index].parent = self[index].parent;
			self[index].parent = l_index;
			
			self[index].descendants[0] = self[l_index].descendants[1];
			self[l_index].descendants[1] = index;
			
			return l_index;
		}
		
		return index;
	}
	
	fn split(&mut self, index: Link) -> Link
	where
		Self: std::ops::IndexMut<Link, Output = Node<Type>>,
	{
		let r_index = self[index].descendants[1];
		
		if r_index == Link::NIL || self[r_index].descendants[1] == Link::NIL
		{
			return index;
		}
		else if self[index].level == self[self[r_index].descendants[1]].level
		{
			let rldesc = self[r_index].descendants[0];
			
			if rldesc != Link::NIL
			{
				self[rldesc].parent = index;
			}
			
			self[r_index].parent = self[index].parent;
			self[index].parent = r_index;
			
			self[index].descendants[1] = self[r_index].descendants[0];
			self[r_index].descendants[0] = index;
			self[r_index].level = self[r_index].level + 1;
			
			return r_index;
		}
		
		return index;
	}
	
	/// `compare` returns the ordering of the searched key relative to a key in the tree.
	fn find<Compare>(&self, root: Link, mut compare: Compare) -> (Link, Link, u8)
	where
		Type: Entry,
		Compare: FnMut(&Type::Key) -> std::cmp::Ordering,
	{
		let mut desc = root;
		let mut parent = Link::NIL;
		let mut parent_index: u8 = 0;
		
		while desc != Link::NIL
		{
			parent = desc;
			
			match compare(self[desc].value.key())
			{
				std::cmp::Ordering::Less =>
				{
					parent_index = 0;
					desc = self[desc].descendants[parent_index as usize];
				},
				std::cmp::Ordering::Greater =>
				{
					parent_index = 1;
					desc = self[desc].descendants[parent_index as usize];
				},
				std::cmp::Ordering::Equal =>
				{
					break;
				}
			}
		}
		
		return (desc, parent, parent_index);
	}
	
	fn swap_nodes(&mut self, index: Link, successor: Link)
	where
		Self: std::ops::IndexMut<Link, Output = Node<Type>>,
	{
		if index == successor
		{
			unreachable!();
		}
		
		let parent = self[index].parent;
		let successor_rdes = self[successor].descendants[1];
		
		if self[index].descendants[1] == successor
		{
			self[index].parent = successor;
			self[successor].descendants[1] = index;
		}
		else
		{
			self[index].parent = self[successor].parent;
			self[successor].descendants[1] = self[index].descendants[1];
			let successor_parent = self[successor].parent;
			self[successor_parent].descendants[0] = index;
			self[index].parent = successor_parent;
			let index = self[index].descendants[1];
			self[index].parent = successor;
		}
		
		self[successor].parent = parent;
		
		if parent != Link::NIL
		{
			let parent_index = self.get_parent_index(index, parent);
			self[parent].descendants[parent_index as usize] = successor;
		}
		
		let index_ldes = self[index].descendants[0];
		self[successor].descendants[0] = index_ldes;
		self[index].descendants[0] = Link::NIL;
		
		if index_ldes != Link::NIL
		{
			self[index_ldes].parent = successor;
		}
		
		self[index].descendants[1] = successor_rdes;
		
		{
			let level = self[index].level;
			self[index].level = self[successor].level;
			self[successor].level = level;
		}
		
		// NOTE The parent of `successor_rdes` is left pointing to `successor`.
		// It is not read before `erase_rebalance_leaf(index)`, the only operation following this one,
		// which sets it to the parent of `index` when unlinking `index`.
	}
	
	/// The number of ancestors which are rebalanced in insertion after the last one which was changed,
	/// counting the inserted node as changed.
	/// 
	/// Rebalancing a node reads only the nodes up to two levels below it, `skew` reads the left child and `split` the right grandchild.
	/// All changes happen in the subtree of the last changed node and the unchanged ancestors keep their levels,
	/// so the ancestors further above read the same nodes as in the valid tree before the insertion, where rebalancing is a no-op.
	/// 
	/// One is not enough, `split` raises the root of the changed subtree to the level of its parent,
	/// which can create two consecutive right horizontal links at the grandparent.
	/// This happens already when inserting the third ascending value.
	const INSERT_PROPAGATION_DISTANCE: i32 = 2;
	
	/// The number of ancestors which are rebalanced in erasure after the last one which was changed,
	/// counting the removed node as changed.
	/// 
	/// Rebalancing a node in erasure makes its subtree a valid tree whose root level is the original one or one lower.
	/// If the rebalancing of its parent is a no-op, the parent's children have the levels required by the invariants,
	/// since they are not lower than the parent's level minus one and not higher than originally,
	/// and the right grandchild is lower than the parent, otherwise `split` would act.
	/// So the subtree of the parent is valid with the same root and level.
	/// The grandparent then reads the unchanged level of the parent and nodes in its valid subtree,
	/// except for `split` comparing its level with the right grandchild,
	/// which was lower than it before and the levels only decreased, so rebalancing it is also a no-op, and so on.
	const ERASE_PROPAGATION_DISTANCE: i32 = 1;
	
	fn insert_rebalance(&mut self, mut parent: Link,
		mut parent_index: u8, mut index: Link) -> bool
	where
		Self: std::ops::IndexMut<Link, Output = Node<Type>>,
	{
		self[index].parent = parent;
		self[parent].descendants[parent_index as usize] = index;
		
		let mut changes = Self::INSERT_PROPAGATION_DISTANCE;
		
		while {index = parent; parent = self[parent].parent;
			parent != Link::NIL && changes > 0
		}
		{
			parent_index = self.get_parent_index(index, parent);
			
			changes -= 1;
			
			let original = index;
			
			let nv = self.skew(index);
			
			if nv != index
			{
				index = nv;
				changes = Self::INSERT_PROPAGATION_DISTANCE;
			}
			
			let nv = self.split(index);
			
			if nv != index
			{
				index = nv;
				changes = Self::INSERT_PROPAGATION_DISTANCE;
			}
			
			if index != original
			{
				self[parent].descendants[parent_index as usize] = index;
			}
		}
		
		return changes > 0;
	}
	
	fn erase_rebalance_leaf(&mut self, mut index: Link) -> Link
	where
		Self: std::ops::IndexMut<Link, Output = Node<Type>>,
	{
		if self[index].level != 0
		{
			unreachable!();
		}
		
		let mut rdes = self[index].descendants[1];
		let mut parent = self[index].parent;
		
		if rdes != Link::NIL
		{
			self[rdes].parent = parent;
		}
		
		if parent == Link::NIL
		{
			return self[index].descendants[1];
		}
		
		{
			let parent_index = self.get_parent_index(index, parent);
			self[parent].descendants[parent_index as usize] = rdes;
		}
		
		let mut changes = Self::ERASE_PROPAGATION_DISTANCE;
		
		loop
		{
			changes -= 1;
			index = parent;
			parent = self[parent].parent;
			
			let mut level = -1;
			
			{
				let ldes = self[index].descendants[0];
				
				if ldes != Link::NIL
				{
					level = self[ldes].level;
				}
			}
			
			rdes = self[index].descendants[1];
			
			if rdes != Link::NIL
			{
				let rlevel = self[rdes].level;
				
				if rlevel < level
				{
					level = rlevel;
				}
			}
			else
			{
				level = -1;
			}
			
			level += 1;
			
			if level < self[index].level
			{
				changes = Self::ERASE_PROPAGATION_DISTANCE;
				
				self[index].level = level;
				
				if rdes != Link::NIL && level < self[rdes].level
				{
					self[rdes].level = level;
				}
				
				let original = index;
				
				{
					let new_index = self.skew(index);
					
					if new_index != index
					{
						index = new_index;
					}
				}
				{
					let mut rdes = self[index].descendants[1];
					
					if rdes != Link::NIL
					{
						rdes = self.skew(rdes);
						
						if rdes != self[index].descendants[1]
						{
							self[index].descendants[1] = rdes;
						}
						
						let mut rrdes = self[rdes].descendants[1];
						
						if rrdes != Link::NIL
						{
							rrdes = self.skew(rrdes);
							
							if rrdes != self[rdes].descendants[1]
							{
								self[rdes].descendants[1] = rrdes;
							}
						}
					}
				}
				{
					let new_index = self.split(index);
					
					if new_index != index
					{
						index = new_index;
					}
				}
				{
					let mut rdes = self[index].descendants[1];
					
					if rdes != Link::NIL
					{
						rdes = self.split(rdes);
						
						if rdes != self[index].descendants[1]
						{
							self[index].descendants[1] = rdes;
						}
					}
				}
				
				if parent != Link::NIL && index != original
				{
					let parent_index = self.get_parent_index(original, parent);
					self[parent].descendants[parent_index as usize] = index;
				}
			}
			
			if parent == Link::NIL || changes == 0
			{
				break;
			}
		}
		
		return if parent == Link::NIL {index} else {Link::NIL};
	}
	
	fn find_successor(&mut self, mut index: Link) -> Link
	{
		if self[index].level == 0
		{
			return Link::NIL;
		}
		
		index = self[index].descendants[1];
		
		while self[index].level > 0
		{
			index = self[index].descendants[0];
		}
		
		return index;
	}
	
	fn erase_rebalance(&mut self, index: Link) -> Link
	where
		Self: std::ops::IndexMut<Link, Output = Node<Type>>,
	{
		let successor = self.find_successor(index);
		
		if successor != Link::NIL
		{
			self.swap_nodes(index, successor);
		}
		
		return self.erase_rebalance_leaf(index);
	}
	
	/// Updates the links pointing to a node which was moved from `from` to `to`.
	fn relocate(&mut self, from: Link, to: Link)
	where
		Self: std::ops::IndexMut<Link, Output = Node<Type>>,
	{
		let parent = self[to].parent;
		
		if parent != Link::NIL
		{
			let parent_index = self.get_parent_index(from, parent);
			self[parent].descendants[parent_index as usize] = to;
		}
		
		for i in 0 .. 2
		{
			let des = self[to].descendants[i];
			
			if des != Link::NIL
			{
				self[des].parent = to;
			}
		}
	}
}

impl<Indexable, Type> AA<Type> for Indexable
where
	Indexable: ?Sized,
	Indexable: std::ops::Index<Link, Output = Node<Type>>,
{
}

/// Checks the AA tree invariants of the subtree, returns the number of its nodes.
#[cfg(test)]
pub(super) fn check_invariants<Type>(nodes: &[Node<Type>], root: Link) -> usize
{
	if root != Link::NIL
	{
		assert_eq!(Link::NIL, nodes[root].parent);
	}
	
	return check_subtree(nodes, root);
}

#[cfg(test)]
fn check_subtree<Type>(nodes: &[Node<Type>], index: Link) -> usize
{
	if index == Link::NIL
	{
		return 0;
	}
	
	let level = |i: Link| if i == Link::NIL {-1} else {nodes[i].level};
	let node = &nodes[index];
	let [ldes, rdes] = node.descendants;
	
	for des in [ldes, rdes]
	{
		if des != Link::NIL
		{
			assert_eq!(index, nodes[des].parent);
		}
	}
	
	assert_eq!(node.level - 1, level(ldes), "left child level of {}", index.index());
	assert!(level(rdes) == node.level || level(rdes) == node.level - 1, "right child level of {}", index.index());
	
	if rdes != Link::NIL
	{
		assert!(level(nodes[rdes].descendants[1]) < node.level, "right grandchild level of {}", index.index());
	}
	
	return 1 + check_subtree(nodes, ldes) + check_subtree(nodes, rdes);
}

pub struct Iterator<Nodes: ?Sized>
{
	#[allow(dead_code)] // Actually used by implementors
	pub(super) first: Link,
	#[allow(dead_code)] // Actually used by implementors
	pub(super) last: Link,
	pub(super) bounds: [Link; 2],
	pub(super) nodes: Nodes,
}

macro_rules! iter_impl
{
	($this: expr, $index: expr) =>
	{
		if $this.bounds[$index] == $crate::svst::aa::node::Link::NIL
		{
			$crate::svst::aa::node::Link::NIL
		}
		else
		{
			let result = $this.bounds[$index];
			
			let desc = $this.nodes[$this.bounds[$index]].descendants[1 - $index];
			
			if desc != $crate::svst::aa::node::Link::NIL
			{
				$this.bounds[$index] = desc;
				
				let mut descendant;
				
				while {descendant = $this.nodes[$this.bounds[$index]].descendants[$index]; descendant != $crate::svst::aa::node::Link::NIL}
				{
					$this.bounds[$index] = descendant;
				}
			}
			else
			{
				loop
				{
					let parent = $this.nodes[$this.bounds[$index]].parent;
					
					if parent == $crate::svst::aa::node::Link::NIL
					{
						$this.bounds[1 - $index] = $crate::svst::aa::node::Link::NIL;
					}
					else if $crate::svst::aa::node::AA::get_parent_index($this.nodes, $this.bounds[$index], parent) == 1 - $index
					{
						$this.bounds[$index] = parent;
						continue;
					}
					
					$this.bounds[$index] = parent;
					
					break;
				}
			}
			
			result
		}
	}
}

pub(super) use iter_impl;
