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
			data: std::ptr::NonNull::<(crate::svst::bit_indexing::IndexType, Type)>::dangling().cast(),
			capacity: 0,
		}
	}
	
	/// Saturates, the allocation fails for such capacity anyway.
	pub fn default_capacity_growth(capacity: usize) -> usize
	{
		capacity.saturating_add(capacity.div_ceil(2)).saturating_add(8)
	}
	
	pub fn default_capacity_for(mut start: usize, capacity: usize) -> usize
	{
		while start < capacity
		{
			start = Self::default_capacity_growth(start);
		}
		return start;
	}
}

unsafe impl Send for VectorStorage {}
unsafe impl Sync for VectorStorage {}
