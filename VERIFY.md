# Verify

> **For AI agents**: Complete this checklist after installation.

## Health check

Confirm binaries are built and available:

```bash
make build && make check
```

Each binary should respond to `--version`:

```bash
bin/strip-front --version
bin/install-agents --version
bin/install-skills --version
bin/validate-module --version
bin/yaml --version
```

## Structure validation

Run the convention test suite against the module:

```bash
bin/validate-module .
```

Check key files exist:

```bash
test -f module.yaml && echo "ok module.yaml"
test -f defaults.yaml && echo "ok defaults.yaml"
test -f Cargo.toml && echo "ok Cargo.toml"
test -f Makefile && echo "ok Makefile"
```

## Functionality tests

Run the full test suite:

```bash
make test
```

Lint check:

```bash
make lint
```

## Success criteria

- [ ] `make check` confirms all binaries present
- [ ] All binaries respond to `--version`
- [ ] `validate-module .` passes skills and structure checks
- [ ] `make test` passes
- [ ] `make lint` reports no warnings
