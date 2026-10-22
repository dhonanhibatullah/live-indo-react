.PHONY: lint generate breaking test

lint:
	@buf lint

generate:
	@buf generate
	@cd .. && cargo fmt --all

breaking:
	@buf breaking --against '.git#branch=development'

test:
	@cd .. && cargo test -p live-indo-proto
