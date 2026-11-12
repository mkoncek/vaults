use criterion::{criterion_group, criterion_main, Criterion, BenchmarkGroup, measurement::WallTime};
use rand::{RngExt, SeedableRng};
use rand::seq::SliceRandom;
use vaults::svst::{AATreeSet, AAVecSet};

static PRIME: usize = 281_474_976_710_597;

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

fn bench_set_insert(c: &mut Criterion)
{
	for limit in [1_000_usize, 10_000]
	{
		let mut group = c.benchmark_group(format!("set_insert_{}", limit));
		configure(&mut group);
		group.bench_function("AATreeSet", |b| b.iter(|| {let mut s = AATreeSet::new(); for j in 0 .. limit {s.insert(j);}}));
		group.bench_function("AAVecSet", |b| b.iter(|| {let mut s = AAVecSet::new(); for j in 0 .. limit {s.insert(j);}}));
		group.bench_function("BTreeSet", |b| b.iter(|| {let mut s = std::collections::BTreeSet::new(); for j in 0 .. limit {s.insert(j);}}));
		group.finish();
	}
}

fn bench_set_insert_random(c: &mut Criterion)
{
	for limit in [1_000_usize, 100_000]
	{
		let mut group = c.benchmark_group(format!("set_insert_random_{}", limit));
		configure(&mut group);
		let values = shuffled(limit);
		group.bench_function("AATreeSet", |b| b.iter(|| {let mut s = AATreeSet::new(); for j in &values {s.insert(*j);} s.len()}));
		group.bench_function("AAVecSet", |b| b.iter(|| {let mut s = AAVecSet::new(); for j in &values {s.insert(*j);} s.len()}));
		group.bench_function("BTreeSet", |b| b.iter(|| {let mut s = std::collections::BTreeSet::new(); for j in &values {s.insert(*j);} s.len()}));
		group.finish();
	}
}

fn bench_set_insert_reuse(c: &mut Criterion)
{
	for limit in [1_000_usize, 10_000]
	{
		let mut group = c.benchmark_group(format!("set_insert_reuse_{}", limit));
		configure(&mut group);
		let mut s = AATreeSet::new();
		group.bench_function("AATreeSet", |b| b.iter(|| {s.clear(); for j in 0 .. limit {s.insert(j);}}));
		let mut s = AAVecSet::new();
		group.bench_function("AAVecSet", |b| b.iter(|| {s.clear(); for j in 0 .. limit {s.insert(j);}}));
		let mut s = std::collections::BTreeSet::new();
		group.bench_function("BTreeSet", |b| b.iter(|| {s.clear(); for j in 0 .. limit {s.insert(j);}}));
		group.finish();
	}
}

fn bench_set_remove(c: &mut Criterion)
{
	for limit in [1_000_usize, 10_000]
	{
		let mut group = c.benchmark_group(format!("set_remove_{}", limit));
		configure(&mut group);
		let removed: Vec<usize> = (0 .. limit).map(|i| i * PRIME % limit).collect();
		let batch_size = if limit >= 10_000 {criterion::BatchSize::LargeInput} else {criterion::BatchSize::SmallInput};
		group.bench_function("AATreeSet", |b| b.iter_batched(
			|| {let mut s = AATreeSet::new(); for j in 0 .. limit {s.insert(j);} s},
			|mut s| for j in &removed {s.remove(j);},
			batch_size,
		));
		group.bench_function("AAVecSet", |b| b.iter_batched(
			|| {let mut s = AAVecSet::new(); for j in 0 .. limit {s.insert(j);} s},
			|mut s| for j in &removed {s.remove(j);},
			batch_size,
		));
		group.bench_function("BTreeSet", |b| b.iter_batched(
			|| {let mut s = std::collections::BTreeSet::new(); for j in 0 .. limit {s.insert(j);} s},
			|mut s| for j in &removed {s.remove(j);},
			batch_size,
		));
		group.finish();
	}
}

