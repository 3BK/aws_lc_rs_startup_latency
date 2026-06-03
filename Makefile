# Makefile — awslc-boot-bench
SHELL  := /bin/bash
TARGET := x86_64-unknown-linux-musl
BIN    := target/$(TARGET)/release/awslc-boot-bench

.PHONY: all release debug clean run ldd-check

all: release

## ── Build targets ───────────────────────────────────────────────────
release:
	cargo build --release --target $(TARGET)
	@echo "Binary → $(BIN)"

debug:
	cargo build --target $(TARGET)

## ── Convenience ─────────────────────────────────────────────────────
run: release
	@echo ""
	$(BIN)

ldd-check: release
	@echo "--- ldd output (expect 'not a dynamic executable') ---"
	@ldd $(BIN) || true
	@echo "--- file output ---"
	@file $(BIN)

clean:
	cargo clean

## ── Repeat benchmark (10 runs, extract Phase 2 seed cost) ──────────
bench: release
	@echo "Phase 2 seed-cost over 10 runs:"
	@for i in $$(seq 1 10); do \
		$(BIN) 2>/dev/null | grep "Phase 2" | awk '{print $$NF, $$7}'; \
	done
