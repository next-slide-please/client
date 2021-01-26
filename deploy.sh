#!/bin/bash

set -e

function run {
	ssh -A nsp@nsp bash -c "'$1'"
}

rm -rf target/release/bundle/osx
cargo bundle --release
(cd target/release/bundle/osx && zip -r ../../../../macOS.zip NextSlidePlease.app)
unzip -l macOS.zip
scp ./macOS.zip nsp@nsp:~/downloads/macOS.zip
rm macOS.zip

