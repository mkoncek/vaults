use vaults::svst::Repository;
use vaults::svst::AATreeSet;

static PRIME: usize = 18_446_744_073_709_551_557;

fn timed(f: impl std::ops::FnOnce()) -> std::time::Duration
{
	let start = std::time::SystemTime::now();
	std::hint::black_box(f());
	let end = std::time::SystemTime::now();	
	return end.duration_since(start).unwrap();
}

fn repeat(times: usize, mut f: impl std::ops::FnMut()) -> std::time::Duration
{
	let mut d = std::time::Duration::new(0, 0);
	for _ in 0 .. times
	{
		d = d.saturating_add(timed(&mut f));
	}
	return d;
}

fn bench_insert(times: usize, limit: usize)
{
	println!("insertion of {} elements ({} times)", limit, times);
	println!("        Repository: {: >10.2?}", repeat(times, ||
	{
		let mut r = Repository::new();
		for j in 0 .. limit
		{
			r.insert(j);
		}
	}));
	println!("        Vec:        {: >10.2?}", repeat(times, ||
	{
		let mut v = Vec::new();
		for j in 0 .. limit
		{
			v.push(j);
		}
	}));
}

fn bench_remove(times: usize, limit: usize, removed: usize)
{
	let mut dr = std::time::Duration::new(0, 0);
	let mut dv = std::time::Duration::new(0, 0);
	
	println!("removal of {} elements from {} at random positions ({} times)", removed, limit, times);
	
	for _ in 0 .. times
	{
		let mut r = Repository::new();
		for j in 0 .. limit
		{
			r.insert(j);
		}
		dr = dr.saturating_add(timed(||
		{
			for j in 0 .. removed
			{
				r.remove((PRIME + j) % r.len());
			}
		}));
		
		let mut v = Vec::new();
		for j in 0 .. limit
		{
			v.push(j)
		}
		dv = dv.saturating_add(timed(||
		{
			for j in 0 .. removed
			{
				v.remove((PRIME + j) % v.len());
			}
		}));
	}
	
	println!("        Repository: {: >10.2?}", dr);
	println!("        Vec:        {: >10.2?}", dv);
}

fn bench_set_insert(times: usize, limit: usize)
{
	println!("insertion of {} elements ({} times)", limit, times);
	println!("        AATreeSet: {: >10.2?}", repeat(times, ||
	{
		let mut s = AATreeSet::new();
		for j in 0 .. limit
		{
			s.insert(j);
		}
	}));
	println!("        BTreeSet:  {: >10.2?}", repeat(times, ||
	{
		let mut s = std::collections::BTreeSet::new();
		for j in 0 .. limit
		{
			s.insert(j);
		}
	}));
	println!("        HashSet:   {: >10.2?}", repeat(times, ||
	{
		let mut s = std::collections::HashSet::new();
		for j in 0 .. limit
		{
			s.insert(j);
		}
	}));
}

fn bench_set_clear_insert(times: usize, limit: usize)
{
	let mut s = AATreeSet::new();
	println!("insertion of {} elements ({} times)", limit, times);
	println!("        AATreeSet: {: >10.2?}", repeat(times, ||
	{
		s.clear();
		for j in 0 .. limit
		{
			s.insert(j);
		}
	}));
	let mut s = std::collections::BTreeSet::new();
	println!("        BTreeSet:  {: >10.2?}", repeat(times, ||
	{
		s.clear();
		for j in 0 .. limit
		{
			s.insert(j);
		}
	}));
	let mut s = std::collections::HashSet::new();
	println!("        HashSet:   {: >10.2?}", repeat(times, ||
	{
		s.clear();
		for j in 0 .. limit
		{
			s.insert(j);
		}
	}));
}

fn main()
{
	println!("Size of AASet (default): {} bytes", std::mem::size_of::<AATreeSet<i32>>());
	
	bench_insert(1_000, 1_000);
	bench_insert(1_000, 10_000);
	
	bench_remove(1_000, 1_000, 400);
	bench_remove(1_000, 10_000, 400);
	bench_remove(100, 10_000, 4_000);
	
	bench_set_insert(1_000, 1_000);
	bench_set_clear_insert(1_000, 1_000);
	
	bench_set_insert(1_000, 10_000);
	bench_set_clear_insert(1_000, 10_000);
}
