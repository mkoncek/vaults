//! Benchmarks of `Repository` against `Vec` in the operations they share, against a vector of
//! optional values with a list of vacant slots, the usual way to keep indices stable with a `Vec`,
//! and against `LinkedList`, whose nodes do not move either.
//!
//! A list has no indexing and no capacity, so it is missing from the reads and the reserved appending.
//!
//! The benchmarks which allocate are sensitive to the state the allocator is left in by the previous
//! ones, so for comparisons run each group in its own process, e.g. `cargo bench --bench repository -- '^push_100000/'`.

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkGroup, measurement::WallTime};
use rand::{RngExt, SeedableRng};
use rand::seq::SliceRandom;
use std::collections::LinkedList;
use vaults::svst::Repository;

fn configure(group: &mut BenchmarkGroup<WallTime>)
{
	group.warm_up_time(std::time::Duration::from_millis(500));
	group.measurement_time(std::time::Duration::from_secs(2));
	group.sample_size(50);
}

/// The values `0 .. limit` in a fixed pseudo-random order.
fn shuffled(limit: usize) -> Vec<usize>
{
	let mut result: Vec<usize> = (0 .. limit).collect();
	result.shuffle(&mut rand::rngs::SmallRng::seed_from_u64(0));
	return result;
}

/// `count` pseudo-random indices lower than `limit`.
fn random_indices(limit: usize, count: usize) -> Vec<usize>
{
	let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
	return (0 .. count).map(|_| rng.random_range(0 .. limit)).collect();
}

// The containers measured by more benchmarks are built on first use, so that the containers of the other benchmarks
// do not change where the measured one is in memory when benchmarks are selected, which changes the times considerably.

fn full_vec(limit: usize) -> Vec<usize>
{
	(0 .. limit).collect()
}

fn full_repository(limit: usize) -> Repository<usize>
{
	(0 .. limit).collect()
}

fn full_free_list(limit: usize) -> FreeListVec<usize>
{
	let mut result = FreeListVec::new();
	for i in 0 .. limit {result.insert(i);}
	return result;
}

/// A list of the values `0 .. limit` for which `keep` holds, in their order, whose nodes are allocated
/// in a pseudo-random order among those of the other values, which are freed then.
/// Following it jumps across memory, like in a list after many modifications,
/// while the nodes of a list built in order are mostly consecutive.
fn scattered_list(limit: usize, keep: impl Fn(usize) -> bool) -> LinkedList<usize>
{
	let mut singles: Vec<LinkedList<usize>> = (0 .. limit).map(|_| LinkedList::new()).collect();
	
	for i in shuffled(limit)
	{
		singles[i].push_back(i);
	}
	
	// Appending moves the nodes without allocating them again
	let mut result = LinkedList::new();
	
	for (i, mut single) in singles.into_iter().enumerate()
	{
		if keep(i)
		{
			result.append(&mut single);
		}
	}
	
	return result;
}

/// A vector of optional values with a list of the vacant slots, which keeps the indices of the values stable
/// like `Repository`, but reuses the most recently vacated slot instead of the lowest one.
struct FreeListVec<Type>
{
	slots: Vec<Option<Type>>,
	vacant: Vec<usize>,
}

impl<Type> FreeListVec<Type>
{
	fn new() -> Self
	{
		Self {slots: Vec::new(), vacant: Vec::new()}
	}
	
	fn insert(&mut self, value: Type) -> usize
	{
		match self.vacant.pop()
		{
			Some(index) =>
			{
				self.slots[index] = Some(value);
				index
			},
			None =>
			{
				self.slots.push(Some(value));
				self.slots.len() - 1
			},
		}
	}
	
	fn remove(&mut self, index: usize) -> Option<Type>
	{
		let value = self.slots.get_mut(index)?.take();
		
		if value.is_some()
		{
			self.vacant.push(index);
		}
		
		return value;
	}
	
	fn iter(&self) -> impl Iterator<Item = &Type>
	{
		self.slots.iter().flatten()
	}
}

