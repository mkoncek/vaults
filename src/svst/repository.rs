use crate::svst::vector_storage::VectorStorage;
use crate::svst::bit_indexing;
#[cfg(test)] use rand::seq::SliceRandom;

/// A data structure holding values of type `Type`.
/// It is backed by vector-like storage and grows dynamically, similar to [Vec].
/// 
/// The difference is that the removal of values from this structure does not cause shifting of the subsequent values to fill the empty space.
/// Therefore after removing values from anywhere but the end, this structure will have "holes".
/// Repository keeps track of these holes and they will be filled on subsequent `insert` operations.
/// 
/// The metadata is stored as a perfectly balanced tree of 128-bit bitsets which are stored together with the storage allocated for the values.
/// This means that despite the `insert` and `remove` operations being _O(log n)_, the logarithm has a base of 128 and the tree is extremely flat.
#[derive(Debug)]
pub struct Repository<Type>
{
	storage: VectorStorage,
	len: usize,
	index_length: usize,
	_data: std::marker::PhantomData<Type>,
}

impl<Type> Repository<Type>
{
	/// Constructs a new, empty `Repository<Type>`.
	pub const fn new() -> Self
	{
		Self
		{
			storage: VectorStorage::new::<Type>(),
			len: 0,
			index_length: 0,
			_data: std::marker::PhantomData,
		}
	}
	
	/// Constructs a new, empty `Repository<Type>` with at least the specified capacity.
	pub fn with_capacity(capacity: usize) -> Self
	{
		let mut result = Self::new();
		result.reserve(capacity);
		return result;
	}
	
	/// Returns the total number of values the repository can hold without reallocating.
	pub fn capacity(&self) -> usize {self.storage.capacity}
	
	/// Reserves capacity for at least `additional` more values to be inserted in the repository.
	pub fn reserve(&mut self, additional: usize)
	{
		if self.capacity() < self.len() + additional
		{
			let additional_exact = VectorStorage::default_capacity_for(self.capacity(), additional + self.capacity());
			self.reserve_exact_unchecked(additional_exact);
		}
	}
	
	/// Reserves capacity for at least `additional` more values to be inserted in the repository without excessive over-allocation.
	pub fn reserve_exact(&mut self, additional: usize)
	{
		if self.capacity() < self.len() + additional
		{
			self.reserve_exact_unchecked(additional);
		}
	}
	
	/// Returns a slice containing the values of the repository.
	/// Note that the slice may contain dropped values.
	pub unsafe fn as_slice(&self) -> &[Type]
	{
		unsafe {std::slice::from_raw_parts(self.storage.data.as_ptr()
			.offset(Self::array_offset(self.index_length) as isize).cast::<Type>(), self.capacity()
		)}
	}
	
	/// Returns a mutable slice containing the values of the repository.
	/// Note that the slice may contain dropped values.
	pub unsafe fn as_mut_slice(&mut self) -> &mut [Type]
	{
		unsafe {std::slice::from_raw_parts_mut(self.storage.data.as_ptr()
			.offset(Self::array_offset(self.index_length) as isize).cast::<Type>(), self.capacity()
		)}
	}
	
	/// Inserts a value in the repository returning its index within the repository.
	/// The inserted value will remain at the position of the returned index for the whole lifetime of the repository or until the value is explicitly removed.
	/// # Time complexity
	/// Amortized _O(log<sub>128</sub> n)_ where _n_ is the number of values in the repository.
	pub fn insert(&mut self, value: Type) -> usize
	{
		self.reserve(1);
		let capacity = self.capacity();
		let index = bit_indexing::push_front(self.index_header_mut(), capacity);
		
		unsafe
		{
			self.storage.data.as_ptr().offset(Self::array_offset(self.index_length) as isize)
				.cast::<Type>().offset(index as isize).write(value)
			;
		}
		
		self.len += 1;
		
		return index;
	}
	
