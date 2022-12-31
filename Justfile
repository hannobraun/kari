run PROGRAM:
	cargo run --bin kari -- kr/examples/{{PROGRAM}}.kr

test:
	cargo clippy
	cargo test
	cargo run --bin tester
