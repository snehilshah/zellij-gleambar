WASM := target/wasm32-wasip1/release/zellij-gleambar.wasm
DEST := $(HOME)/.local/share/zellij/plugins/gleambar.wasm

install: $(WASM)
	install -Dm644 $(WASM) $(DEST)

$(WASM): src/*.rs Cargo.toml
	cargo build --release

.PHONY: install
