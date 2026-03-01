# forge-lib Makefile

.PHONY: build clean test lint check install verify

RELEASE_DIR := target/release
BIN_DIR     := bin
BINARIES    := strip-front install-agents install-skills validate-module yaml

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

SCOPE     ?= workspace
SKILL_SRC  = skills
SKILLS     = BuildSystem
INSTALL_SKILLS  = $(BIN_DIR)/install-skills
VALIDATE_MODULE = $(BIN_DIR)/validate-module

include mk/skills/install.mk
include mk/skills/verify.mk

install: install-skills
verify: verify-skills

clean:
	cargo clean
	@command rm -rf $(BIN_DIR)
