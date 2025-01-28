#!/bin/sh

target=x86_64-unknown-linux-gnu

if ! rustup target list --installed | grep $target >/dev/null; then
	rustup target add $target
fi

# see https://github.com/rust-lang/rust/issues/119500
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/x86_64-linux-gnu-gcc

cargo build --target=$target
