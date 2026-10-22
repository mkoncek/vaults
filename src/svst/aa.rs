//! Sorted set and map collections built on a self-balancing
//! [AA tree](https://en.wikipedia.org/wiki/AA_tree).
//!
//! Two storage backends are available:
//!
//! - [`Repository`](crate::svst::Repository) -- indices are stable: inserting or removing
//!   other elements never moves existing nodes.  Suitable when external code holds
//!   handles (indices) into the tree.
//! - [`Vec`] -- nodes are packed densely; removal uses `swap_remove`, so the last node
//!   is moved into the vacated slot.  Better cache locality and lower memory overhead,
//!   but indices are not stable across removals.

mod node;
pub mod tree;
mod set;
mod map;

pub use set::Set;
pub use map::Map;
pub use set::VecSet;
pub use map::VecMap;
