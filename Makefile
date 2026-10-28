.PHONY: all build release test run clean fmt fmt-check clippy check lint fix install changelog changelog-preview help

BIN := anily
SRC := src
TARGET_DIR := target
RELEASE_DIR := $(TARGET_DIR)/release
DEBUG_DIR := $(TARGET_DIR)/debug

all: build

help:
	@echo "Targets:"
	@echo "  all            Build debug binary (default)"
	@echo "  build          Build debug binary"
	@echo "  release        Build optimized release binary"
	@echo "  test           Run cargo test"
	@echo "  run            Run the debug binary"
	@echo "  clean          Remove target directory"
	@echo "  fmt            Format codebase with rustfmt"
	@echo "  fmt-check      Verify formatting (used in CI)"
	@echo "  clippy         Run clippy linter"
	@echo "  check          Check with cargo check"
	@echo "  lint           Run formatting check and clippy"
	@echo "  fix            Auto-format and auto-fix clippy issues"
	@echo "  install        Install binary to ~/.cargo/bin"
	@echo "  changelog      Generate CHANGELOG.md"
	@echo "  changelog-preview  Preview changelog"

build:
	cargo build --bin $(BIN)

release:
	cargo build --release --bin $(BIN)

test:
	cargo test

run: build
	./$(DEBUG_DIR)/$(BIN)

clean:
	rm -rf $(TARGET_DIR)

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --bin $(BIN) -- -D warnings

check:
	cargo check --bin $(BIN)

lint: fmt-check clippy

fix: fmt
	cargo clippy --fix --allow-dirty --allow-staged

install: release
	mkdir -p $(HOME)/.cargo/bin
	cp $(RELEASE_DIR)/$(BIN) $(HOME)/.cargo/bin/

changelog:
	git-cliff --output CHANGELOG.md

changelog-preview:
	git-cliff
