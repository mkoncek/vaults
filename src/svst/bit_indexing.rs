//! Hierarchical bitset index used by [`Repository`](super::Repository) to track occupied slots.
//!
//! The index is a tree of 64-bit words.  Each bit in a non-leaf word summarizes whether the
//! corresponding child word is fully occupied.  This lets `push_front` (find-first-free) and
//! `erase` run in *O*(depth) steps, where the depth is at most `ceil(log_{64}(capacity))`.

pub type IndexType = u64;

#[inline]
pub(super) const fn indices(index: usize) -> (usize, IndexType)
{
	(
		index / IndexType::BITS as usize,
		1 << index % IndexType::BITS as usize,
	)
}

#[derive(Clone, Copy, Debug)]
pub struct IndexIterator
{
	bitset: IndexType,
}

impl IndexIterator
{
	#[inline]
	pub fn new(bitset: IndexType) -> Self
	{
		Self {bitset}
	}
}

impl Iterator for IndexIterator
{
	type Item = usize;
	
	#[inline]
	fn next(&mut self) -> Option<Self::Item>
	{
		if self.bitset == 0
		{
			return None;
		}
		
		let result: Self::Item = self.bitset.trailing_zeros() as usize;
		// Clears the lowest set bit without shifting a mask
		self.bitset &= self.bitset - 1;
		
		return Some(result);
	}
}

#[test]
fn test_index_iterator()
{
	assert_eq!(0, IndexIterator::new(0).count());
	
	let it = IndexIterator::new(0b11010110);
	assert_eq!([1, 2, 4, 6, 7], *it.collect::<Vec<_>>().as_slice());
}

#[derive(Clone, Copy, Debug)]
pub struct TransientIndexSliceIterator
{
	it: IndexIterator,
	pos: usize,
}

impl TransientIndexSliceIterator
{
	#[inline]
	pub fn new(bitset: &[IndexType]) -> Self
	{
		Self
		{
			it: IndexIterator::new(if bitset.len() == 0 {0} else {bitset[0]}),
			pos: 0,
		}
	}
	
	#[inline]
	pub fn next(&mut self, bitset: &[IndexType]) -> Option<usize>
	{
		if self.pos == bitset.len()
		{
			return None;
		}
		
		if let Some(pos) = self.it.next()
		{
			return Some(self.pos * IndexType::BITS as usize + pos);
		}
		
		loop
		{
			self.pos += 1;
			
			if self.pos == bitset.len()
			{
				break;
			}
			
			let value = bitset[self.pos];
			
			if value != 0
			{
				self.it = IndexIterator::new(bitset[self.pos]);
				break;
			}
		}
		
		if let Some(pos) = self.it.next()
		{
			return Some(self.pos * IndexType::BITS as usize + pos);
		}
		
		return None;
	}
}

#[test]
fn test_index_slice_iterator()
{
	fn indices(bitset: &[IndexType]) -> Vec<usize>
	{
		let mut it = TransientIndexSliceIterator::new(bitset);
		return std::iter::from_fn(|| it.next(bitset)).collect();
	}
	
	assert_eq!(0, indices(&[]).len());
	assert_eq!(0, indices(&[0]).len());
	assert_eq!(0, indices(&[0, 0, 0, 0]).len());
	
	{
		let bits = IndexType::BITS as usize;
		assert_eq!(vec![1, 2, 4, 6, 7, bits + 1, bits + 2, bits + 4, bits + 6, bits + 7], indices(&[0b11010110, 0b11010110]));
	}
	
	assert_eq!(1, indices(&[1]).len());
	assert_eq!(1, indices(&[0, 1]).len());
	assert_eq!(1, indices(&[1, 0]).len());
	assert_eq!(2, indices(&[1, 1]).len());
}

#[inline]
pub const fn level_length(size: usize) -> usize
{
	size.div_ceil(IndexType::BITS as usize)
}