	/// Removes a value at _index_ from the repository, returning it or [None].
	/// # Time complexity
	/// _O(log<sub>128</sub> n)_ where _n_ is the number of values in the repository.
	pub fn remove(&mut self, index: usize) -> Option<Type>
	{
		let capacity = self.capacity();
		let mut result = None;
		
		if index < capacity && bit_indexing::erase(self.index_header_mut(), index, capacity)
		{
			unsafe
			{
				result = Some(self.storage.data.as_ptr().offset(Self::array_offset(self.index_length) as isize)
					.cast::<Type>().offset(index as isize).read()
				);
			}
			
			self.len -= 1;
		}
		
		return result;
	}
	
	/// Removes a value at _index_ from the repository, returning it.
	/// # Time complexity
	/// _O(log<sub>128</sub> n)_ where _n_ is the number of values in the repository.
	pub unsafe fn remove_unchecked(&mut self, index: usize) -> Type
	{
		let capacity = self.capacity();
		bit_indexing::erase(self.index_header_mut(), index, capacity);
		self.len -= 1;
		return self.storage.data.as_ptr().offset(Self::array_offset(self.index_length) as isize)
			.cast::<Type>().offset(index as isize).read()
		;
	}
	
	/// Clears the repository, removing all values.
	pub fn clear(&mut self)
	{
		if self.capacity() != 0
		{
			self.simple_clear();
			self.len = 0;
			self.index_header_mut().fill(0);
		}
	}
	
	/// Returns the number of values in the repository.
	pub fn len(&self) -> usize {self.len}
	
	/// Returns `true` if the repository contains no values.
	pub fn is_empty(&self) -> bool {self.len == 0}
	
	/// Returns an iterator over the **indices** of values present in the repository.
	pub fn index_iter(&self) -> impl std::iter::Iterator<Item = usize> + '_
	{
		bit_indexing::IndexSliceIterator::new(&self.index_header_leaf())
	}
	
	pub unsafe fn get_unchecked(&self, index: usize) -> &Type
	{
		self.as_slice().get_unchecked(index)
	}
	
	pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut Type
	{
		self.as_mut_slice().get_unchecked_mut(index)
	}
	
	pub fn get(&self, index: usize) -> Option<&Type>
	{
		if index < self.capacity()
		{
			let (slice_idx, mask) = bit_indexing::indices(index);
			unsafe
			{
				if self.index_header_leaf().get_unchecked(slice_idx) & mask != 0
				{
					return Some(self.as_slice().get_unchecked(index));
				}
			}
		}
		
		return None;
	}
	
	pub fn get_mut(&mut self, index: usize) -> Option<&mut Type>
	{
		if index < self.capacity()
		{
			let (slice_idx, mask) = bit_indexing::indices(index);
			unsafe
			{
				if self.index_header_leaf().get_unchecked(slice_idx) & mask != 0
				{
					return Some(self.as_mut_slice().get_unchecked_mut(index));
				}
			}
		}
		
		return None;
	}
	
	/// Returns an iterator over the values present in the repository.
	pub fn iter(&self) -> impl std::iter::Iterator<Item = &Type>
	{
		self.into_iter()
	}
	
	/// Returns a mutable iterator over the values present in the repository.
	pub fn iter_mut(&mut self) -> impl std::iter::Iterator<Item = &mut Type>
	{
		self.into_iter()
	}
	
	fn simple_clear(&mut self)
	{
		let array_offset = Self::array_offset(self.index_length);
		
		for i in self.index_iter()
		{
			unsafe
			{
				self.storage.data.as_ptr().offset(array_offset as isize)
					.cast::<Type>().offset(i as isize).drop_in_place()
				;
			};
		}
	}
	
	fn array_offset(index_length: usize) -> usize
	{
		let type_alignment = std::mem::align_of::<Type>();
		return (index_length * std::mem::size_of::<bit_indexing::IndexType>() as usize
			+ type_alignment - 1) / type_alignment * type_alignment
		;
	}
	
