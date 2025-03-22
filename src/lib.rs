//! Specialized collections.

pub mod svst;

pub trait Comparator<Type: ?Sized>
{
	fn compare(&self, lhs: &Type, rhs: &Type) -> std::cmp::Ordering;
}

pub struct DefaultComparator;

impl DefaultComparator
{
	pub const fn new() -> Self {DefaultComparator {}}
}

impl Default for DefaultComparator
{
	fn default() -> Self {Self::new()}
}

impl<Type> Comparator<Type> for DefaultComparator
where Type: std::cmp::Ord
{
	fn compare(&self, lhs: &Type, rhs: &Type) -> std::cmp::Ordering
	{
		lhs.cmp(rhs)
	}
}
