#!/bin/bash -e
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/build"
export RUST_LOG=warn

cmake
make -j8
./text_search_unit
./text_search_bench # --benchmark_filter=MyFixture1/BM_AddSimpleEagerCommit/1/1
./text_search_stress
