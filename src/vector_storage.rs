#[derive(Debug)]
pub struct VectorStorage
{
	pub(super) data: std::ptr::NonNull<u8>,
	pub(super) capacity: usize,
}

impl VectorStorage
{
	pub const fn new<Type>() -> Self
	{
		Self
		{
			data: std::ptr::NonNull::<(crate::bit_indexing::IndexType, Type)>::dangling().cast(),
			capacity: 0,
		}
	}
	
	pub fn default_capacity_growth(capacity: usize) -> usize
	{
		8 + capacity + (capacity + 1) / 2
	}
}

unsafe impl Send for VectorStorage {}
unsafe impl Sync for VectorStorage {}