#[test]
fn test_level_length()
{
	assert_eq!(0, level_length(0));
	assert_eq!(1, level_length(1));
	assert_eq!(1, level_length(2));
	assert_eq!(1, level_length(IndexType::BITS as usize - 1));
	assert_eq!(1, level_length(IndexType::BITS as usize + 0));
	assert_eq!(2, level_length(IndexType::BITS as usize + 1));
	assert_eq!(2, level_length(IndexType::BITS as usize + 2));
}

pub fn index_length(mut size: usize) -> usize
{
	let mut result: usize = 0;
	
	loop
	{
		size = level_length(size);
		result += size;
		
		if size <= 1
		{
			break;
		}
	}
	
	return result;
}

#[test]
fn test_index_length()
{
	assert_eq!(0, index_length(0));
	assert_eq!(1, index_length(1));
	assert_eq!(1, index_length(2));
	assert_eq!(1, index_length(IndexType::BITS as usize - 1));
	assert_eq!(1, index_length(IndexType::BITS as usize + 0));
	assert_eq!(3, index_length(IndexType::BITS as usize + 1));
	assert_eq!(3, index_length(IndexType::BITS as usize + 2));
}

#[allow(dead_code)] // Kept for future use
pub fn contains(mut index_span: &[IndexType], mut position: usize, mut size: usize) -> bool
{
	size = level_length(size);
	index_span = &index_span[index_span.len() - size ..];
	let modulus = position % IndexType::BITS as usize;
	position /= IndexType::BITS as usize;
	let result = index_span[position] & (1 << modulus);
	return result != 0;
}

#[test]
fn test_contains()
{
	{
		let arr = [0 as IndexType];
		
		for i in 0 .. IndexType::BITS as usize
		{
			assert_eq!(false, contains(&arr, i, IndexType::BITS as usize));
		}
	}
	
	{
		let arr = [0b11010110 as IndexType];
		assert_eq!(false, contains(&arr, 0, IndexType::BITS as usize));
		assert_eq!(true, contains(&arr, 1, IndexType::BITS as usize));
		assert_eq!(true, contains(&arr, 2, IndexType::BITS as usize));
		assert_eq!(false, contains(&arr, 3, IndexType::BITS as usize));
		assert_eq!(true, contains(&arr, 4, IndexType::BITS as usize));
		assert_eq!(false, contains(&arr, 5, IndexType::BITS as usize));
		assert_eq!(true, contains(&arr, 6, IndexType::BITS as usize));
		assert_eq!(true, contains(&arr, 7, IndexType::BITS as usize));
	}
	
	{
		let arr = [0, 0, 0b11010110 as IndexType];
		
		for i in 0 .. IndexType::BITS as usize
		{
			assert_eq!(false, contains(&arr, i, 2 * IndexType::BITS as usize));
		}
		
		assert_eq!(false, contains(&arr, IndexType::BITS as usize + 0, 2 * IndexType::BITS as usize));
		assert_eq!(true, contains(&arr, IndexType::BITS as usize + 1, 2 * IndexType::BITS as usize));
		assert_eq!(true, contains(&arr, IndexType::BITS as usize + 2, 2 * IndexType::BITS as usize));
		assert_eq!(false, contains(&arr, IndexType::BITS as usize + 3, 2 * IndexType::BITS as usize));
		assert_eq!(true, contains(&arr, IndexType::BITS as usize + 4, 2 * IndexType::BITS as usize));
		assert_eq!(false, contains(&arr, IndexType::BITS as usize + 5, 2 * IndexType::BITS as usize));
		assert_eq!(true, contains(&arr, IndexType::BITS as usize + 6, 2 * IndexType::BITS as usize));
		assert_eq!(true, contains(&arr, IndexType::BITS as usize + 7, 2 * IndexType::BITS as usize));
	}
}