/// Appending to an empty container, including its growth and dropping it, which frees every node of a list.
fn bench_push(c: &mut Criterion)
{
	for limit in [1_000_usize, 100_000, 1_000_000]
	{
		let mut group = c.benchmark_group(format!("push_{}", limit));
		configure(&mut group);
		group.bench_function("Vec", |b| b.iter(|| {let mut v = Vec::new(); for i in 0 .. limit {v.push(i);} v}));
		group.bench_function("Repository", |b| b.iter(|| {let mut r = Repository::new(); for i in 0 .. limit {r.insert(i);} r}));
		group.bench_function("LinkedList", |b| b.iter(|| {let mut l = LinkedList::new(); for i in 0 .. limit {l.push_back(i);} l}));
		group.finish();
	}
}

/// Appending to an empty container with reserved capacity, the cost of an insertion alone.
fn bench_push_reserved(c: &mut Criterion)
{
	for limit in [1_000_usize, 100_000]
	{
		let mut group = c.benchmark_group(format!("push_reserved_{}", limit));
		configure(&mut group);
		group.bench_function("Vec", |b| b.iter(|| {let mut v = Vec::with_capacity(limit); for i in 0 .. limit {v.push(i);} v}));
		group.bench_function("Repository", |b| b.iter(|| {let mut r = Repository::with_capacity(limit); for i in 0 .. limit {r.insert(i);} r}));
		group.finish();
	}
}

/// Reads at random indices, in a container fitting in the cache and one which does not.
fn bench_get(c: &mut Criterion)
{
	const LOOKUPS: usize = 10_000;
	
	for limit in [1_000_usize, 1_000_000]
	{
		let mut group = c.benchmark_group(format!("get_{}", limit));
		configure(&mut group);
		let indices = random_indices(limit, LOOKUPS);
		let (mut v, mut r) = (None, None);
		group.bench_function("Vec", |b| {let v = v.get_or_insert_with(|| full_vec(limit)); b.iter(|| indices.iter().fold(0_usize, |acc, &i| acc.wrapping_add(v[i])))});
		group.bench_function("Repository", |b| {let r = r.get_or_insert_with(|| full_repository(limit)); b.iter(|| indices.iter().fold(0_usize, |acc, &i| acc.wrapping_add(r[i])))});
		group.finish();
	}
}

/// Iteration over all values of a full container, for lists also with scattered nodes.
fn bench_iter(c: &mut Criterion)
{
	for limit in [1_000_usize, 100_000]
	{
		let mut group = c.benchmark_group(format!("iter_{}", limit));
		configure(&mut group);
		let (mut v, mut r, mut l, mut scattered) = (None, None, None, None);
		group.bench_function("Vec", |b| {let v = v.get_or_insert_with(|| full_vec(limit)); b.iter(|| v.iter().fold(0_usize, |acc, v| acc.wrapping_add(*v)))});
		group.bench_function("Repository", |b| {let r = r.get_or_insert_with(|| full_repository(limit)); b.iter(|| r.iter().fold(0_usize, |acc, v| acc.wrapping_add(*v)))});
		group.bench_function("LinkedList", |b| {let l = l.get_or_insert_with(|| (0 .. limit).collect::<LinkedList<usize>>()); b.iter(|| l.iter().fold(0_usize, |acc, v| acc.wrapping_add(*v)))});
		group.bench_function("LinkedList_scattered", |b|
		{
			let l = scattered.get_or_insert_with(|| {let l = scattered_list(limit, |_| true); assert!(l.iter().copied().eq(0 .. limit)); l});
			b.iter(|| l.iter().fold(0_usize, |acc, v| acc.wrapping_add(*v)))
		});
		group.finish();
	}
}