	fn layout_for(capacity: usize) -> (std::alloc::Layout, usize)
	{
		let alignment = std::cmp::max(
			std::mem::align_of::<Type>(),
			std::mem::align_of::<bit_indexing::IndexType>(),
		);
		let index_length = crate::svst::bit_indexing::index_length(capacity);
		let byte_size = Self::array_offset(index_length) + std::mem::size_of::<Type>() * capacity;
		
		return (std::alloc::Layout::from_size_align(byte_size, alignment).unwrap(), index_length);
	}
	
	fn index_header(&self) -> &[bit_indexing::IndexType]
	{
		unsafe {std::slice::from_raw_parts(
			self.storage.data.as_ptr().cast::<bit_indexing::IndexType>(), self.index_length
		)}
	}
	
	fn index_header_mut(&mut self) -> &mut [bit_indexing::IndexType]
	{
		unsafe {std::slice::from_raw_parts_mut(
			self.storage.data.as_ptr().cast::<bit_indexing::IndexType>(), self.index_length
		)}
	}
	
	pub(super) fn index_header_leaf(&self) -> &[bit_indexing::IndexType]
	{
		&self.index_header()[self.index_length - bit_indexing::level_length(self.capacity()) .. self.index_length]
	}
	
	fn reserve_exact_unchecked(&mut self, additional: usize)
	{
		let capacity = self.len() + additional;
		let (new_layout, index_length) = Self::layout_for(capacity);
		let new_data = match std::ptr::NonNull::new(unsafe {std::alloc::alloc(new_layout)})
		{
			Some(p) => p,
			None => std::alloc::handle_alloc_error(new_layout),
		};
		
		for i in 0 .. index_length
		{
			unsafe {(new_data.as_ptr().cast::<bit_indexing::IndexType>()).offset(i as isize).write(0)};
		}
		
		if self.capacity() != 0
		{
			unsafe
			{
				bit_indexing::copy(
					std::slice::from_raw_parts(self.storage.data.as_ptr().cast::<bit_indexing::IndexType>(), self.index_length),
					self.capacity(),
					std::slice::from_raw_parts_mut(new_data.as_ptr().cast::<bit_indexing::IndexType>(), index_length),
					capacity,
				)
			};
			
			let array_offset = Self::array_offset(self.index_length);
			let new_array_offset = Self::array_offset(index_length);
			
			for i in self.index_iter()
			{
				unsafe
				{
					self.storage.data.as_ptr().offset(array_offset as isize).cast::<Type>()
						.offset(i as isize).copy_to_nonoverlapping(new_data.as_ptr().offset(new_array_offset as isize)
						.cast::<Type>().offset(i as isize), 1)
					;
				};
			}
			
			unsafe {std::alloc::dealloc(self.storage.data.as_ptr(), Self::layout_for(self.capacity()).0)};
		}
		
		self.storage.data = new_data;
		self.storage.capacity = capacity;
		self.index_length = index_length;
	}
}

impl<Type> Drop for Repository<Type>
{
	fn drop(&mut self)
	{
		if self.capacity() != 0
		{
			self.simple_clear();
			for i in 0 .. self.index_length
			{
				unsafe
				{
					self.storage.data.as_ptr().cast::<bit_indexing::IndexType>().offset(i as isize).drop_in_place();
				}
			}
			unsafe {std::alloc::dealloc(self.storage.data.as_ptr(), Self::layout_for(self.capacity()).0)};
		}
	}
}

impl<Type> Default for Repository<Type>
{
	fn default() -> Self {Self::new()}
}

impl<Type> AsRef<Repository<Type>> for Repository<Type>
{
	fn as_ref(&self) -> &Repository<Type> {self}
}

