use crate::svst::vector_storage::VectorStorage;
use crate::svst::bit_indexing;
#[cfg(test)] use rand::seq::SliceRandom;

/// A sparse, index-stable container backed by a flat array.
///
/// Unlike [`Vec`], removing a value does not shift subsequent elements.
/// Instead the vacated slot becomes a hole which is reused by the next [`insert`](Self::insert).
/// This means that the index returned by `insert` remains valid until the value at
/// that index is explicitly removed -- insertions and removals of *other* values never
/// invalidate it.
///
/// Occupancy is tracked by a tree of 64-bit bitsets stored in the same allocation as
/// the values.  `insert` and `remove` are amortized *O*(log_{64} *n*), which is
/// at most 4 levels for 16 million elements and at most 11 levels for any `usize` range.
#[derive(Debug)]
pub struct Repository<Type>
{
	storage: VectorStorage,
	len: usize,
	index_length: usize,
	/// All words of the level of the values in the index before this one are full,
	/// so insertion finds the lowest vacant index without searching the index if this word is not full.
	first_vacant_word: usize,
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
			first_vacant_word: 0,
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
	#[inline]
	pub fn reserve(&mut self, additional: usize)
	{
		if additional > self.capacity() - self.len()
		{
			self.grow(additional);
		}
	}
	
	/// Grows the capacity by the default growth until there is room for `additional` more values.
	#[cold]
	#[inline(never)]
	fn grow(&mut self, additional: usize)
	{
		let required = self.len().checked_add(additional).expect("capacity overflow");
		self.reallocate(VectorStorage::default_capacity_for(self.capacity(), required));
	}
	
	/// Reserves capacity for at least `additional` more values to be inserted in the repository without excessive over-allocation.
	pub fn reserve_exact(&mut self, additional: usize)
	{
		if additional > self.capacity() - self.len()
		{
			self.reallocate(self.len().checked_add(additional).expect("capacity overflow"));
		}
	}
	
	/// Returns a slice containing the values of the repository.
	/// Note that the slice may contain dropped values.
	#[inline]
	pub unsafe fn as_slice(&self) -> &[Type]
	{
		std::slice::from_raw_parts(self.values(), self.capacity())
	}
	
	/// Returns a mutable slice containing the values of the repository.
	/// Note that the slice may contain dropped values.
	#[inline]
	pub unsafe fn as_mut_slice(&mut self) -> &mut [Type]
	{
		std::slice::from_raw_parts_mut(self.values(), self.capacity())
	}
	
	/// Inserts a value in the repository returning its index within the repository.
	/// The inserted value will remain at the position of the returned index for the whole lifetime of the repository or until the value is explicitly removed.
	/// # Time complexity
	/// Amortized _O(log<sub>64</sub> n)_ where _n_ is the number of values in the repository.
	#[inline]
	pub fn insert(&mut self, value: Type) -> usize
	{
		self.reserve(1);
		let capacity = self.capacity();
		let mut hint = self.first_vacant_word;
		let index = bit_indexing::push_front_from(self.index_header_mut(), capacity, &mut hint);
		self.first_vacant_word = hint;
		
		unsafe {self.values().add(index).write(value)};
		
		self.len += 1;
		
		return index;
	}
	
	/// Removes a value at _index_ from the repository, returning it or [None].
	/// # Time complexity
	/// _O(log<sub>64</sub> n)_ where _n_ is the number of values in the repository.
	#[inline]
	pub fn remove(&mut self, index: usize) -> Option<Type>
	{
		let capacity = self.capacity();
		let mut result = None;
		
		if index < capacity && bit_indexing::erase(self.index_header_mut(), index, capacity)
		{
			self.first_vacant_word = self.first_vacant_word.min(index / bit_indexing::IndexType::BITS as usize);
			unsafe {result = Some(self.values().add(index).read())};
			
			self.len -= 1;
		}
		
		return result;
	}
	
	/// Removes a value at _index_ from the repository, returning it.
	/// # Time complexity
	/// _O(log<sub>64</sub> n)_ where _n_ is the number of values in the repository.
	#[inline]
	pub unsafe fn remove_unchecked(&mut self, index: usize) -> Type
	{
		let capacity = self.capacity();
		bit_indexing::erase(self.index_header_mut(), index, capacity);
		self.first_vacant_word = self.first_vacant_word.min(index / bit_indexing::IndexType::BITS as usize);
		self.len -= 1;
		return self.values().add(index).read();
	}
	