/// A fixed number of lookups of random keys, about half of which are present,
/// in sets of increasing size built in random order, to cover the cache hierarchy.
fn bench_set_contains(c: &mut Criterion)
{
	const LOOKUPS: usize = 10_000;
	
	for limit in [1_000_usize, 100_000, 1_000_000]
	{
		let mut group = c.benchmark_group(format!("set_contains_{}", limit));
		configure(&mut group);
		// The set holds the even values `0 .. 2 * limit`
		let values: Vec<usize> = shuffled(limit).into_iter().map(|v| 2 * v).collect();
		let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
		let keys: Vec<usize> = (0 .. LOOKUPS).map(|_| rng.random_range(0 .. 2 * limit)).collect();
		
		let mut s = AATreeSet::new();
		for v in &values {s.insert(*v);}
		group.bench_function("AATreeSet", |b| b.iter(|| keys.iter().filter(|k| s.contains(*k)).count()));
		
		let mut s = AAVecSet::new();
		for v in &values {s.insert(*v);}
		group.bench_function("AAVecSet", |b| b.iter(|| keys.iter().filter(|k| s.contains(*k)).count()));
		
		let s: std::collections::BTreeSet<usize> = values.iter().copied().collect();
		group.bench_function("BTreeSet", |b| b.iter(|| keys.iter().filter(|k| s.contains(*k)).count()));
		group.finish();
	}
}

/// Lookups of consecutive keys, whose search paths are predictable, in contrast to `bench_set_contains`.
fn bench_set_contains_sequential(c: &mut Criterion)
{
	const LOOKUPS: usize = 10_000;
	
	for limit in [1_000_usize, 100_000]
	{
		let mut group = c.benchmark_group(format!("set_contains_sequential_{}", limit));
		configure(&mut group);
		let values: Vec<usize> = shuffled(limit).into_iter().map(|v| 2 * v).collect();
		let keys: Vec<usize> = (0 .. LOOKUPS).map(|i| i % (2 * limit)).collect();
		
		let mut s = AATreeSet::new();
		for v in &values {s.insert(*v);}
		group.bench_function("AATreeSet", |b| b.iter(|| keys.iter().filter(|k| s.contains(*k)).count()));
		
		let mut s = AAVecSet::new();
		for v in &values {s.insert(*v);}
		group.bench_function("AAVecSet", |b| b.iter(|| keys.iter().filter(|k| s.contains(*k)).count()));
		
		let s: std::collections::BTreeSet<usize> = values.iter().copied().collect();
		group.bench_function("BTreeSet", |b| b.iter(|| keys.iter().filter(|k| s.contains(*k)).count()));
		group.finish();
	}
}

fn bench_set_iter(c: &mut Criterion)
{
	for limit in [1_000_usize, 100_000]
	{
		let mut group = c.benchmark_group(format!("set_iter_{}", limit));
		configure(&mut group);
		let values = shuffled(limit);
		
		let mut s = AATreeSet::new();
		for v in &values {s.insert(*v);}
		group.bench_function("AATreeSet", |b| b.iter(|| s.iter().fold(0_usize, |acc, v| acc.wrapping_add(*v))));
		group.bench_function("AATreeSet_collect", |b| b.iter(|| s.iter().copied().collect::<Vec<_>>()));
		group.bench_function("AATreeSet_unordered", |b| b.iter(|| s.iter_unordered().fold(0_usize, |acc, v| acc.wrapping_add(*v))));
		
		let mut s = AAVecSet::new();
		for v in &values {s.insert(*v);}
		group.bench_function("AAVecSet", |b| b.iter(|| s.iter().fold(0_usize, |acc, v| acc.wrapping_add(*v))));
		group.bench_function("AAVecSet_unordered", |b| b.iter(|| s.iter_unordered().fold(0_usize, |acc, v| acc.wrapping_add(*v))));
		
		let s: std::collections::BTreeSet<usize> = values.iter().copied().collect();
		group.bench_function("BTreeSet", |b| b.iter(|| s.iter().fold(0_usize, |acc, v| acc.wrapping_add(*v))));
		group.bench_function("BTreeSet_collect", |b| b.iter(|| s.iter().copied().collect::<Vec<_>>()));
		group.finish();
	}
}

criterion_group!(benches,
	bench_set_insert, bench_set_insert_random, bench_set_insert_reuse,
	bench_set_remove, bench_set_contains, bench_set_contains_sequential, bench_set_iter,
);
criterion_main!(benches);
