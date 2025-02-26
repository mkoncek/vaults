use std::ops::DerefMut;

union Variant<Type, const SIZE: usize>
{
	buffer: std::mem::ManuallyDrop<std::mem::MaybeUninit<[std::mem::MaybeUninit<Type>; SIZE]>>,
	vector: std::mem::ManuallyDrop<Vec<Type>>,
}

pub struct SVec<Type, const SIZE: usize>
{
	size: u32,
	variant: Variant<Type, SIZE>,
}

impl<Type, const SIZE: usize> Drop for SVec<Type, SIZE>
{
	fn drop(&mut self)
	{
		unsafe
		{
			if self.size & 1 == 0
			{
				self.clear_buffer();
			}
			else
			{
				std::ptr::drop_in_place(self.variant.vector.deref_mut());
			}
		}
	}
}

impl<Type, const SIZE: usize> Default for SVec<Type, SIZE>
{
	fn default() -> Self
	{
		Self::new()
	}
}

impl<Type, const SIZE: usize> std::fmt::Debug for SVec<Type, SIZE>
where Type: std::fmt::Debug
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
	{
		std::fmt::Debug::fmt(self.as_slice(), f)
	}
}

impl<Type, const SIZE: usize> Clone for SVec<Type, SIZE>
where Type: Clone
{
	fn clone(&self) -> Self
	{
		unsafe
		{
			if self.size & 1 == 0
			{
				let size = self.size >> 1;
				let mut buffer = std::mem::MaybeUninit::<[std::mem::MaybeUninit<Type>; SIZE]>::uninit();
				for i in 0 .. size
				{
					buffer.assume_init_mut()[i as usize].write(
						self.variant.buffer.assume_init_ref()[i as usize].assume_init_ref().clone()
					);
				}
				Self {size: size << 1, variant: Variant {buffer: std::mem::ManuallyDrop::new(buffer)}}
			}
			else
			{
				Self {size: 1, variant: Variant {vector: self.variant.vector.clone()}}
			}
		}
	}
}

impl<Type, const SIZE: usize> AsRef<[Type]> for SVec<Type, SIZE>
{
	fn as_ref(&self) -> &[Type]
	{
		self.as_slice()
	}
}

impl<Type, const SIZE: usize> AsMut<[Type]> for SVec<Type, SIZE>
{
	fn as_mut(&mut self) -> &mut [Type]
	{
		self.as_mut_slice()
	}
}

impl<Type, const SIZE: usize> std::borrow::Borrow<[Type]> for SVec<Type, SIZE>
{
	fn borrow(&self) -> &[Type]
	{
		self.as_slice()
	}
}

impl<Type, const SIZE: usize> std::borrow::BorrowMut<[Type]> for SVec<Type, SIZE>
{
	fn borrow_mut(&mut self) -> &mut [Type]
	{
		self.as_mut_slice()
	}
}

impl<Type, const SIZE: usize> std::ops::Index<usize> for SVec<Type, SIZE>
{
	type Output = Type;
	
	fn index(&self, index: usize) -> &Self::Output
	{
		&self.as_slice()[index]
	}
}

impl<Type, const SIZE: usize> std::ops::IndexMut<usize> for SVec<Type, SIZE>
{
	fn index_mut(&mut self, index: usize) -> &mut Self::Output
	{
		&mut self.as_mut_slice()[index]
	}
}

impl<Type, const SIZE: usize> std::ops::Deref for SVec<Type, SIZE>
{
	type Target = [Type];
	
	fn deref(&self) -> &Self::Target
	{
		self.as_slice()
	}
}

impl<Type, const SIZE: usize> std::ops::DerefMut for SVec<Type, SIZE>
{
	fn deref_mut(&mut self) -> &mut Self::Target
	{
		self.as_mut_slice()
	}
}

impl<Type, const SIZE: usize> std::hash::Hash for SVec<Type, SIZE>
where Type: std::hash::Hash
{
	fn hash<H: std::hash::Hasher>(&self, state: &mut H)
	{
		std::hash::Hash::hash(self.as_slice(), state)
	}
}

impl<Type, const SIZE: usize> std::cmp::PartialEq<[Type]> for SVec<Type, SIZE>
where Type: std::cmp::PartialEq
{
	fn eq(&self, other: &[Type]) -> bool
	{
		std::cmp::PartialEq::eq(self.as_slice(), other)
	}
}

impl<Type, const SIZE: usize, const OSIZE: usize> std::cmp::PartialEq<SVec<Type, OSIZE>> for SVec<Type, SIZE>
where Type: std::cmp::PartialEq
{
	fn eq(&self, other: &SVec<Type, OSIZE>) -> bool
	{
		std::cmp::PartialEq::eq(self.as_slice(), other.as_slice())
	}
}

