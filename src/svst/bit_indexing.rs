pub type IndexType = u128;

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
	pub fn new(bitset: IndexType) -> Self
	{
		Self {bitset}
	}
}

impl Iterator for IndexIterator
{
	type Item = usize;
	
	fn next(&mut self) -> Option<Self::Item>
	{
		if self.bitset == 0
		{
			return None;
		}
		
		let result: Self::Item = self.bitset.trailing_zeros() as usize;
		self.bitset &= ! (1 << result);
		
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
	pub fn new(bitset: &[IndexType]) -> Self
	{
		Self
		{
			it: IndexIterator::new(if bitset.len() == 0 {0} else {bitset[0]}),
			pos: 0,
		}
	}
	
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

pub struct IndexSliceIterator<'t>
{
	bitset: &'t [IndexType],
	it: TransientIndexSliceIterator,
}

impl<'t> IndexSliceIterator<'t>
{
	pub fn new(bitset: &'t [IndexType]) -> Self
	{
		Self
		{
			bitset,
			it: TransientIndexSliceIterator::new(bitset),
		}
	}
}

impl<'t> Iterator for IndexSliceIterator<'t>
{
	type Item = usize;
	
	fn next(&mut self) -> Option<Self::Item>
	{
		self.it.next(self.bitset)
	}
}

#[test]
fn test_index_slice_iterator()
{
	assert_eq!(0, IndexSliceIterator::new(&[]).count());
	assert_eq!(0, IndexSliceIterator::new(&[0]).count());
	assert_eq!(0, IndexSliceIterator::new(&[0, 0, 0, 0]).count());
	
	{
		let range = [0b11010110, 0b11010110];
		let it = IndexSliceIterator::new(&range);
		let bits = IndexType::BITS as usize;
		assert_eq!([1, 2, 4, 6, 7, bits + 1, bits + 2, bits + 4, bits + 6, bits + 7],
			*it.collect::<Vec<_>>().as_slice()
		);
	}
	
	assert_eq!(1, IndexSliceIterator::new(&[1]).count());
	assert_eq!(1, IndexSliceIterator::new(&[0, 1]).count());
	assert_eq!(1, IndexSliceIterator::new(&[1, 0]).count());
	assert_eq!(2, IndexSliceIterator::new(&[1, 1]).count());
}

pub const fn level_length(mut size: usize) -> usize
{
	size += IndexType::BITS as usize - 1;
	size /= IndexType::BITS as usize;
	return size;
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
	
	let result = position * IndexType::BITS as usize + index_span[offset + position].trailing_ones() as usize;
	
	index_span[offset + position] |= 1 << (result % IndexType::BITS as usize);
	
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
		index_span[offset + position] |= 1 << modulus;
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
