.PHONY: test

all: test

test:
	@cls
	@cargo run -- -i "./tests/*.marco.md" --verbose

basic:
	@cls
	@cargo run -- -i "./tests/basic.marco.md"

multi:
	@cls
	@cargo run -- -i "./tests/multi-tests.marco.md"