impl<Type, const SIZE: usize> std::cmp::Eq for SVec<Type, SIZE>
where Type: std::cmp::Eq
{
}

impl<Type, const SIZE: usize> std::cmp::PartialOrd<[Type]> for SVec<Type, SIZE>
where Type: std::cmp::PartialOrd
{
	fn partial_cmp(&self, other: &[Type]) -> Option<std::cmp::Ordering>
	{
		std::cmp::PartialOrd::partial_cmp(self.as_slice(), other)
	}
}

impl<Type, const SIZE: usize, const OSIZE: usize> std::cmp::PartialOrd<SVec<Type, OSIZE>> for SVec<Type, SIZE>
where Type: std::cmp::PartialOrd
{
	fn partial_cmp(&self, other: &SVec<Type, OSIZE>) -> Option<std::cmp::Ordering>
	{
		std::cmp::PartialOrd::partial_cmp(self.as_slice(), other.as_slice())
	}
}

impl<Type, const SIZE: usize> std::cmp::Ord for SVec<Type, SIZE>
where Type: std::cmp::Ord
{
	fn cmp(&self, other: &Self) -> std::cmp::Ordering
	{
		std::cmp::Ord::cmp(self.as_slice(), other.as_slice())
	}
}

impl<Type, const SIZE: usize> std::iter::FromIterator<Type> for SVec<Type, SIZE>
{
	fn from_iter<T: IntoIterator<Item = Type>>(iter: T) -> Self
	{
		let mut result = Self::new();
		result.extend(iter);
		return result;
	}
}

impl<'t, Type, const SIZE: usize> std::iter::IntoIterator for &'t SVec<Type, SIZE>
{
	type Item = &'t Type;
	type IntoIter = std::slice::Iter<'t, Type>;
	
	fn into_iter(self) -> Self::IntoIter
	{
		self.as_slice().into_iter()
	}
}

impl<'t, Type, const SIZE: usize> std::iter::IntoIterator for &'t mut SVec<Type, SIZE>
{
	type Item = &'t mut Type;
	type IntoIter = std::slice::IterMut<'t, Type>;
	
	fn into_iter(self) -> Self::IntoIter
	{
		self.as_mut_slice().into_iter()
	}
}

pub struct IterVal<Type, const SIZE: usize>
{
	value: std::mem::ManuallyDrop<SVec<Type, SIZE>>,
	index: usize,
}

impl<Type, const SIZE: usize> std::iter::Iterator for IterVal<Type, SIZE>
{
	type Item = Type;
	
	fn next(&mut self) -> Option<Self::Item>
	{
		unsafe
		{
			let Some(value) = self.value.as_slice().get(self.index) else
			{
				return None;
			};
			
			self.index += 1;
			return Some(std::ptr::read(value));
		}
	}
	
	fn size_hint(&self) -> (usize, Option<usize>)
	{
		let result = self.value.len() - self.index;
		return (result, Some(result));
	}
	
	fn count(self) -> usize where Self: Sized,
	{
		self.len()
	}
	
	fn last(mut self) -> Option<Self::Item> where Self: Sized,
	{
		self.next_back()
	}
}

impl<Type, const SIZE: usize> std::iter::DoubleEndedIterator for IterVal<Type, SIZE>
{
	fn next_back(&mut self) -> Option<Self::Item>
	{
		self.value.pop()
	}
}

impl<Type, const SIZE: usize> std::iter::ExactSizeIterator for IterVal<Type, SIZE>
{
	fn len(&self) -> usize
	{
		self.value.len() - self.index
	}
}

impl<Type, const SIZE: usize> Drop for IterVal<Type, SIZE>
{
	fn drop(&mut self)
	{
		unsafe
		{
			let slice = self.value.as_mut_slice();
			while self.index < slice.len()
			{
				std::ptr::drop_in_place(&mut slice[self.index]);
				self.index += 1;
			}
			
			if self.value.size & 1 != 0
			{
				self.value.variant.vector.deref_mut().set_len(0);
				std::ptr::drop_in_place(self.value.variant.vector.deref_mut());
			}
		}
	}
}

impl<Type, const SIZE: usize> std::iter::IntoIterator for SVec<Type, SIZE>
{
	type Item = Type;
	type IntoIter = IterVal<Type, SIZE>;
	
	fn into_iter(self) -> Self::IntoIter
	{
		IterVal {value: std::mem::ManuallyDrop::new(self), index: 0}
	}
}

