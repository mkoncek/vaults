MAKEFLAGS += -r
.PHONY: test test-valgrind bench doc

test:
	cargo test --target x86_64-unknown-linux-gnu

test-valgrind: export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER = valgrind --error-exitcode=1 --leak-check=full
test-valgrind: export DEBUGINFOD_URLS = ""
test-valgrind: test

bench:
	cargo run --target x86_64-unknown-linux-gnu --release --bin bench

doc:
	cargo doc --target x86_64-unknown-linux-gnu