impl<Type> AsMut<Repository<Type>> for Repository<Type>
{
	fn as_mut(&mut self) -> &mut Repository<Type> {self}
}

impl<Type> Clone for Repository<Type>
where Type: Clone
{
	fn clone(&self) -> Self
	{
		let mut result = Self::new();
		result.clone_from(self);
		return result;
	}
	
	fn clone_from(&mut self, source: &Self)
	{
		self.clear();
		
		if self.capacity() < source.capacity()
		{
			let mut capacity = 0;
			let header = source.index_header_leaf();
			
			for i in (0 .. header.len()).rev()
			{
				if header[i] != 0
				{
					capacity = bit_indexing::IndexType::BITS as usize * i;
					break;
				}
			}
			
			if self.capacity() < capacity
			{
				*self = Self::with_capacity(capacity);
			}
		}
		
		todo!()
	}
}

impl<Type> FromIterator<Type> for Repository<Type>
{
	fn from_iter<T: IntoIterator<Item = Type>>(iter: T) -> Self
	{
		let iter = iter.into_iter();
		let mut result = Self::with_capacity(iter.size_hint().0);
		for v in iter
		{
			result.insert(v);
		}
		return result;
	}
}

pub struct Iter<'t, Type>
{
	it: bit_indexing::TransientIndexSliceIterator,
	repository: &'t Repository<Type>,
}

impl<'t, Type> std::iter::Iterator for Iter<'t, Type>
{
	type Item = &'t Type;
	fn next(&mut self) -> Option<Self::Item>
	{
		self.it.next(self.repository.index_header_leaf()).map(move |i| &self.repository[i])
	}
}

impl<'t, Type> IntoIterator for &'t Repository<Type>
{
	type Item = &'t Type;
	type IntoIter = Iter<'t, Type>;
	
	fn into_iter(self) -> Self::IntoIter
	{
		Self::IntoIter
		{
			it: bit_indexing::TransientIndexSliceIterator::new(self.index_header_leaf()),
			repository: self,
		}
	}
}

pub struct IterMut<'t, Type>
{
	it: bit_indexing::TransientIndexSliceIterator,
	repository: &'t mut Repository<Type>,
}

impl<'t, Type> std::iter::Iterator for IterMut<'t, Type>
{
	type Item = &'t mut Type;
	fn next(&mut self) -> Option<Self::Item>
	{
		let Some(i) = self.it.next(self.repository.index_header_leaf()) else
		{
			return None;
		};
		unsafe {Some(std::ptr::addr_of_mut!(self.repository[i]).as_mut().unwrap())}
	}
}

impl<'t, Type> IntoIterator for &'t mut Repository<Type>
{
	type Item = &'t mut Type;
	type IntoIter = IterMut<'t, Type>;
	
	fn into_iter(self) -> Self::IntoIter
	{
		Self::IntoIter
		{
			it: bit_indexing::TransientIndexSliceIterator::new(self.index_header_leaf()),
			repository: self,
		}
	}
}

pub struct IterVal<Type>
{
	it: bit_indexing::TransientIndexSliceIterator,
	repository: Repository<Type>,
}

impl<Type> std::iter::Iterator for IterVal<Type>
{
	type Item = Type;
	fn next(&mut self) -> Option<Self::Item>
	{
		let Some(i) = self.it.next(self.repository.index_header_leaf()) else
		{
			return None;
		};
		
		unsafe {Some(self.repository.remove_unchecked(i))}
	}
}

impl<Type> IntoIterator for Repository<Type>
{
	type Item = Type;
	type IntoIter = IterVal<Type>;
	
	fn into_iter(self) -> Self::IntoIter
	{
		Self::IntoIter
		{
			it: bit_indexing::TransientIndexSliceIterator::new(self.index_header_leaf()),
			repository: self,
		}
	}
}

impl<Type> std::ops::Index<usize> for Repository<Type>
{
	type Output = Type;
	