	/// Clears the repository, removing all values.
	pub fn clear(&mut self)
	{
		if self.capacity() != 0
		{
			if std::mem::needs_drop::<Type>()
			{
				self.simple_clear();
			}
			
			self.len = 0;
			self.first_vacant_word = 0;
			self.index_header_mut().fill(0);
		}
	}
	
	/// Returns the number of values in the repository.
	pub fn len(&self) -> usize {self.len}
	
	/// Returns `true` if the repository contains no values.
	pub fn is_empty(&self) -> bool {self.len == 0}
	
	/// Returns an iterator over the **indices** of values present in the repository.
	pub fn index_iter(&self) -> Indices<'_>
	{
		Indices
		{
			occupied: Occupied::new(self.index_header_leaf(), self.len),
			_data: std::marker::PhantomData,
		}
	}
	
	pub unsafe fn get_unchecked(&self, index: usize) -> &Type
	{
		self.as_slice().get_unchecked(index)
	}
	
	pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut Type
	{
		self.as_mut_slice().get_unchecked_mut(index)
	}
	
	#[inline]
	pub fn get(&self, index: usize) -> Option<&Type>
	{
		if index < self.capacity() && self.occupied(index)
		{
			return Some(unsafe {&*self.values().add(index)});
		}
		
		return None;
	}
	
	#[inline]
	pub fn get_mut(&mut self, index: usize) -> Option<&mut Type>
	{
		if index < self.capacity() && self.occupied(index)
		{
			return Some(unsafe {&mut *self.values().add(index)});
		}
		
		return None;
	}
	
	/// Returns whether there is a value at `index`, which must be lower than the capacity.
	#[inline]
	fn occupied(&self, index: usize) -> bool
	{
		let (word, mask) = bit_indexing::indices(index);
		// The level of the values ends the index, with a word for each `IndexType::BITS` slots of the capacity
		let leaf = self.index_length - bit_indexing::level_length(self.capacity());
		return unsafe {*self.index_header().as_ptr().add(leaf + word)} & mask != 0;
	}
	
	/// Returns an iterator over the values present in the repository, in the order of their indices.
	pub fn iter(&self) -> Iter<'_, Type>
	{
		self.into_iter()
	}
	
	/// Returns a mutable iterator over the values present in the repository, in the order of their indices.
	pub fn iter_mut(&mut self) -> IterMut<'_, Type>
	{
		self.into_iter()
	}
	
	fn simple_clear(&mut self)
	{
		for i in self.index_iter()
		{
			unsafe {self.values().add(i).drop_in_place()};
		}
	}
	
	/// Returns the array of the values, which starts the allocation.
	#[inline]
	fn values(&self) -> *mut Type
	{
		self.storage.data.as_ptr().cast::<Type>()
	}
	
	/// Returns the offset of the index in the allocation for `capacity` values, it follows them.
	/// The size of the allocation must not overflow.
	#[inline]
	fn index_offset(capacity: usize) -> usize
	{
		(std::mem::size_of::<Type>() * capacity).next_multiple_of(std::mem::align_of::<bit_indexing::IndexType>())
	}
	
	/// Returns the layout of the allocation for `capacity` values followed by the index and the length of the index.
	/// Panics if the size overflows.
	fn layout_for(capacity: usize) -> (std::alloc::Layout, usize)
	{
		let alignment = std::cmp::max(
			std::mem::align_of::<Type>(),
			std::mem::align_of::<bit_indexing::IndexType>(),
		);
		let index_length = crate::svst::bit_indexing::index_length(capacity);
		// The index has at most `capacity / 127 + 10` words, so its size does not overflow
		let byte_size = std::mem::size_of::<Type>().checked_mul(capacity)
			.and_then(|size| size.checked_next_multiple_of(std::mem::align_of::<bit_indexing::IndexType>()))
			.and_then(|offset| offset.checked_add(index_length * std::mem::size_of::<bit_indexing::IndexType>()))
			.expect("capacity overflow")
		;
		let layout = std::alloc::Layout::from_size_align(byte_size, alignment).expect("capacity overflow");
		
		return (layout, index_length);
	}
	
