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

class MyFixture1 : public benchmark::Fixture {
public:
  void SetUp(const ::benchmark::State &state) {
    if (!global_init_done) {
      cxxtantivy::init();
      global_init_done = true;
    }
    auto index_name = fmt::format("index{}", cnt.load());
    context = std::make_unique<cxxtantivy::Context>(
        cxxtantivy::create_index1(index_name));
  }
  void TearDown(const ::benchmark::State &state) {
    // TODO(gitbuda): Drop all generate index folders.
    // auto index_name = fmt::format("index{}", cnt.load());
    // cxxtantivy::drop_index(index_name);
    cnt.fetch_add(1);
  }
  std::unique_ptr<cxxtantivy::Context> context;
};

class MyFixture2 : public benchmark::Fixture {
public:
  void SetUp(const ::benchmark::State &state) {
    if (!global_init_done) {
      cxxtantivy::init();
      global_init_done = true;
    }
    auto index_name = fmt::format("index{}", cnt.load());
    context = std::make_unique<cxxtantivy::Context>(
        cxxtantivy::create_index2(index_name));
  }
  void TearDown(const ::benchmark::State &state) {
    // TODO(gitbuda): Drop all generate index folders.
    // auto index_name = fmt::format("index{}", cnt.load());
    // cxxtantivy::drop_index(index_name);
    cnt.fetch_add(1);
  }
  std::unique_ptr<cxxtantivy::Context> context;
};

BENCHMARK_DEFINE_F(MyFixture1, BM_AddSimpleEagerCommit)
(benchmark::State &state) {
  auto repeat_no = state.range(0);
  auto size = state.range(1);
  auto generated_data = dummy_data1(repeat_no, size);

  for (auto _ : state) {
    for (const auto &doc : generated_data) {
      cxxtantivy::add1(*context, doc, false);
    }
  }
}

BENCHMARK_DEFINE_F(MyFixture1, BM_AddSimpleLazyCommit)
(benchmark::State &state) {
  auto repeat_no = state.range(0);
  auto size = state.range(1);
  auto generated_data = dummy_data1(repeat_no, size);

  for (auto _ : state) {
    for (const auto &doc : generated_data) {
      cxxtantivy::add1(*context, doc, true);
    }
  }
  cxxtantivy::commit(*context);
}

BENCHMARK_DEFINE_F(MyFixture1, BM_BenchLookup)(benchmark::State &state) {
  auto repeat_no = state.range(0);
  auto generated_data = dummy_data1(repeat_no, 5);
  for (const auto &doc : generated_data) {
    cxxtantivy::add1(*context, doc, true);
  }
  cxxtantivy::commit(*context);

  cxxtantivy::SearchInput search_input = {
      .search_query = fmt::format("metadata.gid:{}", 0)};
  for (auto _ : state) {
    auto result = cxxtantivy::search(*context, search_input);
    if (result.docs.size() < 1) {
      std::exit(1);
    }
  }
}

BENCHMARK_DEFINE_F(MyFixture2, BM_BenchLookup)(benchmark::State &state) {
  auto repeat_no = state.range(0);
  auto generated_data = dummy_data2(repeat_no, 5);
  for (const auto &doc : generated_data) {
    cxxtantivy::add2(*context, doc, true);
  }
  cxxtantivy::commit(*context);

  cxxtantivy::SearchInput search_input = {.search_query = fmt::format("{}", 0)};
  for (auto _ : state) {
    auto result = cxxtantivy::find(*context, search_input);
    if (result.docs.size() < 1) {
      std::exit(1);
    }
  }
}

// LEARNING: Seems like it takes the similar time to add 1 and 128 prop JSON to
// the index.
BENCHMARK_REGISTER_F(MyFixture1, BM_AddSimpleEagerCommit)
    ->RangeMultiplier(2)
    // { number of additions, document_size (number of JSON props)}
    ->Ranges({{1, 1 << 2}, {1, 1}})
    ->Unit(benchmark::kMillisecond);
BENCHMARK_REGISTER_F(MyFixture1, BM_AddSimpleEagerCommit)
    ->RangeMultiplier(2)
    // { number of additions, document_size (number of JSON props)}
    ->Ranges({{1, 1}, {1, 1 << 7}})
    ->Unit(benchmark::kMillisecond);

// LEARNING: Lazy commit is much faster ON_DISK, as expected.
BENCHMARK_REGISTER_F(MyFixture1, BM_AddSimpleLazyCommit)
    ->RangeMultiplier(2)
    // { number of additions, document_size (number of JSON props)}
    ->Ranges({{1, 1 << 16}, {1, 1}})
    ->Unit(benchmark::kMillisecond);

// Learn direct field lookup vs JSON/TEXT lookup diff
//   -> seems like u64 INDEXED field is slightly faster
//   -> mappings FTW
BENCHMARK_REGISTER_F(MyFixture1, BM_BenchLookup)
    ->RangeMultiplier(2)
    // { number of additions, document_size (number of JSON props)}
    ->Ranges({{1, 1 << 16}})
    ->Unit(benchmark::kMillisecond);
BENCHMARK_REGISTER_F(MyFixture2, BM_BenchLookup)
    ->RangeMultiplier(2)
    // { number of additions, document_size (number of JSON props)}
    ->Ranges({{1, 1 << 16}})
    ->Unit(benchmark::kMillisecond);

// Run the benchmark
BENCHMARK_MAIN();