	fn index(&self, index: usize) -> &Self::Output
	{
		if index < self.capacity()
		{
			let (slice_idx, mask) = bit_indexing::indices(index);
			unsafe
			{
				if self.index_header_leaf().get_unchecked(slice_idx) & mask != 0
				{
					return self.as_slice().get_unchecked(index);
				}
			}
		}
		
		panic!("index {} contains an invalid value", index);
	}
}

impl<Type> std::ops::IndexMut<usize> for Repository<Type>
{
	fn index_mut(&mut self, index: usize) -> &mut Self::Output
	{
		if index < self.capacity()
		{
			let (slice_idx, mask) = bit_indexing::indices(index);
			unsafe
			{
				if self.index_header_leaf().get_unchecked(slice_idx) & mask != 0
				{
					return self.as_mut_slice().get_unchecked_mut(index);
				}
			}
		}
		
		panic!("index {} contains an invalid value", index);
	}
}

#[test]
fn test_repository()
{
	let mut idxs = Vec::<usize>::new();
	let mut rp = Repository::<Box<i32>>::new();
	let max_limit = 100;
	
	for i in 0 .. max_limit
	{
		assert_eq!(i, rp.len());
		assert!(i <= rp.capacity());
		idxs.push(rp.insert(Box::new(0)));
	}
	
	idxs.shuffle(&mut rand::thread_rng());
	
	let limit = 45;
	
	for i in 0 .. limit
	{
		assert_eq!(max_limit - i, rp.len());
		rp.remove(idxs.pop().unwrap()).unwrap();
	}
	
	for i in 0 .. limit
	{
		assert_eq!(max_limit - limit + i, rp.len());
		idxs.push(rp.insert(Box::new(0)));
	}
	
	idxs.shuffle(&mut rand::thread_rng());
	
	for i in 0 .. limit
	{
		assert_eq!(max_limit - i, rp.len());
		rp.remove(idxs.pop().unwrap()).unwrap();
	}
	
	for i in 0 .. limit
	{
		assert_eq!(max_limit - limit + i, rp.len());
		idxs.push(rp.insert(Box::new(99)));
	}
}

#[test]
fn test_out_of_bounds()
{
	let mut r = Repository::new();
	assert_eq!(None, r.get(0));
	assert_eq!(None, r.get_mut(0));
	assert_eq!(None, r.get(1));
	assert_eq!(None, r.get_mut(1));
	assert_eq!(None, r.get(usize::MAX));
	assert_eq!(None, r.get_mut(usize::MAX));
	assert_eq!(None, r.remove(0));
	assert_eq!(None, r.remove(1));
	
	r.insert(0);
	assert_eq!(Some(0), r.get(0).copied());
	assert_eq!(Some(0), r.get_mut(0).copied());
	assert_eq!(None, r.get(1));
	assert_eq!(None, r.get_mut(1));
	
	assert_eq!(None, r.remove(1));
}

#[test]
fn test_clear()
{
	let mut r = Repository::new();
	for i in 0 .. 1000
	{
		r.insert(i);
	}
	
	r.clear();
	assert!(r.is_empty());
	assert_eq!(0, r.len());
	
	for i in 0 .. 1000
	{
		r.insert(i);
	}
	
	assert_eq!(1000, r.len());
	
	r.clear();
	assert!(r.is_empty());
	assert_eq!(0, r.len());
}

#[test]
fn test_remove()
{
	let mut r = Repository::new();
	for i in 0 .. 1000
	{
		r.insert(i);
	}
	
	let mut len = r.len();
	for i in (0 .. 1000).step_by(2)
	{
		assert_eq!(Some(i), r.remove(i));
		len -= 1;
		assert_eq!(len, r.len());
	}
}

#[test]
fn test_empty_type()
{
	let mut r = Repository::<()>::new();
	r.insert(());
	assert_eq!(1, r.len());
	assert_eq!((), r[0]);
}