	#[inline]
	fn index_header(&self) -> &[bit_indexing::IndexType]
	{
		unsafe {std::slice::from_raw_parts(
			self.storage.data.as_ptr().add(Self::index_offset(self.capacity())).cast::<bit_indexing::IndexType>(), self.index_length
		)}
	}
	
	#[inline]
	fn index_header_mut(&mut self) -> &mut [bit_indexing::IndexType]
	{
		unsafe {std::slice::from_raw_parts_mut(
			self.storage.data.as_ptr().add(Self::index_offset(self.capacity())).cast::<bit_indexing::IndexType>(), self.index_length
		)}
	}
	
	#[inline]
	pub(super) fn index_header_leaf(&self) -> &[bit_indexing::IndexType]
	{
		&self.index_header()[self.index_length - bit_indexing::level_length(self.capacity()) .. self.index_length]
	}
	
	/// Moves the values to an allocation for `capacity` values, which must be greater than the current capacity.
	///
	/// The allocation is resized, which the allocator can do without copying, so the index follows the values.
	fn reallocate(&mut self, capacity: usize)
	{
		let (new_layout, index_length) = Self::layout_for(capacity);
		let old_capacity = self.capacity();
		let old_offset = Self::index_offset(old_capacity);
		let new_offset = Self::index_offset(capacity);
		let word_size = std::mem::size_of::<bit_indexing::IndexType>();
		
		// The index moves further, but its new place overlaps the old one if it moves less than its length,
		// which happens only for very small values or a very small growth, the old index is kept aside then
		let saved = if old_capacity != 0 && new_offset < old_offset + self.index_length * word_size
		{
			Some(self.index_header().to_vec())
		}
		else
		{
			None
		};
		
		let new_data = unsafe
		{
			if old_capacity == 0
			{
				std::alloc::alloc(new_layout)
			}
			else
			{
				std::alloc::realloc(self.storage.data.as_ptr(), Self::layout_for(old_capacity).0, new_layout.size())
			}
		};
		let new_data = std::ptr::NonNull::new(new_data).unwrap_or_else(|| std::alloc::handle_alloc_error(new_layout));
		
		unsafe
		{
			let new_index = new_data.as_ptr().add(new_offset).cast::<bit_indexing::IndexType>();
			new_index.write_bytes(0, index_length);
			
			if old_capacity != 0
			{
				let old_index = match &saved
				{
					Some(saved) => saved.as_slice(),
					None => std::slice::from_raw_parts(new_data.as_ptr().add(old_offset).cast::<bit_indexing::IndexType>(), self.index_length),
				};
				
				bit_indexing::copy(old_index, old_capacity, std::slice::from_raw_parts_mut(new_index, index_length), capacity);
			}
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
			if std::mem::needs_drop::<Type>()
			{
				self.simple_clear();
			}
			
			unsafe {std::alloc::dealloc(self.storage.data.as_ptr(), Self::layout_for(self.capacity()).0)};
		}
	}
}

impl<Type> Default for Repository<Type>
{
	fn default() -> Self {Self::new()}
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
	
