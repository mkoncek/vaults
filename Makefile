MAKEFLAGS += -r
.PHONY: test bench

test:
	CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="valgrind" DEBUGINFOD_URLS= cargo test --target x86_64-unknown-linux-gnu

bench:
	cargo run --target x86_64-unknown-linux-gnu --release --bin bench
