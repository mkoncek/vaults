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
				std::ptr::drop_in_place(self.variant.vector.deref_mut().as_mut_ptr());
			}
		}
	}
}

impl<Type, const SIZE: usize> SVec<Type, SIZE>
{
	pub const fn new() -> Self
	{
		Self
		{
			size: 0,
			variant: Variant {buffer: std::mem::ManuallyDrop::new(std::mem::MaybeUninit::uninit())},
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
				let new_size = 1 + (self.size >> 1);
				if new_size as usize == SIZE
				{
					let array = std::ptr::read(self.variant.buffer.as_ptr());
					let vector = Vec::from_iter(array.into_iter().map(|v| std::mem::MaybeUninit::assume_init(v)));
					self.variant.vector = std::mem::ManuallyDrop::new(vector);
					self.size = 1;
				}
				else
				{
					self.variant.buffer.deref_mut().assume_init_mut()[new_size as usize - 1].write(value);
					self.size = new_size;
					self.size <<= 1;
				}
			}
			else
			{
				return self.variant.vector.deref_mut().push(value);
			}
		}
	}
}
