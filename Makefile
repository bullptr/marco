.PHONY: test

all: test

test:
	@cls
	@cargo run -- -i "./tests/*.marco.md"
