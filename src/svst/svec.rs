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
				let ptr = self.variant.buffer.deref_mut().as_mut_ptr().cast::<Type>();
				std::slice::from_raw_parts(ptr, (self.size >> 1) as usize).iter().for_each(std::mem::drop);
			}
			else
			{
				std::ptr::drop_in_place(self.variant.vector.deref_mut());
			}
		}
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
	
	pub fn is_empty(&self) -> bool
	{
		unsafe
		{
			if self.size & 1 == 0
			{
				self.size >> 1 == 0
			}
			else
			{
				self.variant.vector.is_empty()
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
				if (self.size >> 1) as usize == SIZE
				{
					let array = std::ptr::read(self.variant.buffer.as_ptr());
					let mut vector = Vec::new();
					vector.reserve(SIZE + 1);
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
fn test_svec_push_simple()
{
	type TSvec = SVec::<i32, 4>;
	let mut svec = TSvec::new();
	assert!(svec.is_empty());
	
	for i in 1 .. TSvec::STATIC_CAPACITY as i32 + 2
	{
		svec.push(i);
		assert!(!svec.is_empty());
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
	
	for i in 1 .. TSvec::STATIC_CAPACITY as i32 + 2
	{
		svec.push(Box::new(i));
		assert!(!svec.is_empty());
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
		assert_eq!(Some(i), svec.pop());
	}
	
	assert!(svec.is_empty());
	assert_eq!(None, svec.pop());
}
