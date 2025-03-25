use std::borrow::Borrow;

pub trait Entry
{
	type Key;
	type Value;
	fn key(&self) -> &Self::Key;
	fn value(self) -> Self::Value;
}

#[derive(Debug)]
pub struct Node<Type>
{
	pub(super) parent: usize,
	pub(super) descendants: [usize; 2],
	pub(super) level: i16,
	value: Type,
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
			parent: usize::MAX,
			descendants: [usize::MAX, usize::MAX],
			level: 0,
			value,
		}
	}
	
	pub fn value(self) -> Type {self.value}
}

pub(super) trait AA<Type>: std::ops::Index<usize, Output = Node<Type>>
{
	fn get_parent_index(&self, index: usize, parent: usize) -> u8
	{
		for i in 0 .. 2
		{
			if self[parent].descendants[i as usize] == index
			{
				return i;
			}
		}
		
		unreachable!();
	}
	
	fn skew(&mut self, index: usize) -> usize
	where
		Self: std::ops::IndexMut<usize, Output = Node<Type>>,
	{
		let l_index = self[index].descendants[0];
		
		if l_index == usize::MAX
		{
			return index;
		}
		else if self[index].level == self[l_index].level
		{
			let lrdesc = self[l_index].descendants[1];
			
			if lrdesc != usize::MAX
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
	
	fn split(&mut self, index: usize) -> usize
	where
		Self: std::ops::IndexMut<usize, Output = Node<Type>>,
	{
		let r_index = self[index].descendants[1];
		
		if r_index == usize::MAX || self[r_index].descendants[1] == usize::MAX
		{
			return index;
		}
		else if self[index].level == self[self[r_index].descendants[1]].level
		{
			let rldesc = self[r_index].descendants[0];
			
			if rldesc != usize::MAX
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
	
	fn find<Key, Compare>(&self, root: usize, key: &Key, comapre: Compare) -> (usize, usize, u8)
	where
		Type: Entry,
		Key: ?Sized,
		Compare: Fn(&Key, &Type::Key) -> std::cmp::Ordering,
	{
		let mut desc = root;
		let mut parent = usize::MAX;
		let mut parent_index: u8 = 0;
		
		while desc != usize::MAX
		{
			parent = desc;
			
			match comapre(key, self[desc].value.key())
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
	
	fn swap_nodes(&mut self, index: usize, successor: usize)
	where
		Self: std::ops::IndexMut<usize, Output = Node<Type>>,
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
		
		if parent != usize::MAX
		{
			let parent_index = self.get_parent_index(index, parent);
			self[parent].descendants[parent_index as usize] = successor;
		}
		
		let index_ldes = self[index].descendants[0];
		self[successor].descendants[0] = index_ldes;
		self[index].descendants[0] = usize::MAX;
		
		if index_ldes != usize::MAX
		{
			self[index_ldes].parent = successor;
		}
		
		self[index].descendants[1] = successor_rdes;
		
		{
			let level = self[index].level;
			self[index].level = self[successor].level;
			self[successor].level = level;
		}
		
	// 	NOTE unnecessary, this is set in `erase_rebalance`
	// 	if (successor_rdes != -1)
	// 	{
	// 		set_parent(self[successor_rdes], index);
	// 	}
	}
	
	const CHANGE_PROPAGATION_DISTANCE: i32 = 3;
	
	fn insert_rebalance(&mut self, mut parent: usize,
		mut parent_index: u8, mut index: usize) -> bool
	where
		Self: std::ops::IndexMut<usize, Output = Node<Type>>,
	{
		self[index].parent = parent;
		self[parent].descendants[parent_index as usize] = index;
		
		let mut changes = Self::CHANGE_PROPAGATION_DISTANCE;
		
		while {index = parent; parent = self[parent].parent;
			parent != usize::MAX && changes > 0
		}
		{
			parent_index = self.get_parent_index(index, parent);
			
			changes -= 1;
			
			let nv = self.skew(index);
			
			if nv != index
			{
				index = nv;
				changes = Self::CHANGE_PROPAGATION_DISTANCE;
			}
			
			let nv = self.split(index);
			
			if nv != index
			{
				index = nv;
				changes = Self::CHANGE_PROPAGATION_DISTANCE;
			}
			
			self[index].parent = parent;
			self[parent].descendants[parent_index as usize] = index;
		}
		
		return changes > 0;
	}
	
	fn erase_rebalance_leaf(&mut self, mut index: usize) -> usize
	where
		Self: std::ops::IndexMut<usize, Output = Node<Type>>,
	{
		if self[index].level != 0
		{
			unreachable!();
		}
		
		let mut rdes = self[index].descendants[1];
		let mut parent = self[index].parent;
		
		if rdes != usize::MAX
		{
			self[rdes].parent = parent;
		}
		
		if parent == usize::MAX
		{
			return self[index].descendants[1];
		}
		
		{
			let parent_index = self.get_parent_index(index, parent);
			self[parent].descendants[parent_index as usize] = rdes;
		}
		
		let mut changes = Self::CHANGE_PROPAGATION_DISTANCE;
		
		loop
		{
			changes -= 1;
			index = parent;
			parent = self[parent].parent;
			let mut parent_index = 0;
			
			if parent != usize::MAX
			{
				parent_index = self.get_parent_index(index, parent);
			}
			
			let mut level = -1;
			
			{
				let ldes = self[index].descendants[0];
				
				if ldes != usize::MAX
				{
					level = self[ldes].level;
				}
			}
			
			rdes = self[index].descendants[1];
			
			if rdes != usize::MAX
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
				changes = Self::CHANGE_PROPAGATION_DISTANCE;
				
				self[index].level = level;
				
				if rdes != usize::MAX && level < self[rdes].level
				{
					self[rdes].level = level;
				}
			}
			
			{
				let new_index = self.skew(index);
				
				if new_index != index
				{
					index = new_index;
					changes = Self::CHANGE_PROPAGATION_DISTANCE;
				}
			}
			{
				let mut rdes = self[index].descendants[1];
				
				if rdes != usize::MAX
				{
					rdes = self.skew(rdes);
					
					if rdes != self[index].descendants[1]
					{
						self[index].descendants[1] = rdes;
						changes = Self::CHANGE_PROPAGATION_DISTANCE;
					}
					
					let mut rrdes = self[rdes].descendants[1];
					
					if rrdes != usize::MAX
					{
						rrdes = self.skew(rrdes);
						
						if rrdes != self[rdes].descendants[1]
						{
							self[rdes].descendants[1] = rrdes;
							changes = Self::CHANGE_PROPAGATION_DISTANCE;
						}
					}
				}
			}
			{
				let new_index = self.split(index);
				
				if new_index != index
				{
					index = new_index;
					changes = Self::CHANGE_PROPAGATION_DISTANCE;
				}
			}
			{
				let mut rdes = self[index].descendants[1];
				
				if rdes != usize::MAX
				{
					rdes = self.split(rdes);
					
					if rdes != self[index].descendants[1]
					{
						self[index].descendants[1] = rdes;
						changes = Self::CHANGE_PROPAGATION_DISTANCE;
					}
				}
			}
			
			if parent != usize::MAX
			{
				self[parent].descendants[parent_index as usize] = index;
			}
			
			if parent == usize::MAX || changes == 0
			{
				break;
			}
		}
		
		return if parent == usize::MAX {index} else {usize::MAX};
	}
	
	fn find_successor(&mut self, mut index: usize) -> usize
	{
		if self[index].level == 0
		{
			return usize::MAX;
		}
		
		index = self[index].descendants[1];
		
		while self[index].level > 0
		{
			index = self[index].descendants[0];
		}
		
		return index;
	}
	
	fn erase_rebalance(&mut self, index: usize) -> usize
	where
		Self: std::ops::IndexMut<usize, Output = Node<Type>>,
	{
		let successor = self.find_successor(index);
		
		if successor != usize::MAX
		{
			self.swap_nodes(index, successor);
		}
		
		return self.erase_rebalance_leaf(index);
	}
	
	fn swap_positions(&mut self, removed: usize, last: usize)
	where
		Self: std::ops::IndexMut<usize, Output = Node<Type>>,
	{
		if removed == last
		{
			unreachable!();
		}
		
		let removed_parent = self[removed].parent;
		let last_parent_index = self.get_parent_index(last, self[last].parent);
		self[removed_parent].descendants[last_parent_index as usize] = removed;
		
		for i in 0 .. 2
		{
			let des = self[last].descendants[i];
			self[des].parent = removed;
		}
		
		unsafe {std::ptr::swap(std::ptr::from_mut(&mut self[removed]), std::ptr::from_mut(&mut self[last]))};
	}
}

impl<Indexable, Type> AA<Type> for Indexable
where
	Indexable: ?Sized,
	Indexable: std::ops::Index<usize, Output = Node<Type>>,
{
}

pub struct Iterator<Nodes: ?Sized>
{
	#[allow(dead_code)] // Actually used by implementors
	pub(super) first: usize,
	#[allow(dead_code)] // Actually used by implementors
	pub(super) last: usize,
	pub(super) bounds: [usize; 2],
	pub(super) nodes: Nodes,
}

macro_rules! iter_impl
{
	($this: expr, $index: expr) =>
	{
		if $this.bounds[$index] == usize::MAX
		{
			usize::MAX
		}
		else
		{
			let result = $this.bounds[$index];
			
			let desc = $this.nodes[$this.bounds[$index]].descendants[1 - $index];
			
			if desc != usize::MAX
			{
				$this.bounds[$index] = desc;
				
				let mut descendant;
				
				while {descendant = $this.nodes[$this.bounds[$index]].descendants[$index]; descendant != usize::MAX}
				{
					$this.bounds[$index] = descendant;
				}
			}
			else
			{
				loop
				{
					let parent = $this.nodes[$this.bounds[$index]].parent;
					
					if parent == usize::MAX
					{
						$this.bounds[1 - $index] = usize::MAX;
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