/// Iteration over the values after removing half of them at random positions,
/// which `Vec` cannot do without moving the values.
fn bench_iter_sparse(c: &mut Criterion)
{
	for limit in [1_000_usize, 100_000]
	{
		let mut group = c.benchmark_group(format!("iter_sparse_{}", limit));
		configure(&mut group);
		let removed = &shuffled(limit)[.. limit / 2];
		let mut kept = vec![true; limit];
		for &i in removed {kept[i] = false;}
		let expected = || (0 .. limit).filter(|&i| kept[i]);
		let (mut r, mut f, mut l) = (None, None, None);
		
		group.bench_function("Repository", |b|
		{
			let r = r.get_or_insert_with(|| {let mut r = full_repository(limit); for &i in removed {r.remove(i);} assert!(r.iter().copied().eq(expected())); r});
			b.iter(|| r.iter().fold(0_usize, |acc, v| acc.wrapping_add(*v)))
		});
		group.bench_function("FreeListVec", |b|
		{
			let f = f.get_or_insert_with(|| {let mut f = full_free_list(limit); for &i in removed {f.remove(i);} assert!(f.iter().copied().eq(expected())); f});
			b.iter(|| f.iter().fold(0_usize, |acc, v| acc.wrapping_add(*v)))
		});
		group.bench_function("LinkedList_scattered", |b|
		{
			let l = l.get_or_insert_with(|| {let l = scattered_list(limit, |i| kept[i]); assert!(l.iter().copied().eq(expected())); l});
			b.iter(|| l.iter().fold(0_usize, |acc, v| acc.wrapping_add(*v)))
		});
		group.finish();
	}
}

/// Iteration moving the values out of a full container, which frees it afterwards.
fn bench_into_iter(c: &mut Criterion)
{
	let limit = 100_000;
	let mut group = c.benchmark_group(format!("into_iter_{}", limit));
	configure(&mut group);
	let batch_size = criterion::BatchSize::LargeInput;
	
	let (mut v, mut r, mut l) = (None, None, None);
	group.bench_function("Vec", |b| {let v = v.get_or_insert_with(|| full_vec(limit)); b.iter_batched(|| v.clone(), |v| v.into_iter().fold(0_usize, |acc, v| acc.wrapping_add(v)), batch_size)});
	group.bench_function("Repository", |b| {let r = r.get_or_insert_with(|| full_repository(limit)); b.iter_batched(|| r.clone(), |r| r.into_iter().fold(0_usize, |acc, v| acc.wrapping_add(v)), batch_size)});
	group.bench_function("LinkedList", |b| {let l = l.get_or_insert_with(|| (0 .. limit).collect::<LinkedList<usize>>()); b.iter_batched(|| l.clone(), |l| l.into_iter().fold(0_usize, |acc, v| acc.wrapping_add(v)), batch_size)});
	group.finish();
}

/// Removals at random indices of a full container, each followed by an insertion, which reuses the vacated slot.
/// `Vec::swap_remove` is the cheapest removal of `Vec`, but moves the last value to the removed index.
/// A list cannot remove at a position in constant time with the stable interface, so it removes the first
/// value and appends it, which frees a node, allocates one and links it like a removal and an insertion.
fn bench_churn(c: &mut Criterion)
{
	const OPERATIONS: usize = 10_000;
	let limit = 100_000;
	let mut group = c.benchmark_group(format!("churn_{}", limit));
	configure(&mut group);
	let indices = random_indices(limit, OPERATIONS);
	
	let (mut v, mut r, mut f, mut l) = (None, None, None, None);
	group.bench_function("Vec_swap_remove", |b| {let v = v.get_or_insert_with(|| full_vec(limit)); b.iter(|| {for &i in &indices {let value = v.swap_remove(i); v.push(value);} v.len()})});
	group.bench_function("Repository", |b| {let r = r.get_or_insert_with(|| full_repository(limit)); b.iter(|| {for &i in &indices {let value = r.remove(i).unwrap(); r.insert(value);} r.len()})});
	group.bench_function("FreeListVec", |b| {let f = f.get_or_insert_with(|| full_free_list(limit)); b.iter(|| {for &i in &indices {let value = f.remove(i).unwrap(); f.insert(value);} f.slots.len()})});
	group.bench_function("LinkedList_pop_push", |b| {let l = l.get_or_insert_with(|| (0 .. limit).collect::<LinkedList<usize>>()); b.iter(|| {for _ in &indices {let value = l.pop_front().unwrap(); l.push_back(value);} l.len()})});
	group.finish();
}

criterion_group!(benches,
	bench_push, bench_push_reserved, bench_get,
	bench_iter, bench_iter_sparse, bench_into_iter, bench_churn,
);
criterion_main!(benches);
