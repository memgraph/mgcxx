#!/bin/bash -e
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
export RUST_LOG=warn

cd "$SCRIPT_DIR"
# TODO(gitbuda): Add clang-format call here.

cd "$SCRIPT_DIR/rust"
cargo fmt

cd "$SCRIPT_DIR/build"
rm -rf index*
cmake
make -j8
./text_search_unit
./text_search_bench # --benchmark_filter=MyFixture1/BM_AddSimpleEagerCommit/1/1
./text_search_stress
