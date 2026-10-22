use criterion::{criterion_group, criterion_main, Criterion, BenchmarkGroup, measurement::WallTime};
use vaults::svst::{AATreeSet, AAVecSet};

static PRIME: usize = 281_474_976_710_597;

fn configure(group: &mut BenchmarkGroup<WallTime>)
{
	group.warm_up_time(std::time::Duration::from_millis(500));
	group.measurement_time(std::time::Duration::from_secs(2));
	group.sample_size(50);
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

criterion_group!(benches,
	bench_set_insert, bench_set_insert_reuse,
	bench_set_remove,
);
criterion_main!(benches);