	/// The values keep their indices.
	fn clone_from(&mut self, source: &Self)
	{
		self.clear();
		
		// The capacity has to exceed the highest occupied index
		let header = source.index_header_leaf();
		let end = match header.iter().rposition(|&word| word != 0)
		{
			Some(i) => (i + 1) * bit_indexing::IndexType::BITS as usize - header[i].leading_zeros() as usize,
			None => 0,
		};
		
		if self.capacity() < end
		{
			// There are no values to keep, so a new allocation avoids copying the old one
			*self = Self::new();
			self.reserve_exact(end);
		}
		
		let capacity = self.capacity();
		
		for index in source.index_iter()
		{
			let value = unsafe {source.get_unchecked(index)}.clone();
			
			// Each value is marked right after it is written, so that it is dropped if a later clone panics
			unsafe {self.values().add(index).write(value)};
			bit_indexing::insert(self.index_header_mut(), index, capacity);
			self.len += 1;
		}
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

/// A cursor over the occupied indices of a repository in ascending order,
/// which reads the level of the index holding the bits of the values.
#[derive(Clone)]
struct Occupied
{
	words: *const bit_indexing::IndexType,
	word_count: usize,
	/// The current word and its bits which were not visited yet.
	word: usize,
	bits: bit_indexing::IndexType,
	/// The number of the occupied indices which were not visited yet.
	remaining: usize,
}

impl Occupied
{
	/// `words` is the level of the values of the index, in which `remaining` bits are set.
	fn new(words: &[bit_indexing::IndexType], remaining: usize) -> Self
	{
		Self
		{
			words: words.as_ptr(),
			word_count: words.len(),
			word: 0,
			bits: words.first().copied().unwrap_or(0),
			remaining,
		}
	}
	
	/// Moves to the next word, returns `false` if there is none.
	#[inline]
	fn next_word(&mut self) -> bool
	{
		self.word += 1;
		
		if self.word >= self.word_count
		{
			return false;
		}
		
		self.bits = unsafe {*self.words.add(self.word)};
		return true;
	}
	
	#[inline]
	fn next(&mut self) -> Option<usize>
	{
		if self.remaining == 0
		{
			return None;
		}
		
		while self.bits == 0
		{
			if ! self.next_word()
			{
				// Only with an inconsistent number of set bits
				self.remaining = 0;
				return None;
			}
		}
		
		let bit = self.bits.trailing_zeros() as usize;
		// Clears the lowest set bit without shifting a mask
		self.bits &= self.bits - 1;
		self.remaining -= 1;
		return Some(self.word * bit_indexing::IndexType::BITS as usize + bit);
	}
	
	/// Visits the remaining indices as runs of consecutive ones, calling `function(accumulator, first, count)`
	/// with the whole word if it is full and with single indices otherwise.
	#[inline]
	fn fold<Acc>(mut self, mut accumulator: Acc, mut function: impl FnMut(Acc, usize, usize) -> Acc) -> Acc
	{
		const BITS: usize = bit_indexing::IndexType::BITS as usize;
		
		while self.remaining != 0
		{
			if self.bits == bit_indexing::IndexType::MAX
			{
				accumulator = function(accumulator, self.word * BITS, BITS);
				self.remaining -= BITS;
			}
			else
			{
				while self.bits != 0
				{
					accumulator = function(accumulator, self.word * BITS + self.bits.trailing_zeros() as usize, 1);
					self.bits &= self.bits - 1;
					self.remaining -= 1;
				}
			}
			
			if ! self.next_word()
			{
				break;
			}
		}
		
		return accumulator;
	}
}

/// An iterator over the occupied indices of a repository in ascending order.
pub struct Indices<'t>
{
	occupied: Occupied,
	_data: std::marker::PhantomData<&'t bit_indexing::IndexType>,
}

// It only reads the words of the index
unsafe impl<'t> Send for Indices<'t> {}
unsafe impl<'t> Sync for Indices<'t> {}

impl<'t> Clone for Indices<'t>
{
	fn clone(&self) -> Self
	{
		Self {occupied: self.occupied.clone(), _data: std::marker::PhantomData}
	}
}

impl<'t> std::iter::Iterator for Indices<'t>
{
	type Item = usize;
	
	#[inline]
	fn next(&mut self) -> Option<Self::Item>
	{
		self.occupied.next()
	}
	
	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>)
	{
		(self.occupied.remaining, Some(self.occupied.remaining))
	}
}

impl<'t> std::iter::ExactSizeIterator for Indices<'t> {}
impl<'t> std::iter::FusedIterator for Indices<'t> {}

/// An iterator over the values of a repository in the order of their indices.
pub struct Iter<'t, Type>
{
	occupied: Occupied,
	values: *const Type,
	_data: std::marker::PhantomData<&'t Type>,
}

// Like a slice iterator, it only shares the values
unsafe impl<'t, Type: Sync> Send for Iter<'t, Type> {}
unsafe impl<'t, Type: Sync> Sync for Iter<'t, Type> {}

impl<'t, Type> Clone for Iter<'t, Type>
{
	fn clone(&self) -> Self
	{
		Self {occupied: self.occupied.clone(), values: self.values, _data: std::marker::PhantomData}
	}
}

impl<'t, Type> std::iter::Iterator for Iter<'t, Type>
{
	type Item = &'t Type;
	
