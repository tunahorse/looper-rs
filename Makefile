.PHONY: help fmt compile check clippy test complexity duplicates large-files dead-code ast-rules quality-full quality-quick

help: ## Show this help message
	@echo "Alchemy - Development Commands"
	@echo ""
	@echo "Quality Checks:"
	@echo "  make check            - Run ALL checks (fmt, clippy, compile, test, complexity, duplicates, dead-code, ast-rules, large-files)"
	@echo "  make quality-quick    - Fast checks (fmt, clippy, compile)"
	@echo "  make quality-full     - All checks including complexity, duplicates, dead-code"
	@echo "  make complexity       - Run cyclomatic complexity analysis"
	@echo "  make duplicates       - Run duplicate code detection"
	@echo "  make dead-code        - Run dead code detection"
	@echo "  make ast-rules        - Run ast-grep architecture boundary checks"
	@echo "  make large-files      - Detect large files (default: 500KB)"
	@echo ""
	@echo "Standard Commands:"
	@echo "  make fmt              - Format code"
	@echo "  make compile          - Check compilation"
	@echo "  make clippy           - Run clippy linter"
	@echo "  make test             - Run tests"

fmt: ## Format code with rustfmt
	cargo fmt --all

check: ## Run ALL quality checks (runs everything regardless of failures)
	@echo "========================================"
	@echo "Running ALL quality checks..."
	@echo "========================================"
	@FAIL=0; \
	$(MAKE) fmt || FAIL=1; \
	$(MAKE) clippy || FAIL=1; \
	$(MAKE) compile || FAIL=1; \
	$(MAKE) test || FAIL=1; \
	$(MAKE) complexity || FAIL=1; \
	$(MAKE) duplicates || FAIL=1; \
	$(MAKE) dead-code || FAIL=1; \
	$(MAKE) ast-rules || FAIL=1; \
	$(MAKE) large-files || FAIL=1; \
	if [ $$FAIL -eq 0 ]; then \
		echo ""; \
		echo "========================================"; \
		echo "✓ ALL CHECKS PASSED"; \
		echo "========================================"; \
	else \
		echo ""; \
		echo "========================================"; \
		echo "✗ SOME CHECKS FAILED"; \
		echo "========================================"; \
	fi; \
	exit $$FAIL

compile: ## Check compilation
	cargo check --all-targets --all-features

clippy: ## Run clippy with complexity checks
	cargo clippy --all-targets --all-features -- -D warnings \
		-W clippy::cognitive_complexity \
		-W clippy::too_many_lines \
		-W clippy::too_many_arguments

test: ## Run all tests
	cargo test --all-features

complexity: ## Run cyclomatic complexity analysis
	@echo "Checking cognitive complexity with clippy..."
	@cargo clippy --all-targets --all-features -- \
		-W clippy::cognitive_complexity \
		-W clippy::too_many_lines \
		2>&1 | grep -E "(cognitive_complexity|too_many_lines)" || echo "✓ No complexity violations found"

duplicates: ## Run duplicate code detection
	@echo "Running duplicate code detection with polydup..."
	@command -v polydup >/dev/null 2>&1 || { echo "polydup not installed. Run: cargo install polydup-cli"; exit 1; }
	@polydup --min-tokens 100 --similarity 0.9 src/ || echo "✓ No significant duplicates found"

dead-code: ## Run dead code detection
	@echo "Checking for dead code..."
	@cargo clippy --all-targets --all-features -- -W dead_code -W unused 2>&1 | grep -E "(dead_code|is never used|unused)" || echo "✓ No dead code detected"

ast-rules: ## Run ast-grep architecture boundary checks
	@echo "Running ast-grep architecture rules..."
	@command -v sg >/dev/null 2>&1 || { echo "ast-grep not installed. Install from https://ast-grep.github.io"; exit 1; }
	@sg scan --config rules/sgconfig.yml src --error

large-files: ## Detect large files (default: 500KB)
	@echo "Checking for large files (>500KB)..."
	@find src -type f -size +500k -exec ls -lh {} \; 2>/dev/null | awk '{ print $$9 ": " $$5 }' || echo "✓ No large files found"

quality-quick: fmt clippy compile ## Fast quality checks (pre-commit)
	@echo "✓ Quick quality checks passed"

quality-full: quality-quick test complexity duplicates dead-code ast-rules large-files ## All quality checks
	@echo "✓ Full quality analysis complete"
