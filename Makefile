.PHONY: build build-release setup-test test clippy check-lint clean prepare

prepare:
	rustup target add wasm32-unknown-unknown
	rustup component add clippy
	rustup component add rustfmt

build:
	cargo build
	cargo build --lib --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/debug/cowl_cli.wasm

build-release:
	cargo build --release
	cargo build --lib --target wasm32-unknown-unknown --release
	wasm-strip target/wasm32-unknown-unknown/release/cowl_cli.wasm

setup-test: build
	mkdir -p wasm

	$(eval LATEST_WASM_CEP18=$(shell curl -s https://api.github.com/repos/cowlnetwork/cep18/releases/latest | jq -r '.assets[] | select(.name=="cowl-cep18-wasm.tar.gz") | .browser_download_url'))
	$(eval LATEST_WASM_VESTING=$(shell curl -s https://api.github.com/repos/cowlnetwork/cowl-vesting/releases/latest | jq -r '.assets[] | select(.name=="cowl-vesting-wasm.tar.gz") | .browser_download_url'))
	$(eval LATEST_WASM_SWAP=$(shell curl -s https://api.github.com/repos/cowlnetwork/cowl-swap/releases/latest | jq -r '.assets[] | select(.name=="cowl-swap-wasm.tar.gz") | .browser_download_url'))

	@if [ -z "$(LATEST_WASM_CEP18)" ]; then \
		echo "Error: cowl-cep18 WASM URL is empty."; \
		exit 1; \
	fi

	@if [ -z "$(LATEST_WASM_VESTING)" ]; then \
		echo "Error: cowl-vesting WASM URL is empty."; \
		exit 1; \
	fi

	@if [ -z "$(LATEST_WASM_SWAP)" ]; then \
		echo "Error: cowl-swap WASM URL is empty."; \
		exit 1; \
	fi

	@echo "Downloading and extracting latest cowl-cep18 WASM..."
	curl -L $(LATEST_WASM_CEP18) -o cowl-cep18-wasm.tar.gz && \
	tar -xvzf cowl-cep18-wasm.tar.gz -C wasm && \
	rm cowl-cep18-wasm.tar.gz

	@echo "Downloading and extracting latest cowl-vesting WASM..."
	curl -L $(LATEST_WASM_VESTING) -o cowl-vesting-wasm.tar.gz && \
	tar -xvzf cowl-vesting-wasm.tar.gz -C wasm && \
	rm cowl-vesting-wasm.tar.gz

	@echo "Downloading and extracting latest cowl-swap WASM..."
	curl -L $(LATEST_WASM_SWAP) -o cowl-swap-wasm.tar.gz && \
	tar -xvzf cowl-swap-wasm.tar.gz -C wasm && \
	rm cowl-swap-wasm.tar.gz

test: setup-test test-dev

test-dev:
	cd tests && cargo test -- --test-threads=1 --nocapture

clippy:
	cargo clippy --bins
	cargo clippy --lib --target wasm32-unknown-unknown
	cd tests && cargo clippy --all-targets -- -D warnings

check-lint: clippy
	cargo fmt -- --check
	cd tests && cargo fmt -- --check

clean:
	cargo clean