	#[inline]
	fn next(&mut self) -> Option<Self::Item>
	{
		let index = self.occupied.next()?;
		// Sound, since the bit of the index is set
		return Some(unsafe {&*self.values.add(index)});
	}
	
	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>)
	{
		(self.occupied.remaining, Some(self.occupied.remaining))
	}
	
	fn count(self) -> usize
	{
		self.occupied.remaining
	}
	
	#[inline]
	fn fold<Acc, Function>(self, accumulator: Acc, mut function: Function) -> Acc
	where
		Function: FnMut(Acc, Self::Item) -> Acc,
	{
		let values = self.values;
		
		// The values of a full word are consecutive, so they are visited as a slice, which the compiler can vectorize
		self.occupied.fold(accumulator, |accumulator, first, count|
			unsafe {std::slice::from_raw_parts(values.add(first), count)}.iter().fold(accumulator, &mut function)
		)
	}
}

impl<'t, Type> std::iter::ExactSizeIterator for Iter<'t, Type> {}
impl<'t, Type> std::iter::FusedIterator for Iter<'t, Type> {}

impl<'t, Type> IntoIterator for &'t Repository<Type>
{
	type Item = &'t Type;
	type IntoIter = Iter<'t, Type>;
	
	fn into_iter(self) -> Self::IntoIter
	{
		Iter
		{
			occupied: Occupied::new(self.index_header_leaf(), self.len),
			values: self.values(),
			_data: std::marker::PhantomData,
		}
	}
}

/// A mutable iterator over the values of a repository in the order of their indices.
pub struct IterMut<'t, Type>
{
	occupied: Occupied,
	values: *mut Type,
	_data: std::marker::PhantomData<&'t mut Type>,
}

// Like a mutable slice iterator
unsafe impl<'t, Type: Send> Send for IterMut<'t, Type> {}
unsafe impl<'t, Type: Sync> Sync for IterMut<'t, Type> {}

impl<'t, Type> std::iter::Iterator for IterMut<'t, Type>
{
	type Item = &'t mut Type;
	
	#[inline]
	fn next(&mut self) -> Option<Self::Item>
	{
		let index = self.occupied.next()?;
		// Sound, since the bit of the index is set and each index is visited once
		return Some(unsafe {&mut *self.values.add(index)});
	}
	
	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>)
	{
		(self.occupied.remaining, Some(self.occupied.remaining))
	}
	
	fn count(self) -> usize
	{
		self.occupied.remaining
	}
	
	#[inline]
	fn fold<Acc, Function>(self, accumulator: Acc, mut function: Function) -> Acc
	where
		Function: FnMut(Acc, Self::Item) -> Acc,
	{
		let values = self.values;
		
		self.occupied.fold(accumulator, |accumulator, first, count|
			unsafe {std::slice::from_raw_parts_mut(values.add(first), count)}.iter_mut().fold(accumulator, &mut function)
		)
	}
}

impl<'t, Type> std::iter::ExactSizeIterator for IterMut<'t, Type> {}
impl<'t, Type> std::iter::FusedIterator for IterMut<'t, Type> {}

impl<'t, Type> IntoIterator for &'t mut Repository<Type>
{
	type Item = &'t mut Type;
	type IntoIter = IterMut<'t, Type>;
	
