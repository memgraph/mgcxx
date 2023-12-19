#include <iostream>
#include <memory>
#include <thread>

#include <benchmark/benchmark.h>

#include "common.hpp"
#include "rust.hpp"

// TODO(gitbuda): Add benchmarks:
//   * BENCH1: Search direct field vs JSON, FAST vs SLOW, String vs CxxString
//   * BENCH2: MATCH (n) RETURN count(n), n.deleted;
//   * BENCH3: search of a specific property value
//   * BENCH4: search of all properties

static std::atomic<uint64_t> cnt{0};
static bool global_init_done{false};

class MyFixture : public benchmark::Fixture {
public:
  void SetUp(const ::benchmark::State &state) {
    if (!global_init_done) {
      cxxtantivy::init();
      global_init_done = true;
    }
    auto index_name = fmt::format("index{}", cnt.load());
    context = std::make_unique<cxxtantivy::Context>(
        cxxtantivy::create_index(index_name));
  }
  void TearDown(const ::benchmark::State &state) {
    // TODO(gitbuda): Drop all generate index folders.
    // auto index_name = fmt::format("index{}", cnt.load());
    // cxxtantivy::drop_index(index_name);
    cnt.fetch_add(1);
  }
  std::unique_ptr<cxxtantivy::Context> context;
};

BENCHMARK_DEFINE_F(MyFixture, BM_AddSimple)(benchmark::State &state) {
  auto repeat_no = state.range(0);
  auto size = state.range(1);
  auto generated_data = dummy_data(repeat_no, size);

  for (auto _ : state) {
    for (const auto &doc : generated_data) {
      cxxtantivy::add1(*context, doc);
    }
  }
}

BENCHMARK_REGISTER_F(MyFixture, BM_AddSimple)
    ->RangeMultiplier(2)
    // { number of additions, document_size (number of JSON props)}
    ->Ranges({{1, 1 << 8}, {1, 1}})
    ->Unit(benchmark::kMillisecond);
// LEARN: Seems like it takes the similar time to add 1 and 128 prop JSON to the
// index.
BENCHMARK_REGISTER_F(MyFixture, BM_AddSimple)
    ->RangeMultiplier(2)
    // { number of additions, document_size (number of JSON props)}
    ->Ranges({{1, 1}, {1, 1 << 7}})
    ->Unit(benchmark::kMillisecond);

// Run the benchmark
BENCHMARK_MAIN();
