#!/bin/bash -e
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
export RUST_LOG=warn
export TMPDIR="/tmp" # To make std::filesystem::temp_directory_path returns path under /tmp
rm -rf /tmp/text_search_index_*

cd "$SCRIPT_DIR"
# TODO(gitbuda): Add clang-format call here.
cargo fmt

cd "$SCRIPT_DIR/../build"
if [ "$1" == "--full" ]; then
  rm -rf ./* && rm -rf .cache
else
  rm -rf index*
fi
cmake ..
make -j8
cd "$SCRIPT_DIR/../build/text_search"
./test_unit
./test_bench
# ./test_bench --benchmark_filter="MyFixture2/BM_BenchLookup"
./test_stress

rm -rf /tmp/text_search_index_*
