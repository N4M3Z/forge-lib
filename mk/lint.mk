# lint.mk — linting targets for forge modules
#
# Requires: LIB_DIR, SKILL_SRC, AGENT_SRC set before include (from common.mk)
# Provides: lint-schema, lint-docs, lint-rules, lint-shell
#
# Schema resolution: local override > $(LIB_DIR)/rules/ canonical

.PHONY: lint-schema lint-docs lint-rules lint-shell

lint-schema:
	@if ! command -v mdschema >/dev/null 2>&1; then \
	  echo "  SKIP mdschema (not installed — brew install jackchuka/tap/mdschema)"; \
	else \
	  SKILL_SCHEMA=""; \
	  if [ -f $(SKILL_SRC)/.mdschema ]; then \
	    SKILL_SCHEMA="$(SKILL_SRC)/.mdschema"; \
	  elif [ -f $(LIB_DIR)/rules/skill.mdschema ]; then \
	    SKILL_SCHEMA="$(LIB_DIR)/rules/skill.mdschema"; \
	  fi; \
	  if [ -n "$$SKILL_SCHEMA" ]; then \
	    echo "  skills ($$SKILL_SCHEMA)"; \
	    mdschema check "$(SKILL_SRC)/*/SKILL.md" --schema "$$SKILL_SCHEMA"; \
	  fi; \
	  AGENT_SCHEMA=""; \
	  if [ -f $(AGENT_SRC)/.mdschema ]; then \
	    AGENT_SCHEMA="$(AGENT_SRC)/.mdschema"; \
	  elif [ -f $(LIB_DIR)/rules/agent.mdschema ]; then \
	    AGENT_SCHEMA="$(LIB_DIR)/rules/agent.mdschema"; \
	  fi; \
	  if [ -n "$$AGENT_SCHEMA" ]; then \
	    echo "  agents ($$AGENT_SCHEMA)"; \
	    mdschema check "$(AGENT_SRC)/*.md" --schema "$$AGENT_SCHEMA"; \
	  fi; \
	fi

lint-docs:
	@if ! command -v mdschema >/dev/null 2>&1; then \
	  echo "  SKIP mdschema (not installed)"; \
	else \
	  for DOC in README INSTALL VERIFY; do \
	    SCHEMA=""; \
	    if [ -f ".$$DOC.mdschema" ]; then \
	      SCHEMA=".$$DOC.mdschema"; \
	    elif [ -f "$(LIB_DIR)/rules/$$DOC.mdschema" ]; then \
	      SCHEMA="$(LIB_DIR)/rules/$$DOC.mdschema"; \
	    fi; \
	    if [ -n "$$SCHEMA" ] && [ -f "$$DOC.md" ]; then \
	      echo "  $$DOC.md ($$SCHEMA)"; \
	      mdschema check "$$DOC.md" --schema "$$SCHEMA"; \
	    fi; \
	  done; \
	fi

lint-rules:
	@if ! command -v mdschema >/dev/null 2>&1; then \
	  echo "  SKIP mdschema (not installed)"; \
	elif [ -d rules ] && ls rules/*.md >/dev/null 2>&1; then \
	  SCHEMA=""; \
	  if [ -f rules/.mdschema ]; then \
	    SCHEMA="rules/.mdschema"; \
	  elif [ -f $(LIB_DIR)/rules/rules.mdschema ]; then \
	    SCHEMA="$(LIB_DIR)/rules/rules.mdschema"; \
	  fi; \
	  if [ -n "$$SCHEMA" ]; then \
	    echo "  rules ($$SCHEMA)"; \
	    mdschema check "rules/*.md" --schema "$$SCHEMA"; \
	  fi; \
	fi

lint-shell:
	@if find . -name '*.sh' -not -path '*/target/*' -not -path '*/lib/*' | grep -q .; then \
	  if command -v shellcheck >/dev/null 2>&1; then \
	    find . -name '*.sh' -not -path '*/target/*' -not -path '*/lib/*' -print0 | xargs -0 shellcheck -S warning; \
	  else \
	    echo "  SKIP shellcheck (not installed)"; \
	  fi; \
	fi
