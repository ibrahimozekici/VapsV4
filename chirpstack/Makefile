.PHONY: dist

PKG_VERSION := $(shell cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')
DATABASE ?= postgres
ifeq ($(DATABASE),postgres)
	VARIANT_FLAGS ?=
else
	VARIANT_FLAGS ?= --variant="$(DATABASE)"
endif

debug-amd64:
	cross build --target x86_64-unknown-linux-musl --no-default-features --features="$(DATABASE)"

release-amd64:
	cross build --release --no-default-features --features="$(DATABASE)" --platform linux/amd64 
	

dist:
	# Build for x86_64-linux-musl only
	cross build --target x86_64-unknown-linux-gnu --release

	# Create the .deb and .rpm packages
	cargo deb --target x86_64-unknown-linux-musl --no-build --no-strip
	cargo generate-rpm --target x86_64-unknown-linux-musl --target-dir ../target

	# Package the release into a tarball
	mkdir -p ../dist
	cp ../target/x86_64-unknown-linux-musl/debian/*.deb ../dist
	cp ../target/x86_64-unknown-linux-musl/generate-rpm/*.rpm ../dist
	tar -czvf ../dist/chirpstack_amd64.tar.gz -C ../target/x86_64-unknown-linux-musl/release chirpstack

test:
	cargo fmt --check
	cargo clippy --no-deps --no-default-features --features="$(DATABASE)"
	TZ=UTC cargo test --no-default-features --features="$(DATABASE)"

test-all:
	cargo fmt --check
	cargo clippy --no-deps --no-default-features --features="$(DATABASE)"
	TZ=UTC cargo test --no-default-features --features="$(DATABASE),test-all-integrations"
