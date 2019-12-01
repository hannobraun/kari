run PROGRAM:
	cargo run --bin kari -- kr/examples/{{PROGRAM}}.kr

test:
	cargo build
	cargo test
	cargo run --bin tester
