.PHONY: init build build-release test run run-all dev check fmt lint clean docker-build docker-test docker-shell docker-clean help

help:
	@echo "Payment Engine - Available Make Targets:"
	@echo ""
	@echo "  make init          - Initialize development environment"
	@echo "  make build         - Build debug version"
	@echo "  make build-release - Build optimized release version"
	@echo "  make test          - Run all tests"
	@echo "  make run-all       - Run CLI with all example input files"
	@echo "  make dev           - Run in development mode with sample input"
	@echo "  make check         - Run clippy linter"
	@echo "  make fmt           - Check code formatting"
	@echo "  make lint          - Run both clippy and fmt checks"
	@echo "  make clean         - Clean build artifacts"
	@echo ""
	@echo "Docker Targets:"
	@echo "  make docker-build  - Build Docker image for the payment engine"
	@echo "  make docker-run FILE=<path> - Run payment engine in Docker with a CSV file"
	@echo "  make docker-shell  - Get a shell inside the Docker container"
	@echo "  make docker-clean  - Remove Docker image"
	@echo ""
	@echo "Examples:"
	@echo "  make docker-run FILE=example_inputs/deposit_withdraw.csv"
	@echo "  make docker-run FILE=/path/to/your/file.csv"
	@echo ""

init:
	@echo "Initializing payment-engine development environment..."
	@command -v cargo >/dev/null 2>&1 || { echo "Error: Rust/Cargo not installed. Install from https://rustup.rs/"; exit 1; }
	@echo "✓ Rust toolchain found"
	cargo build
	@echo "✓ Dependencies installed"
	@echo "✓ Project built successfully"
	@echo ""
	@echo "Ready to develop! Try 'make test' or 'make run'"

build:
	cargo build --workspace

build-release:
	cargo build --workspace --release

test:
	cargo test --workspace --all-features

run-all:
	@echo "Running CLI with all example inputs..."
	@echo ""
	@for file in $$(find example_inputs -name "*.csv" -type f | sort); do \
		echo "Processing: $$file"; \
		echo "============================================"; \
		cargo run -q -- "$$file" 2>&1 | head -20; \
		echo ""; \
	done
	@echo "✓ Finished processing all example files"

check:
	cargo clippy --workspace --all-features -- -D warnings

fmt:
	cargo fmt --all --check

lint: check fmt
	@echo "✓ All lint checks passed"

clean:
	cargo clean

# Docker targets
docker-build:
	@echo "Building Docker image for payment-engine..."
	docker build -t payment-engine:latest .
	@echo "✓ Docker image built successfully: payment-engine:latest"

docker-run:
	@if [ -z "$(FILE)" ]; then \
		echo "Error: Please specify a CSV file to process"; \
		echo "Usage: make docker-run FILE=<path-to-csv>"; \
		echo "Example: make docker-run FILE=example_inputs/deposit_withdraw.csv"; \
		exit 1; \
	fi
	@if [ ! -f "$(FILE)" ]; then \
		echo "Error: File '$(FILE)' not found"; \
		exit 1; \
	fi
	@docker run --rm \
		-v "$(shell pwd)/$(FILE):/data/input.csv:ro" \
		payment-engine:latest \
		/data/input.csv

docker-shell:
	@echo "Starting shell in Docker container..."
	@echo "Note: The payment-engine binary is available at /usr/local/bin/payment-engine"
	@docker run --rm -it \
		-v "$(shell pwd)/example_inputs:/data/example_inputs:ro" \
		--entrypoint /bin/bash \
		payment-engine:latest

docker-clean:
	@echo "Removing Docker image..."
	docker rmi payment-engine:latest 2>/dev/null || true
	@echo "✓ Docker image removed"
