.PHONY: lint generate breaking test

lint:
	@buf lint

generate:
	@buf generate
	@cd .. && cargo fmt --all

breaking:
	@buf breaking --against '../../.git#branch=development,subdir=shared/proto'

test:
	@cd .. && cargo test -p live-indo-proto
