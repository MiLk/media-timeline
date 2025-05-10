CARGO := cargo

ifeq ($(shell uname),Linux)
CARGO = cross
endif

build/aarch64-linux:
	$(CARGO) build --release --target aarch64-unknown-linux-gnu
	@rm -rf dist
	@mkdir -p dist
	@tar --create -vf dist/build.tar -C target/aarch64-unknown-linux-gnu/release/ media-timeline
	@tar --append -vf dist/build.tar static templates
	@gzip --force dist/build.tar


setup/macos:
	brew tap messense/macos-cross-toolchains
	brew install aarch64-unknown-linux-gnu
	rustup target add aarch64-unknown-linux-gnu

setup/fedora:
	cargo install cross --git https://github.com/cross-rs/cross
