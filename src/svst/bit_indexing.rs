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
pub(super) struct TransientIndexSliceIterator
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
	
	pub(super) fn next(&mut self, bitset: &[IndexType]) -> Option<usize>
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

/*
const fn max_levels() -> usize
{
	let mut result = 1;
	let mut level = usize::MAX - IndexType::BITS as usize;
	while level != 1
	{
		level = level_length(level);
		result += 1;
	}
	return result;
}
*/

#[derive(Debug, Clone, Copy)]
struct Levels
{
	bitfield: u128,
	pub len: u8,
}

impl From<usize> for Levels
{
	fn from(value: usize) -> Self
	{
		let mut result = Self {bitfield: 0, len: 0};
		let mut size = level_length(value);
		let mut shift: u8 = 64;
		
		while size != 1
		{
			result.len += 1;
			size = level_length(size);
			result.bitfield |= (size as u128) << shift;
			
			shift /= 2;
		}
		
		assert_ne!(0, shift);
		
		shift *= 2;
		
		while shift <= 64
		{
			let mut addition = (result.bitfield >> (shift / 2)) & ((shift / 2) as u128 - 1);
			addition <<= shift;
			result.bitfield += addition;
			shift *= 2;
		}
		
		return result;
	}
}

impl Levels
{
	pub fn at(&self, index: u8) -> usize
	{
		assert!(index < self.len.max(1), "Levels::at: index is {}, but self.len is {}", index, self.len);
		let shift = 64 / (1 << index);
		return ((self.bitfield >> shift) & (shift as u128 - 1)) as usize;
	}
}

#[test]
fn test_levels()
{
	/*
	println!("{:b}", Levels::from(0).bitfield);
	println!("{:b}", Levels::from(1).bitfield);
	println!("{:b}", Levels::from(2).bitfield);
	println!("{:b}", Levels::from(3).bitfield);
	println!("{:b}", Levels::from(4).bitfield);
	println!("{:b}", Levels::from(127).bitfield);
	println!("{:b}", Levels::from(128).bitfield);
	println!("{:b}", Levels::from(129).bitfield);
	
	
	assert_eq!(0, Levels::from(0).bitfield);
	assert_eq!(IndexType::BITS as usize + 0, Levels::from(1).bitfield);
	assert_eq!(IndexType::BITS as usize + 1, Levels::from(2).bitfield);
	assert_eq!(IndexType::BITS as usize + IndexType::BITS as usize, Levels::from(IndexType::BITS as usize).bitfield);
	*/
}

pub fn _push_front(index_span: &mut [IndexType], size: usize) -> usize
{
	let mut sizes = [0_usize; 6];
	let mut sizes_len: usize = 0;
	
	{
		let mut size = level_length(size);
		
		for i in 0 .. sizes.len()
		{
			if size == 1
			{
				break;
			}
			
			sizes_len += 1;
			size = level_length(size);
			sizes[i] = size;
		}
		
		for i in (0 .. sizes_len.saturating_sub(1)).rev()
		{
			sizes[i] += sizes[i + 1];
		}
	}
	
	let mut position = 0;
	
	if sizes_len > 0
	{
		position = index_span[0].trailing_ones() as usize;
		
		for i in (1 .. sizes_len).rev()
		{
			position = position * IndexType::BITS as usize + index_span[sizes[i] + position].trailing_ones() as usize;
		}
	}
	
	let result = position * IndexType::BITS as usize + index_span[sizes[0] + position].trailing_ones() as usize;
	
	index_span[sizes[0] + position] |= 1 << (result % IndexType::BITS as usize);
	
	for i in 0 .. sizes.len()
	{
		if ! index_span[sizes[i] + position] != 0
		{
			break;
		}
		
		let modulus = position % IndexType::BITS as usize;
		position /= IndexType::BITS as usize;
		let offset = if i + 1 == sizes.len()
		{
			0
		}
		else
		{
			sizes[i + 1]
		};
		
		index_span[offset + position] |= 1 << modulus;
	}
	
	return result;
}

pub fn push_front(index_span: &mut [IndexType], size: usize) -> usize
{
	let levels = Levels::from(size);
	let mut position = 0;
	
	if levels.len > 0
	{
		position = index_span[0].trailing_ones() as usize;
		
		for i in (1 .. levels.len).rev()
		{
			position = position * IndexType::BITS as usize + index_span[levels.at(i) + position].trailing_ones() as usize;
		}
	}
	
	let result = position * IndexType::BITS as usize + index_span[levels.at(0) + position].trailing_ones() as usize;
	
	index_span[levels.at(0) + position] |= 1 << (result % IndexType::BITS as usize);
	
	for i in 0 .. levels.len
	{
		if ! index_span[levels.at(i) + position] != 0
		{
			break;
		}
		
		let modulus = position % IndexType::BITS as usize;
		position /= IndexType::BITS as usize;
		let offset = if i + 1 == levels.len
		{
			0
		}
		else
		{
			levels.at(i + 1)
		};
		
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
