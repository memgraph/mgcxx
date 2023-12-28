#include <iostream>
#include <memory>
#include <thread>

#include <benchmark/benchmark.h>

#include "common.hpp"

// cnt is here to ensure unique directory names because gbench is running
// benchmarks in parallel.
static std::atomic<uint64_t> cnt{0};
static bool global_init_done{false};

class MyFixture1 : public benchmark::Fixture {
public:
  void SetUp(const ::benchmark::State &state) {
    if (!global_init_done) {
      memcxx::text_search::init("todo");
      global_init_done = true;
    }
    index_path = create_temporary_directory("text_search_index_",
                                            "_" + std::to_string(cnt.load()))
                     .string();
    auto index_config =
        memcxx::text_search::IndexConfig{.mappings = dummy_mappings1().dump()};
    context = std::make_unique<memcxx::text_search::Context>(
        memcxx::text_search::create_index(index_path, index_config));
  }
  void TearDown(const ::benchmark::State &state) {
    // NOTE: Dropping index here produces errors probably because of the
    // concurrent access. Folder delete under the test.sh script.
    cnt.fetch_add(1);
  }
  std::unique_ptr<memcxx::text_search::Context> context;
  std::string index_path;
};

class MyFixture2 : public benchmark::Fixture {
public:
  void SetUp(const ::benchmark::State &state) {
    if (!global_init_done) {
      memcxx::text_search::init("todo");
      global_init_done = true;
    }
    index_path = create_temporary_directory("text_search_index_",
                                            "_" + std::to_string(cnt.load()))
                     .string();
    auto index_config =
        memcxx::text_search::IndexConfig{.mappings = dummy_mappings2().dump()};
    context = std::make_unique<memcxx::text_search::Context>(
        memcxx::text_search::create_index(index_path, index_config));
  }
  void TearDown(const ::benchmark::State &state) {
    // NOTE: Dropping index here produces errors probably because of the
    // concurrent access. Folder delete under the test.sh script.
    cnt.fetch_add(1);
  }
  std::unique_ptr<memcxx::text_search::Context> context;
  std::string index_path;
};

BENCHMARK_DEFINE_F(MyFixture1, BM_AddSimpleEagerCommit)
(benchmark::State &state) {
  auto repeat_no = state.range(0);
  auto size = state.range(1);
  auto generated_data = dummy_data1(repeat_no, size);

  for (auto _ : state) {
    for (const auto &doc : generated_data) {
      memcxx::text_search::add(*context, doc, false);
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
      memcxx::text_search::add(*context, doc, true);
    }
  }
  memcxx::text_search::commit(*context);
}

BENCHMARK_DEFINE_F(MyFixture1, BM_BenchLookup)(benchmark::State &state) {
  auto repeat_no = state.range(0);
  auto generated_data = dummy_data1(repeat_no, 5);
  for (const auto &doc : generated_data) {
    memcxx::text_search::add(*context, doc, true);
  }
  memcxx::text_search::commit(*context);

  memcxx::text_search::SearchInput search_input = {
      .search_query = fmt::format("metadata.gid:{}", 0)};
  for (auto _ : state) {
    auto result = memcxx::text_search::search(*context, search_input);
    if (result.docs.size() < 1) {
      std::exit(1);
    }
  }
}

BENCHMARK_DEFINE_F(MyFixture2, BM_BenchLookup)(benchmark::State &state) {
  auto repeat_no = state.range(0);
  auto generated_data = dummy_data2(repeat_no, 5);
  for (const auto &doc : generated_data) {
    memcxx::text_search::add(*context, doc, true);
  }
  memcxx::text_search::commit(*context);

  memcxx::text_search::SearchInput search_input = {.search_query =
                                                       fmt::format("{}", 0)};
  for (auto _ : state) {
    auto result = memcxx::text_search::find(*context, search_input);
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
