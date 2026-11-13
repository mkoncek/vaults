//! Benchmarks of `Repository` against `Vec` in the operations they share, and against a vector of
//! optional values with a list of vacant slots, the usual way to keep indices stable with a `Vec`.
//!
//! The benchmarks which allocate are sensitive to the state the allocator is left in by the previous
//! ones, so for comparisons run each group in its own process, e.g. `cargo bench --bench repository -- '^push_100000/'`.

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkGroup, measurement::WallTime};
use rand::{RngExt, SeedableRng};
use rand::seq::SliceRandom;
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

/// Appending to an empty container, including its growth.
fn bench_push(c: &mut Criterion)
{
	for limit in [1_000_usize, 100_000, 1_000_000]
	{
		let mut group = c.benchmark_group(format!("push_{}", limit));
		configure(&mut group);
		group.bench_function("Vec", |b| b.iter(|| {let mut v = Vec::new(); for i in 0 .. limit {v.push(i);} v}));
		group.bench_function("Repository", |b| b.iter(|| {let mut r = Repository::new(); for i in 0 .. limit {r.insert(i);} r}));
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
		let v: Vec<usize> = (0 .. limit).collect();
		let r: Repository<usize> = (0 .. limit).collect();
		group.bench_function("Vec", |b| b.iter(|| indices.iter().fold(0_usize, |acc, &i| acc.wrapping_add(v[i]))));
		group.bench_function("Repository", |b| b.iter(|| indices.iter().fold(0_usize, |acc, &i| acc.wrapping_add(r[i]))));
		group.finish();
	}
}

/// Iteration over all values of a full container.
fn bench_iter(c: &mut Criterion)
{
	for limit in [1_000_usize, 100_000]
	{
		let mut group = c.benchmark_group(format!("iter_{}", limit));
		configure(&mut group);
		let v: Vec<usize> = (0 .. limit).collect();
		let r: Repository<usize> = (0 .. limit).collect();
		group.bench_function("Vec", |b| b.iter(|| v.iter().fold(0_usize, |acc, v| acc.wrapping_add(*v))));
		group.bench_function("Repository", |b| b.iter(|| r.iter().fold(0_usize, |acc, v| acc.wrapping_add(*v))));
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
		
		let mut r: Repository<usize> = (0 .. limit).collect();
		for &i in removed {r.remove(i);}
		group.bench_function("Repository", |b| b.iter(|| r.iter().fold(0_usize, |acc, v| acc.wrapping_add(*v))));
		
		let mut f = FreeListVec::new();
		for i in 0 .. limit {f.insert(i);}
		for &i in removed {f.remove(i);}
		group.bench_function("FreeListVec", |b| b.iter(|| f.iter().fold(0_usize, |acc, v| acc.wrapping_add(*v))));
		group.finish();
	}
}

/// Removals at random indices of a full container, each followed by an insertion, which reuses the vacated slot.
/// `Vec::swap_remove` is the cheapest removal of `Vec`, but moves the last value to the removed index.
fn bench_churn(c: &mut Criterion)
{
	const OPERATIONS: usize = 10_000;
	let limit = 100_000;
	let mut group = c.benchmark_group(format!("churn_{}", limit));
	configure(&mut group);
	let indices = random_indices(limit, OPERATIONS);
	
	let mut v: Vec<usize> = (0 .. limit).collect();
	group.bench_function("Vec_swap_remove", |b| b.iter(|| {for &i in &indices {let value = v.swap_remove(i); v.push(value);} v.len()}));
	
	let mut r: Repository<usize> = (0 .. limit).collect();
	group.bench_function("Repository", |b| b.iter(|| {for &i in &indices {let value = r.remove(i).unwrap(); r.insert(value);} r.len()}));
	
	let mut f = FreeListVec::new();
	for i in 0 .. limit {f.insert(i);}
	group.bench_function("FreeListVec", |b| b.iter(|| {for &i in &indices {let value = f.remove(i).unwrap(); f.insert(value);} f.slots.len()}));
	group.finish();
}

criterion_group!(benches,
	bench_push, bench_push_reserved, bench_get,
	bench_iter, bench_iter_sparse, bench_churn,
);
criterion_main!(benches);
