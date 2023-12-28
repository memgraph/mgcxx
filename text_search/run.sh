#!/bin/bash -e
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
export RUST_LOG=warn
export TMPDIR="/tmp" # To make std::filesystem::temp_directory_path returns path under /tmp
rm -rf /tmp/text_search_index_*

cd "$SCRIPT_DIR"
# TODO(gitbuda): Add clang-format call here.

cd "$SCRIPT_DIR"
cargo fmt

cd "$SCRIPT_DIR/../build"
rm -rf index*
cmake
make -j8
cd "$SCRIPT_DIR/../build/text_search"
./unit
./bench
# ./bench --benchmark_filter="MyFixture2/BM_BenchLookup"
./stress

rm -rf /tmp/text_search_index_*
