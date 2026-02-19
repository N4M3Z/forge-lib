# forge-lib Makefile

.PHONY: build clean test lint check

RELEASE_DIR := target/release
BIN_DIR     := bin
BINARIES    := strip-front install-agents install-skills validate-module install-codex-agent-config

build:
	cargo build --release
	@mkdir -p $(BIN_DIR)
	@for b in $(BINARIES); do \
	  ln -sf ../$(RELEASE_DIR)/$$b $(BIN_DIR)/$$b; \
	done

test:
	cargo test

lint:
	cargo fmt --check
	cargo clippy -- -D warnings

check:
	@for b in $(BINARIES); do \
	  if [ -x "$(BIN_DIR)/$$b" ]; then \
	    echo "  ok $$b"; \
	  else \
	    echo "  MISSING $$b (run: make build)"; \
	  fi; \
	done

clean:
	cargo clean
	@command rm -rf $(BIN_DIR)