#[inline]
pub fn erase(mut index_span: &mut [IndexType], mut position: usize, mut size: usize) -> bool
{
	let mut result = false;
	
	loop
	{
		size = level_length(size);
		let modulus = position % IndexType::BITS as usize;
		position /= IndexType::BITS as usize;
		let level_begin = index_span.len() - size;
		
		if index_span[level_begin + position] & (1 << modulus) == 0
		{
			break;
		}
		
		result = true;
		
		index_span[level_begin + position] &= ! (1 << modulus);
		index_span = &mut index_span[.. level_begin];
		
		if size <= 1
		{
			break;
		}
	}
	
	return result;
}

#[test]
fn test_erase()
{
	{
		let mut arr = [0b11010110 as IndexType];
		assert_eq!(false, erase(&mut arr, 0, IndexType::BITS as usize));
		assert_eq!([0b11010110 as IndexType], arr.as_slice());
		assert_eq!(true, erase(&mut arr, 1, IndexType::BITS as usize));
		assert_eq!([0b11010100 as IndexType], arr.as_slice());
		assert_eq!(true, erase(&mut arr, 6, IndexType::BITS as usize));
		assert_eq!([0b10010100 as IndexType], arr.as_slice());
		assert_eq!(false, erase(&mut arr, 6, IndexType::BITS as usize));
		assert_eq!([0b10010100 as IndexType], arr.as_slice());
	}
	
	{
		let mut arr = [0, 0, 0b11010110 as IndexType];
		
		for i in 0 .. IndexType::BITS as usize
		{
			assert_eq!(false, erase(&mut arr, i, 2 * IndexType::BITS as usize));
		}
		
		assert_eq!(false, erase(&mut arr, IndexType::BITS as usize + 0, 2 * IndexType::BITS as usize));
		assert_eq!([0, 0, 0b11010110 as IndexType], arr.as_slice());
		assert_eq!(true, erase(&mut arr, IndexType::BITS as usize + 1, 2 * IndexType::BITS as usize));
		assert_eq!([0, 0, 0b11010100 as IndexType], arr.as_slice());
		assert_eq!(true, erase(&mut arr, IndexType::BITS as usize + 6, 2 * IndexType::BITS as usize));
		assert_eq!([0, 0, 0b10010100 as IndexType], arr.as_slice());
		assert_eq!(false, erase(&mut arr, IndexType::BITS as usize + 6, 2 * IndexType::BITS as usize));
		assert_eq!([0, 0, 0b10010100 as IndexType], arr.as_slice());
	}
	
	{
		let mut arr = [0b10 as IndexType, 0, IndexType::MAX];
		
		assert_eq!(true, erase(&mut arr, IndexType::BITS as usize + 5, 2 * IndexType::BITS as usize));
		assert_eq!([0, 0, IndexType::MAX & ! (1 << 5)], arr.as_slice());
	}
}

/// The number of words of the level `level` of the index, where level 0 holds the bits of the values,
/// i.e. `level_length` applied `level + 1` times to `size`.
#[inline]
const fn level_length_at(size: usize, level: u32) -> usize
{
	let shift = IndexType::BITS.trailing_zeros() * (level + 1);
	
	if shift >= usize::BITS
	{
		return (size != 0) as usize;
	}
	
	return (size >> shift) + (size & ((1 << shift) - 1) != 0) as usize;
}

/// The number of levels of the index below its single top word.
#[inline]
const fn depth(size: usize) -> u32
{
	let bits = usize::BITS - size.saturating_sub(1).leading_zeros();
	let bits_per_level = IndexType::BITS.trailing_zeros();
	return (bits + bits_per_level - 1) / bits_per_level - (bits != 0) as u32;
}

#[test]
fn test_level_length_at()
{
	for size in [0, 1, 2, 127, 128, 129, 128 * 128, 128 * 128 + 1, 795_566, 1_988_923, u32::MAX as usize, 1 << 43, (1 << 49) + 1, usize::MAX - IndexType::BITS as usize + 1]
	{
		let mut length = level_length(size);
		let mut level = 0;
		
		while length > 1
		{
			assert_eq!(length, level_length_at(size, level), "size {}, level {}", size, level);
			length = level_length(length);
			level += 1;
		}
		
		assert_eq!(length, level_length_at(size, level), "size {}, level {}", size, level);
		assert_eq!(level, depth(size), "size {}", size);
		
		if size > 1
		{
			assert_eq!(index_length(size), (0 ..= level).map(|level| level_length_at(size, level)).sum::<usize>(), "size {}", size);
		}
	}
}

