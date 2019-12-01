run PROGRAM:
	cargo run --bin interpreter -- kr/examples/{{PROGRAM}}.kr

test:
	cargo build
	cargo test
	cargo run --bin tester
