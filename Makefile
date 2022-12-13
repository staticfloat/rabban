all: build

NATIVE_EXE := target/release/rabban

# We like to be fancy
COLS := $(shell tput cols 2>/dev/null)
ifeq ($(COLS),)
COLS := 80
endif

# Always strip debuginfo and symbols in the release build
# This helps to reduce footprint on constrained systems
export CARGO_PROFILE_RELEASE_STRIP=true

# If we're supposed to use `docker`, do so with the image in `$(1)`.
# Otherwise, just run natively.
ifeq ($(USE_DOCKER),true)
define docker_exec
docker run -e TERM=xterm \
           -e CARGO_HOME=/usr/local/cargo_home \
           -e CARGO_TERM_PROGRESS_WIDTH=$(COLS) \
           -e CARGO_TERM_PROGRESS_WHEN=always \
           -e CARGO_PROFILE_RELEASE_STRIP \
           -v $(shell pwd)/target/.docker_cargo_home:/usr/local/cargo_home \
           -v $(shell pwd):/app \
           -w /app \
           -i \
           $(1) \
           $(2)
endef

# To speed up interactive development with cargo, cache the cargo home in a subdir of `target`
target/.docker_cargo_home:
	mkdir -p $@
$(NATIVE_EXE): target/.docker_cargo_home
else
define docker_exec
$(2)
endef
endif

$(NATIVE_EXE): Cargo.toml Cargo.lock src/main.rs
	$(call docker_exec,rust,cargo build --color=always --release)
build: $(NATIVE_EXE)

# Tell `cross` to use a specific docker image for cross-compiling to Darwin:
target/aarch64-apple-darwin/release/rabban: export CROSS_TARGET_AARCH64_APPLE_DARWIN_IMAGE=joseluisq/rust-linux-darwin-builder
target/x86_64-apple-darwin/release/rabban:  export CROSS_TARGET_X86_64_APPLE_DARWIN_IMAGE=joseluisq/rust-linux-darwin-builder

# Because `cross-rs` mounts something over the cargo home, we recreate the configuration set in the `Cargo.toml` here:
# https://github.com/joseluisq/rust-linux-darwin-builder/blob/master/cargo/config.toml
target/aarch64-apple-darwin/release/rabban: export CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER=arm64e-apple-darwin21.4-clang
target/x86_64-apple-darwin/release/rabban:  export CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER=x86_64-apple-darwin21.4-clang

# Use `cross` to build for other architectures
target/%/release/rabban:
ifeq ($(shell which cross 2>/dev/null),)
	cargo install cross --git https://github.com/cross-rs/cross
endif
	cross build --target $* --release

# First, all our glibc Linux targets:
TARGET_TRIPLETS := aarch64-unknown-linux-gnu \
                   armv7-unknown-linux-gnueabihf \
                   powerpc64le-unknown-linux-gnu \
                   x86_64-unknown-linux-gnu \
                   i686-unknown-linux-gnu

# Next, musl Linux targets:
TARGET_TRIPLETS += aarch64-unknown-linux-musl \
                   armv7-unknown-linux-musleabihf \
                   x86_64-unknown-linux-musl \
                   i686-unknown-linux-musl

# Next, macOS targets:
# NOTE; these are disabled due to the fact that `cross-rs` does not have docker containers
# to build them; so we must build them manually.  SAD.
TARGET_TRIPLETS += aarch64-apple-darwin \
                   x86_64-apple-darwin

# Next, Windows targets:
TARGET_TRIPLETS += x86_64-pc-windows-gnu \
                   i686-pc-windows-gnu

$(foreach triplet,$(TARGET_TRIPLETS),$(eval multibuild: target/$(triplet)/release/rabban))

check:
	$(call docker_exec,rust,cargo fmt --all -- --check)

format:
	$(call docker_exec,rust,cargo fmt --all)

.PHONY: build test

test: build
	echo "No tests yet, maybe someday."

clean:
	rm -rf target