/// Marks the lowest index which is not present, returns that index.
/// There must be such index lower than `size`.
#[inline]
pub fn push_front(index_span: &mut [IndexType], size: usize) -> usize
{
	let depth = depth(size);
	
	// The offset of the current level and the position of the current word within it, starting at the top word
	let mut offset = 0;
	let mut position = 0;
	
	for level in (0 .. depth).rev()
	{
		position = position * IndexType::BITS as usize + index_span[offset + position].trailing_ones() as usize;
		offset += level_length_at(size, level + 1);
	}
	
	return position * IndexType::BITS as usize + mark_lowest(index_span, size, depth, offset, position);
}

/// Like [`push_front`], but looks at the word `hint` of the level of the values first, before which all words must be full,
/// and moves `hint` to the word of the marked index if that word was full.
#[inline]
pub fn push_front_from(index_span: &mut [IndexType], size: usize, hint: &mut usize) -> usize
{
	let offset = index_span.len() - level_length_at(size, 0);
	
	// The lowest clear bit of the word is the lowest one of all, since the words before it are full
	if index_span[offset + *hint] != IndexType::MAX
	{
		return *hint * IndexType::BITS as usize + mark_lowest(index_span, size, depth(size), offset, *hint);
	}
	
	let result = push_front(index_span, size);
	*hint = result / IndexType::BITS as usize;
	return result;
}

/// Marks the lowest clear bit of the word at `position` in the level of the values, which starts at `offset`,
/// and the bits of the words in the levels above which become full, returns the position of the marked bit.
/// The words before the marked ones in each level must be full.
#[inline]
fn mark_lowest(index_span: &mut [IndexType], size: usize, depth: u32, mut offset: usize, mut position: usize) -> usize
{
	let word = &mut index_span[offset + position];
	let result = word.trailing_ones() as usize;
	// Sets the lowest clear bit without shifting a mask
	*word |= *word + 1;
	
	// Mark the full words in the levels above
	for level in 0 .. depth
	{
		if ! index_span[offset + position] != 0
		{
			break;
		}
		
		let modulus = position % IndexType::BITS as usize;
		position /= IndexType::BITS as usize;
		offset -= level_length_at(size, level + 1);
		let parent = &mut index_span[offset + position];
		// The bit of the full word is the lowest clear one, since the words before it are full
		debug_assert_eq!(modulus, parent.trailing_ones() as usize);
		*parent |= *parent + 1;
	}
	
	return result;
}

#[test]
fn test_push_front()
{
	{
		let mut arr = [0b1111 as IndexType];
		assert_eq!(4, push_front(&mut arr, IndexType::BITS as usize));
		assert_eq!([0b11111 as IndexType], arr);
	}
	
	{
		let mut arr = [1, IndexType::MAX, 0b1111 as IndexType];
		assert_eq!(IndexType::BITS as usize + 4, push_front(&mut arr, 2 * IndexType::BITS as usize));
		assert_eq!([1, IndexType::MAX, 0b11111 as IndexType], arr);
	}
	
	{
		let capacity = 100_000;
		let len = index_length(capacity);
		let last_level_begin = len - level_length(capacity);
		let mut arr = Vec::<IndexType>::new();
		arr.resize(len, 0);
		
		for i in 0 .. capacity
		{
			assert_eq!(i, push_front(&mut arr, capacity));
			assert_ne!(0, arr[last_level_begin + i / IndexType::BITS as usize] & 1 << i % IndexType::BITS as usize);
		}
		
		assert!(erase(&mut arr, 40_000, capacity));
		assert!(!erase(&mut arr, 40_000, capacity));
		assert_eq!(40_000, push_front(&mut arr, capacity));
		
		assert!(erase(&mut arr, 40_000, capacity));
		assert!(erase(&mut arr, 30_000, capacity));
		assert!(erase(&mut arr, 20_000, capacity));
		assert!(erase(&mut arr, 19_999, capacity));
		assert!(erase(&mut arr, 19_998, capacity));
		
		assert_eq!(19_998, push_front(&mut arr, capacity));
		assert_eq!(19_999, push_front(&mut arr, capacity));
		assert_eq!(20_000, push_front(&mut arr, capacity));
		assert_eq!(30_000, push_front(&mut arr, capacity));
		assert_eq!(40_000, push_front(&mut arr, capacity));
	}
}

