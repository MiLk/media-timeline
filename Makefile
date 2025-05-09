build/aarch64-linux:
	cargo build --release --target aarch64-unknown-linux-gnu
	@rm -rf dist
	@mkdir -p dist
	@tar --create -vf dist/build.tar -C target/aarch64-unknown-linux-gnu/release/ media-timeline
	@tar --append -vf dist/build.tar static templates
	@gzip --force dist/build.tar


setup/macos:
	brew tap messense/macos-cross-toolchains
	brew install aarch64-unknown-linux-gnu
