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

pub(super) fn get_parent_index<Nodes, Type>(nodes: &Nodes, index: usize, parent: usize) -> usize
where
	Nodes: ?Sized + std::ops::Index<usize, Output = Node<Type>>
{
	for i in 0 .. 2
	{
		if nodes[parent].descendants[i] == index
		{
			return i;
		}
	}
	
	unreachable!();
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
					else if $crate::svst::aa::node::get_parent_index($this.nodes, $this.bounds[$index], parent) == 1 - $index
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

pub(super) fn skew<Nodes, Type>(nodes: &mut Nodes, index: usize) -> usize
where
	Nodes: ?Sized + std::ops::IndexMut<usize, Output = Node<Type>>
{
	let l_index = nodes[index].descendants[0];
	
	if l_index == usize::MAX
	{
		return index;
	}
	else if nodes[index].level == nodes[l_index].level
	{
		let lrdesc = nodes[l_index].descendants[1];
		
		if lrdesc != usize::MAX
		{
			nodes[lrdesc].parent = index;
		}
		
		nodes[l_index].parent = nodes[index].parent;
		nodes[index].parent = l_index;
		
		nodes[index].descendants[0] = nodes[l_index].descendants[1];
		nodes[l_index].descendants[1] = index;
		
		return l_index;
	}
	
	return index;
}

pub(super) fn split<Nodes, Type>(nodes: &mut Nodes, index: usize) -> usize
where
	Nodes: ?Sized + std::ops::IndexMut<usize, Output = Node<Type>>
{
	let r_index = nodes[index].descendants[1];
	
	if r_index == usize::MAX || nodes[r_index].descendants[1] == usize::MAX
	{
		return index;
	}
	else if nodes[index].level == nodes[nodes[r_index].descendants[1]].level
	{
		let rldesc = nodes[r_index].descendants[0];
		
		if rldesc != usize::MAX
		{
			nodes[rldesc].parent = index;
		}
		
		nodes[r_index].parent = nodes[index].parent;
		nodes[index].parent = r_index;
		
		nodes[index].descendants[1] = nodes[r_index].descendants[0];
		nodes[r_index].descendants[0] = index;
		nodes[r_index].level = nodes[r_index].level + 1;
		
		return r_index;
	}
	
	return index;
}

pub(super) fn find<Key, Nodes, Type, Compare>(nodes: &Nodes, root: usize, key: &Key, comapre: &Compare) -> (usize, usize, usize)
where
	Nodes: ?Sized + std::ops::Index<usize, Output = Node<Type>>,
	Type: Entry,
	Type::Key: std::borrow::Borrow<Key>,
	Key: ?Sized,
	Compare: crate::Comparator<Key>,
{
	let mut desc = root;
	let mut parent = usize::MAX;
	let mut parent_index: usize = 0;
	
	while desc != usize::MAX
	{
		parent = desc;
		
		match comapre.compare(key, nodes[desc].value.key().borrow())
		{
			std::cmp::Ordering::Less =>
			{
				parent_index = 0;
				desc = nodes[desc].descendants[parent_index as usize];
			},
			std::cmp::Ordering::Greater =>
			{
				parent_index = 1;
				desc = nodes[desc].descendants[parent_index as usize];
			},
			std::cmp::Ordering::Equal =>
			{
				break;
			}
		}
	}
	
	return (desc, parent, parent_index);
}

pub(super) fn swap_nodes<Nodes, Type>(nodes: &mut Nodes, index: usize, successor: usize)
where
	Nodes: ?Sized + std::ops::IndexMut<usize, Output = Node<Type>>
{
	if index == successor
	{
		unreachable!();
	}
	
	let parent = nodes[index].parent;
	let successor_rdes = nodes[successor].descendants[1];
	
	if nodes[index].descendants[1] == successor
	{
		nodes[index].parent = successor;
		nodes[successor].descendants[1] = index;
	}
	else
	{
		nodes[index].parent = nodes[successor].parent;
		nodes[successor].descendants[1] = nodes[index].descendants[1];
		let successor_parent = nodes[successor].parent;
		nodes[successor_parent].descendants[0] = index;
		nodes[index].parent = successor_parent;
		let index = nodes[index].descendants[1];
		nodes[index].parent = successor;
	}
	
	nodes[successor].parent = parent;
	
	if parent != usize::MAX
	{
		let parent_index = get_parent_index(nodes, index, parent);
		nodes[parent].descendants[parent_index] = successor;
	}
	
	let index_ldes = nodes[index].descendants[0];
	nodes[successor].descendants[0] = index_ldes;
	nodes[index].descendants[0] = usize::MAX;
	
	if index_ldes != usize::MAX
	{
		nodes[index_ldes].parent = successor;
	}
	
	nodes[index].descendants[1] = successor_rdes;
	
	{
		let level = nodes[index].level;
		nodes[index].level = nodes[successor].level;
		nodes[successor].level = level;
	}
	
// 	NOTE unnecessary, this is set in `erase_rebalance`
// 	if (successor_rdes != -1)
// 	{
// 		set_parent(nodes[successor_rdes], index);
// 	}
}

const CHANGE_PROPAGATION_DISTANCE: i32 = 3;

pub(super) fn insert_rebalance<Nodes, Type>(nodes: &mut Nodes, mut parent: usize,
	mut parent_index: usize, mut index: usize) -> bool
where
	Nodes: ?Sized + std::ops::IndexMut<usize, Output = Node<Type>>
{
	nodes[index].parent = parent;
	nodes[parent].descendants[parent_index] = index;
	
	let mut changes = CHANGE_PROPAGATION_DISTANCE;
	
	while {index = parent; parent = nodes[parent].parent;
		parent != usize::MAX && changes > 0
	}
	{
		parent_index = get_parent_index(nodes, index, parent);
		
		changes -= 1;
		
		let nv = skew(nodes, index);
		
		if nv != index
		{
			index = nv;
			changes = CHANGE_PROPAGATION_DISTANCE;
		}
		
		let nv = split(nodes, index);
		
		if nv != index
		{
			index = nv;
			changes = CHANGE_PROPAGATION_DISTANCE;
		}
		
		nodes[index].parent = parent;
		nodes[parent].descendants[parent_index] = index;
	}
	
	return changes > 0;
}

pub(super) fn erase_rebalance_leaf<Nodes, Type>(nodes: &mut Nodes, mut index: usize) -> usize
where
	Nodes: ?Sized + std::ops::IndexMut<usize, Output = Node<Type>>
{
	if nodes[index].level != 0
	{
		unreachable!();
	}
	
	let mut rdes = nodes[index].descendants[1];
	let mut parent = nodes[index].parent;
	
	if rdes != usize::MAX
	{
		nodes[rdes].parent = parent;
	}
	
	if parent == usize::MAX
	{
		return nodes[index].descendants[1];
	}
	
	{
		let parent_index = get_parent_index(nodes, index, parent);
		nodes[parent].descendants[parent_index] = rdes;
	}
	
	let mut changes = CHANGE_PROPAGATION_DISTANCE;
	
	loop
	{
		changes -= 1;
		index = parent;
		parent = nodes[parent].parent;
		let mut parent_index = usize::MAX;
		
		if parent != usize::MAX
		{
			parent_index = get_parent_index(nodes, index, parent);
		}
		
		let mut level = -1;
		
		{
			let ldes = nodes[index].descendants[0];
			
			if ldes != usize::MAX
			{
				level = nodes[ldes].level;
			}
		}
		
		rdes = nodes[index].descendants[1];
		
		if rdes != usize::MAX
		{
			let rlevel = nodes[rdes].level;
			
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
		
		if level < nodes[index].level
		{
			changes = CHANGE_PROPAGATION_DISTANCE;
			
			nodes[index].level = level;
			
			if rdes != usize::MAX && level < nodes[rdes].level
			{
				nodes[rdes].level = level;
			}
		}
		
		{
			let new_index = skew(nodes, index);
			
			if new_index != index
			{
				index = new_index;
				changes = CHANGE_PROPAGATION_DISTANCE;
			}
		}
		{
			let mut rdes = nodes[index].descendants[1];
			
			if rdes != usize::MAX
			{
				rdes = skew(nodes, rdes);
				
				if rdes != nodes[index].descendants[1]
				{
					nodes[index].descendants[1] = rdes;
					changes = CHANGE_PROPAGATION_DISTANCE;
				}
				
				let mut rrdes = nodes[rdes].descendants[1];
				
				if rrdes != usize::MAX
				{
					rrdes = skew(nodes, rrdes);
					
					if rrdes != nodes[rdes].descendants[1]
					{
						nodes[rdes].descendants[1] = rrdes;
						changes = CHANGE_PROPAGATION_DISTANCE;
					}
				}
			}
		}
		{
			let new_index = split(nodes, index);
			
			if new_index != index
			{
				index = new_index;
				changes = CHANGE_PROPAGATION_DISTANCE;
			}
		}
		{
			let mut rdes = nodes[index].descendants[1];
			
			if rdes != usize::MAX
			{
				rdes = split(nodes, rdes);
				
				if rdes != nodes[index].descendants[1]
				{
					nodes[index].descendants[1] = rdes;
					changes = CHANGE_PROPAGATION_DISTANCE;
				}
			}
		}
		
		if parent != usize::MAX
		{
			nodes[parent].descendants[parent_index] = index;
		}
		
		if parent == usize::MAX || changes == 0
		{
			break;
		}
	}
	
	return if parent == usize::MAX {index} else {usize::MAX};
}

pub(super) fn find_successor<Nodes, Type>(nodes: &mut Nodes, mut index: usize) -> usize
where
	Nodes: ?Sized + std::ops::IndexMut<usize, Output = Node<Type>>
{
	if nodes[index].level == 0
	{
		return usize::MAX;
	}
	
	index = nodes[index].descendants[1];
	
	while nodes[index].level > 0
	{
		index = nodes[index].descendants[0];
	}
	
	return index;
}

pub(super) fn erase_rebalance<Nodes, Type>(nodes: &mut Nodes, index: usize) -> usize
where
	Nodes: ?Sized + std::ops::IndexMut<usize, Output = Node<Type>>
{
	let successor = find_successor(nodes, index);
	
	if successor != usize::MAX
	{
		swap_nodes(nodes, index, successor);
	}
	
	return erase_rebalance_leaf(nodes, index);
}