impl<Type, const SIZE: usize> std::iter::Extend<Type> for SVec<Type, SIZE>
{
	fn extend<T: IntoIterator<Item = Type>>(&mut self, iter: T)
	{
		unsafe
		{
			let iterator = iter.into_iter();
			if self.size & 1 == 0
			{
				if let (_, Some(max)) = iterator.size_hint()
				{
					let mut size = self.size >> 1;
					let ptr = self.variant.buffer.deref_mut().as_mut_ptr().cast::<std::mem::MaybeUninit<Type>>();
					
					if size as usize + max > Self::STATIC_CAPACITY
					{
						let mut vec = Vec::with_capacity(size as usize + max);
						let slice = std::slice::from_raw_parts_mut(ptr, size as usize);
						vec.extend(slice.iter_mut().map(|v| std::mem::replace(v, std::mem::MaybeUninit::uninit()).assume_init()));
						vec.extend(iterator);
						self.size = 1;
						self.variant.vector = std::mem::ManuallyDrop::new(vec);
					}
					else
					{
						for v in iterator
						{
							ptr.offset(size as isize).as_mut().unwrap().write(v);
							size += 1;
						}
						
						self.size = size;
						self.size <<= 1;
					}
				}
				else
				{
					for v in iterator
					{
						self.push(v);
					}
				}
			}
			else
			{
				self.variant.vector.deref_mut().extend(iterator);
			}
		}
	}
}

impl<const SIZE: usize> std::io::Write for SVec<u8, SIZE>
{
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>
	{
		self.extend(buf.iter().copied());
		Ok(buf.len())
	}
	
	fn flush(&mut self) -> std::io::Result<()>
	{
		Ok(())
	}
}

impl<Type, const SIZE: usize> SVec<Type, SIZE>
{
	pub const STATIC_CAPACITY: usize = SIZE;
	
	pub const fn new() -> Self
	{
		Self
		{
			size: 0,
			variant: Variant {buffer: std::mem::ManuallyDrop::new(std::mem::MaybeUninit::uninit())},
		}
	}
	
	pub fn len(&self) -> usize
	{
		unsafe
		{
			if self.size & 1 == 0
			{
				(self.size >> 1) as usize
			}
			else
			{
				self.variant.vector.len()
			}
		}
	}
	
	pub fn capacity(&self) -> usize
	{
		unsafe
		{
			if self.size & 1 == 0
			{
				Self::STATIC_CAPACITY
			}
			else
			{
				self.variant.vector.capacity()
			}
		}
	}
	
	pub fn is_empty(&self) -> bool
	{
		self.len() == 0
	}
	
	unsafe fn clear_buffer(&mut self)
	{
		let ptr = self.variant.buffer.deref_mut().as_mut_ptr().cast::<Type>();
		let slice = std::slice::from_raw_parts_mut(ptr, (self.size >> 1) as usize);
		slice.iter_mut().for_each(|v| std::ptr::drop_in_place(v));
	}
	
	pub fn clear(&mut self)
	{
		unsafe
		{
			if self.size & 1 == 0
			{
				self.clear_buffer();
				self.size = 0;
			}
			else
			{
				self.variant.vector.deref_mut().clear();
			}
		}
	}
	
	pub fn as_slice(&self) -> &[Type]
	{
		unsafe
		{
			if self.size & 1 == 0
			{
				let ptr = self.variant.buffer.as_ptr();
				return std::slice::from_raw_parts(ptr.cast(), (self.size >> 1) as usize);
			}
			else
			{
				return self.variant.vector.as_slice();
			}
		}
	}
	
	pub fn as_mut_slice(&mut self) -> &mut [Type]
	{
		unsafe
		{
			if self.size & 1 == 0
			{
				let ptr = self.variant.buffer.deref_mut().as_mut_ptr();
				return std::slice::from_raw_parts_mut(ptr.cast(), (self.size >> 1) as usize);
			}
			else
			{
				return self.variant.vector.deref_mut().as_mut_slice();
			}
		}
	}
	
	pub fn push(&mut self, value: Type)
	{
		unsafe
		{
			if self.size & 1 == 0
			{
				if (self.size >> 1) as usize == Self::STATIC_CAPACITY
				{
					let array = std::ptr::read(self.variant.buffer.as_ptr());
					let mut vector = Vec::new();
					vector.reserve(Self::STATIC_CAPACITY + 1);
					vector.extend(array.into_iter().map(|v| std::mem::MaybeUninit::assume_init(v)));
					vector.push(value);
					self.variant.vector = std::mem::ManuallyDrop::new(vector);
					self.size = 1;
				}
				else
				{
					let new_size = 1 + (self.size >> 1);
					self.variant.buffer.deref_mut().assume_init_mut()[new_size as usize - 1].write(value);
					self.size = new_size;
					self.size <<= 1;
				}
			}
			else
			{
				self.variant.vector.deref_mut().push(value);
			}
		}
	}
	
