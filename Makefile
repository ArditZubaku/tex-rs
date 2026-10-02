BINARY := ./tex-bin
FILE ?= ../../golang/tex/main.go

.PHONY: all run build clean

# all: run

run: build
	@$(BINARY) $(FILE)

build:
	@RUSTFLAGS="-C target-cpu=native" cargo build --release --quiet
	@cp target/release/tex $(BINARY)

clean:
	@cargo clean
	@rm -f $(BINARY)

lint:
	cargo clippy -- -W clippy::pedantic

