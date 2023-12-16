#include <iostream>

#include <benchmark/benchmark.h>

#include "common.hpp"
#include "rust.hpp"

static void DoSetup(const benchmark::State &state) { cxxtantivy::init(); }

static void BM_AddSimple(benchmark::State &state) {
  // Setup
  cxxtantivy::drop_index();
  auto context = cxxtantivy::create_index();
  auto repeat_no = state.range(0);
  auto generated_data = dummy_data(repeat_no);

  // Measure
  for (auto _ : state) {
    for (const auto &doc : generated_data) {
      cxxtantivy::add(context, doc);
    }
  }
}
// Register the function as a benchmark
BENCHMARK(BM_AddSimple)
    ->RangeMultiplier(2)
    ->Range(1, 2 << 4)
    ->Unit(benchmark::kMillisecond)
    ->Setup(DoSetup);

// Run the benchmark
BENCHMARK_MAIN();