	pub fn pop(&mut self) -> Option<Type>
	{
		unsafe
		{
			if self.size & 1 == 0
			{
				let mut size = self.size >> 1;
				if size != 0
				{
					size -= 1;
					let array = self.variant.buffer.deref_mut().assume_init_mut();
					self.size = size;
					self.size <<= 1;
					Some(std::ptr::read(array.as_mut_ptr().offset(size as isize)).assume_init())
				}
				else
				{
					None
				}
			}
			else
			{
				self.variant.vector.deref_mut().pop()
			}
		}
	}
}

#[test]
fn test_svec_drop_boxed_empty()
{
	type TSvec = SVec::<Box<i32>, 4>;
	let _svec = TSvec::new();
}

#[test]
fn test_svec_drop_boxed_static()
{
	type TSvec = SVec::<Box<i32>, 4>;
	let mut svec = TSvec::new();
	svec.push(Box::new(0));
}

#[test]
fn test_svec_push_simple()
{
	type TSvec = SVec::<i32, 4>;
	let mut svec = TSvec::new();
	assert!(svec.is_empty());
	assert_eq!(0, svec.len());
	
	for i in 1 .. TSvec::STATIC_CAPACITY as i32 + 2
	{
		svec.push(i);
		assert!(!svec.is_empty());
		assert_eq!(i as usize, svec.len());
		for j in 1 .. i + 1
		{
			assert_eq!(j, svec.as_slice()[j as usize - 1]);
		}
	}
}

#[test]
fn test_svec_push_simple_boxed()
{
	type TSvec = SVec::<Box<i32>, 4>;
	let mut svec = TSvec::new();
	assert!(svec.is_empty());
	assert_eq!(0, svec.len());
	
	for i in 1 .. TSvec::STATIC_CAPACITY as i32 + 2
	{
		svec.push(Box::new(i));
		assert!(!svec.is_empty());
		assert_eq!(i as usize, svec.len());
		for j in 1 .. i + 1
		{
			assert_eq!(j, *svec.as_slice()[j as usize - 1]);
		}
	}
}

#[test]
fn test_svec_pop_simple()
{
	type TSvec = SVec::<i32, 4>;
	let mut svec = TSvec::new();
	
	for i in 1 .. TSvec::STATIC_CAPACITY as i32 + 2
	{
		svec.push(i);
	}
	
	for i in (1 .. TSvec::STATIC_CAPACITY as i32 + 2).rev()
	{
		assert_eq!(i as usize, svec.len());
		assert_eq!(Some(i), svec.pop());
	}
	
	assert!(svec.is_empty());
	assert_eq!(0, svec.len());
	assert_eq!(None, svec.pop());
}

#[test]
fn test_svec_iter_val_boxed_single()
{
	type TSvec = SVec::<Box<i32>, 4>;
	let mut svec = TSvec::new();
	svec.push(Box::new(0));
	
	for v in svec
	{
		assert_eq!(0, *v);
	}
}

#[test]
fn test_svec_iter_val_boxed_single_drop()
{
	type TSvec = SVec::<Box<i32>, 4>;
	let mut svec = TSvec::new();
	svec.push(Box::new(0));
	let _ = svec.into_iter();
}

#[test]
fn test_svec_iter_val_boxed_double()
{
	type TSvec = SVec::<Box<i32>, 4>;
	let mut svec = TSvec::new();
	svec.push(Box::new(0));
	svec.push(Box::new(0));
	
	let mut it = svec.into_iter();
	assert_eq!(0, *it.next().unwrap());
}

#[test]
fn test_svec_iter_val_boxed_multiple()
{
	type TSvec = SVec::<Box<i32>, 4>;
	let mut svec = TSvec::new();
	
	for _ in 1 .. TSvec::STATIC_CAPACITY as i32 + 2
	{
		svec.push(Box::new(0));
	}
	
	for v in svec
	{
		assert_eq!(0, *v);
	}
}

#[test]
fn test_svec_iter_val_boxed_multiple_drop()
{
	type TSvec = SVec::<Box<i32>, 4>;
	let mut svec = TSvec::new();
	
	for _ in 1 .. TSvec::STATIC_CAPACITY as i32 + 2
	{
		svec.push(Box::new(0));
	}
	
	let _ = svec.into_iter();
}
