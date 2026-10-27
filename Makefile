.PHONY: all build release test run clean fmt clippy check install changelog changelog-preview help

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
	@echo "  fmt            Format with rustfmt"
	@echo "  clippy         Run clippy linter"
	@echo "  check          Check with cargo check"
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
	cargo fmt -- --check

fmt-fix:
	cargo fmt

clippy:
	cargo clippy --bin $(BIN) -- -D warnings

check:
	cargo check --bin $(BIN)

install: release
	mkdir -p $(HOME)/.cargo/bin
	cp $(RELEASE_DIR)/$(BIN) $(HOME)/.cargo/bin/

changelog:
	git-cliff --output CHANGELOG.md

changelog-preview:
	git-cliff
