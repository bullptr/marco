.PHONY: test

all: test

test:
	@cls
	@cargo run -- -i "./tests/*.marco.md" --verbose

hello:
	@cls
	@cargo run -- -i "./tests/hello.marco.md"