	fn into_iter(self) -> Self::IntoIter
	{
		IterMut
		{
			occupied: Occupied::new(self.index_header_leaf(), self.len),
			values: self.values(),
			_data: std::marker::PhantomData,
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
	
	#[inline]
	fn index(&self, index: usize) -> &Self::Output
	{
		match self.get(index)
		{
			Some(value) => value,
			None => panic!("index {} contains an invalid value", index),
		}
	}
}

impl<Type> std::ops::IndexMut<usize> for Repository<Type>
{
	#[inline]
	fn index_mut(&mut self, index: usize) -> &mut Self::Output
	{
		match self.get_mut(index)
		{
			Some(value) => value,
			None => panic!("index {} contains an invalid value", index),
		}
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
	
	idxs.shuffle(&mut rand::rng());
	
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
	
	idxs.shuffle(&mut rand::rng());
	
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
fn test_insert_lowest_vacant()
{
	// Every insertion takes the lowest vacant index, with three levels of the index
	use rand::{RngExt, SeedableRng};
	let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
	let mut r = Repository::new();
	let mut end = 0;
	let mut vacant = std::collections::BTreeSet::new();
	
	let insert = |r: &mut Repository<u8>, end: &mut usize, vacant: &mut std::collections::BTreeSet<usize>|
	{
		let expected = vacant.pop_first().unwrap_or_else(|| {*end += 1; *end - 1});
		assert_eq!(expected, r.insert(0));
	};
	
	for _ in 0 .. 20_000
	{
		insert(&mut r, &mut end, &mut vacant);
	}
	
	for _ in 0 .. 40_000
	{
		if rng.random_bool(0.5)
		{
			insert(&mut r, &mut end, &mut vacant);
		}
		else
		{
			let index = rng.random_range(0 .. end);
			assert_eq!(! vacant.contains(&index), r.remove(index).is_some());
			vacant.insert(index);
		}
	}
	
	assert_eq!(end - vacant.len(), r.len());
	r.clear();
	assert_eq!(0, r.insert(0));
}

#[test]
fn test_iterators()
{
	// Full, sparse and empty words of the index
	let mut r = Repository::new();
	
	for i in 0 .. 1000
	{
		r.insert(i);
	}
	
	for i in (256 .. 512).chain((600 .. 1000).filter(|i| i % 3 != 0))
	{
		r.remove(i);
	}
	
	let expected: Vec<usize> = (0 .. 256).chain(512 .. 600).chain((600 .. 1000).filter(|i| i % 3 == 0)).collect();
	assert!(r.iter().copied().eq(expected.iter().copied()));
	assert_eq!(expected.iter().sum::<usize>(), r.iter().sum::<usize>());
	assert_eq!(expected.len(), r.iter().count());
	assert_eq!(expected.len(), r.iter().len());
	assert!(r.index_iter().eq(expected.iter().copied()));
	
	// Folding after visiting a part of a word
	for skip in [0, 1, 127, 128, 129, 300, expected.len() - 1, expected.len()]
	{
		let mut it = r.iter();
		let visited: Vec<usize> = it.by_ref().take(skip).copied().collect();
		assert_eq!(expected.len() - skip, it.len());
		let rest = it.fold(Vec::new(), |mut values, value| {values.push(*value); values});
		assert_eq!(expected, [visited, rest].concat());
	}
	
	r.iter_mut().for_each(|value| *value *= 2);
	
	for value in &mut r
	{
		*value += 1;
	}
	
	assert!(r.iter().copied().eq(expected.iter().map(|value| 2 * value + 1)));
	assert_eq!(None, Repository::<u8>::new().iter().next());
	assert_eq!(0, Repository::<u8>::new().iter_mut().fold(0, |count, _| count + 1));
}

#[test]
fn test_growth()
{
	// Insertions grow the capacity by the default growth
	let mut r = Repository::new();
	let mut capacities = Vec::new();
	
	for i in 0 .. 1000
	{
		r.insert(i);
		
		if capacities.last() != Some(&r.capacity())
		{
			capacities.push(r.capacity());
		}
	}
	
	let mut expected = vec![VectorStorage::default_capacity_growth(0)];
	
	while *expected.last().unwrap() < 1000
	{
		expected.push(VectorStorage::default_capacity_growth(*expected.last().unwrap()));
	}
	
	assert_eq!(expected, capacities);
	
	// Reservation counts the vacant slots, values stay at their indices
	for i in 0 .. 990
	{
		r.remove(i);
	}
	
	let capacity = r.capacity();
	r.reserve(capacity - 10);
	assert_eq!(capacity, r.capacity());
	r.reserve(capacity - 9);
	assert_eq!(VectorStorage::default_capacity_growth(capacity), r.capacity());
	r.reserve_exact(r.capacity() - 5);
	assert_eq!(VectorStorage::default_capacity_growth(capacity) + 5, r.capacity());
	assert!((990 .. 1000).all(|i| r[i] == i));
	assert_eq!(10, r.len());
}

#[test]
#[should_panic(expected = "capacity overflow")]
fn test_capacity_overflow_with_capacity()
{
	Repository::<u64>::with_capacity(usize::MAX);
}

#[test]
#[should_panic(expected = "capacity overflow")]
fn test_capacity_overflow_byte_size()
{
	// The byte size of the values wraps around to 0 without the check
	Repository::<[u8; 1 << 20]>::new().reserve_exact(1 << 44);
}

#[test]
#[should_panic(expected = "capacity overflow")]
fn test_capacity_overflow_additional()
{
	let mut r = Repository::new();
	r.insert(0);
	r.reserve(usize::MAX);
}

#[test]
fn test_clone()
{
	use std::rc::Rc;
	
	// Vacant slots throughout, the highest index is the last one of a word
	let mut source = Repository::new();
	
	for i in 0 .. 300
	{
		source.insert(Rc::new(i));
	}
	
	for i in (0 .. 300).filter(|&i| i % 3 == 0 && i != 255 || i > 255)
	{
		source.remove(i);
	}
	
	let check = |clone: &Repository<Rc<i32>>|
	{
		assert_eq!(source.len(), clone.len());
		assert!(source.index_iter().eq(clone.index_iter()));
		assert!(source.index_iter().all(|i| Rc::ptr_eq(&source[i], &clone[i])));
	};
	
	let clone = source.clone();
	check(&clone);
	assert_eq!(256, clone.capacity());
	
	// Into repositories holding values, with more and less capacity than needed
	let mut larger = Repository::with_capacity(1000);
	let mut smaller = Repository::new();
	
	for _ in 0 .. 500
	{
		larger.insert(Rc::new(-1));
	}
	
	smaller.insert(Rc::new(-1));
	larger.clone_from(&source);
	smaller.clone_from(&source);
	check(&larger);
	check(&smaller);
	assert!(larger.capacity() >= 1000);
	
	// Insertion continues at the lowest vacant index
	let mut clone = clone;
	assert_eq!(0, clone.insert(Rc::new(0)));
	
	drop((clone, larger, smaller));
	assert!(source.iter().all(|value| Rc::strong_count(value) == 1));
	assert_eq!(0, Repository::<i32>::new().clone().len());
}

#[test]
fn test_clone_panic()
{
	// A panicking clone leaves the values cloned before it, which are dropped later
	struct Bomb(std::rc::Rc<()>, bool);
	
	impl Clone for Bomb
	{
		fn clone(&self) -> Self
		{
			assert!(! self.1, "bomb");
			Bomb(self.0.clone(), false)
		}
	}
	
	let counter = std::rc::Rc::new(());
	let mut source = Repository::new();
	
	for i in 0 .. 10
	{
		source.insert(Bomb(counter.clone(), i == 6));
	}
	
	let mut target = Repository::new();
	assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| target.clone_from(&source))).is_err());
	assert_eq!(6, target.len());
	assert_eq!(1 + 10 + 6, std::rc::Rc::strong_count(&counter));
	drop(target);
	assert_eq!(1 + 10, std::rc::Rc::strong_count(&counter));
}

#[test]
fn test_reallocation_overlapping_index()
{
	// The new place of the index overlaps the old one for zero-sized values
	let mut zero_sized = Repository::new();
	
	for _ in 0 .. 1000
	{
		zero_sized.insert(());
	}
	
	for i in (0 .. 1000).step_by(3)
	{
		zero_sized.remove(i);
	}
	
	zero_sized.reserve_exact(zero_sized.capacity() - zero_sized.len() + 1);
	assert!(zero_sized.index_iter().eq((0 .. 1000).filter(|i| i % 3 != 0)));
	
	// And for growth by one, across the addition of an index level
	let mut bytes = Repository::new();
	
	for i in 0 .. 300
	{
		bytes.reserve_exact(bytes.capacity() - bytes.len() + 1);
		assert_eq!(i, bytes.insert(i as u8));
	}
	
	assert_eq!(300, bytes.capacity());
	assert!((0 .. 300).all(|i| bytes[i] == i as u8));
}

#[test]
fn test_empty_type()
{
	let mut r = Repository::<()>::new();
	r.insert(());
	assert_eq!(1, r.len());
	assert_eq!((), r[0]);
}