/// Marks `position`, which must not be present and must be lower than `size`.
#[inline]
pub fn insert(index_span: &mut [IndexType], mut position: usize, size: usize)
{
	// The offset of the current level, starting at the level of the values
	let mut offset = index_span.len() - level_length_at(size, 0);
	let modulus = position % IndexType::BITS as usize;
	position /= IndexType::BITS as usize;
	index_span[offset + position] |= 1 << modulus;
	
	// Mark the full words in the levels above
	for level in 0 .. depth(size)
	{
		if ! index_span[offset + position] != 0
		{
			break;
		}
		
		let modulus = position % IndexType::BITS as usize;
		position /= IndexType::BITS as usize;
		offset -= level_length_at(size, level + 1);
		index_span[offset + position] |= 1 << modulus;
	}
}

#[test]
fn test_insert()
{
	// Three levels with partial last words
	let capacity = 128 * 128 + 5;
	let length = index_length(capacity);
	
	{
		let mut arr = vec![0; length];
		insert(&mut arr, 2, capacity);
		insert(&mut arr, 3, capacity);
		assert_eq!([0, 1, 4, 5], [0; 4].map(|_| push_front(&mut arr, capacity)));
	}
	
	{
		// Marking all positions in any order gives the same index as pushing them in order
		let mut pushed = vec![0; length];
		let mut inserted = vec![0; length];
		
		for i in 0 .. capacity
		{
			push_front(&mut pushed, capacity);
			// A permutation, since 7919 is a prime not dividing the capacity
			insert(&mut inserted, i * 7919 % capacity, capacity);
		}
		
		assert_eq!(pushed, inserted);
	}
	
	{
		// A full word marks its bit in the level above, so pushing skips it
		let mut arr = vec![0; length];
		
		for i in (0 .. 2 * 128).rev()
		{
			insert(&mut arr, i, capacity);
		}
		
		assert_eq!(2 * 128, push_front(&mut arr, capacity));
	}
}

pub fn copy(source_span: &[IndexType], mut source_size: usize, target_span: &mut [IndexType], mut target_size: usize)
{
	assert!(source_span.len() <= target_span.len());
	
	let mut source_end = source_span.len();
	let mut target_end = target_span.len();
	
	loop
	{
		source_size = level_length(source_size);
		target_size = level_length(target_size);
		
		let source_begin = source_end - source_size;
		let target_begin = target_end - target_size;
		
		target_span[target_begin ..][.. source_size].copy_from_slice(&source_span[source_begin .. source_end]);
		
		source_end = source_begin;
		target_end = target_begin;
		
		if source_size <= 1
		{
			break;
		}
	}
	
	if ! source_span[source_end] == 0 && target_size > 1
	{
		target_size = level_length(target_size);
		target_span[target_end - target_size] = 1;
	}
}

#[test]
fn test_copy()
{
	{
		let value = 0b1010110101010;
		let mut result = [0 as IndexType];
		copy(&[value], IndexType::BITS as usize, &mut result, IndexType::BITS as usize);
		assert_eq!([value], result);
	}
	
	{
		let mut result = [0 as IndexType; 3];
		copy(&[IndexType::MAX as IndexType], IndexType::BITS as usize, &mut result, 2 * IndexType::BITS as usize);
		assert_eq!([1, IndexType::MAX, 0], result);
	}
}
