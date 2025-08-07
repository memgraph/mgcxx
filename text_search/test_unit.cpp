#include "gtest/gtest.h"
#include <mutex>
#include <thread>

#include "test_util.hpp"

TEST(text_search_test_case, simple_test1) {
  try {
    auto index_name = "tantivy_index_simple_test1";
    auto index_config =
        mgcxx::text_search::IndexConfig{.mappings = dummy_mappings1().dump()};
    auto context = mgcxx::text_search::create_index(index_name, index_config);

    for (const auto &doc : dummy_data1(5, 5)) {
      measure_time_diff<int>("add", [&]() {
        mgcxx::text_search::add_document(context, doc, false);
        return 0;
      });
    }

    // wait for delay to ensure all documents are indexed
    while (mgcxx::text_search::get_num_docs(context) < 5) {
      std::this_thread::sleep_for(std::chrono::milliseconds(100));
    }
    mgcxx::text_search::SearchInput search_input = {
        .search_fields = {"metadata"},
        .search_query = "data.key1:AWESOME",
        .return_fields = {"data"}};
    auto result1 =
        measure_time_diff<mgcxx::text_search::SearchOutput>("search1", [&]() {
          return mgcxx::text_search::search(context, search_input);
        });
    ASSERT_EQ(result1.docs.size(), 5);
    for (uint64_t i = 0; i < 10; ++i) {
      auto result = measure_time_diff<mgcxx::text_search::SearchOutput>(
          fmt::format("search{}", i),
          [&]() { return mgcxx::text_search::search(context, search_input); });
    }

    nlohmann::json aggregation_query = {};
    aggregation_query["count"]["value_count"]["field"] = "metadata.txid";
    mgcxx::text_search::SearchInput aggregate_input = {
        .search_fields = {"data"},
        .search_query = "data.key1:AWESOME",
        .aggregation_query = aggregation_query.dump(),
    };
    auto aggregation_result = nlohmann::json::parse(
        mgcxx::text_search::aggregate(context, aggregate_input).data);
    EXPECT_NEAR(aggregation_result["count"]["value"], 5, 1e-6);
    mgcxx::text_search::drop_index(std::move(context));
  } catch (const ::rust::Error &error) {
    FAIL() << "Test failed: " << error.what();  
  }
}

TEST(text_search_test_case, simple_test2) {
  try {
    auto index_name = "tantivy_index_simple_test2";
    auto index_config =
        mgcxx::text_search::IndexConfig{.mappings = dummy_mappings2().dump()};
    auto context = mgcxx::text_search::create_index(index_name, index_config);

    for (const auto &doc : dummy_data2(2, 1)) {
      measure_time_diff<int>("add", [&]() {
        mgcxx::text_search::add_document(context, doc, false);
        return 0;
      });
    }
    // wait for delay to ensure all documents are indexed
    while (mgcxx::text_search::get_num_docs(context) < 2) {
      std::this_thread::sleep_for(std::chrono::milliseconds(100));
    }

    mgcxx::text_search::SearchInput search_input = {.search_fields = {"gid"},
                                                     .search_query =
                                                         fmt::format("{}", 0),
                                                     .return_fields = {"data"}};
    auto result = mgcxx::text_search::search(context, search_input);
    ASSERT_EQ(result.docs.size(), 1);
    mgcxx::text_search::drop_index(std::move(context));
  } catch (const ::rust::Error &error) {
    FAIL() << "Test failed: " << error.what();
  }
}

TEST(text_search_test_case, mappings) {
  try {
    constexpr auto index_name = "tantivy_index_mappings";
    nlohmann::json mappings = {};
    mappings["properties"] = {};
    mappings["properties"]["prop1"] = {
        {"type", "u64"}, {"fast", true}, {"indexed", true}};
    mappings["properties"]["prop2"] = {
        {"type", "text"}, {"stored", true}, {"text", true}, {"fast", true}};
    mappings["properties"]["prop3"] = {
        {"type", "json"}, {"stored", true}, {"text", true}, {"fast", true}};
    mappings["properties"]["prop4"] = {
        {"type", "bool"}, {"stored", true}, {"text", true}, {"fast", true}};
    auto context = mgcxx::text_search::create_index(
        index_name,
        mgcxx::text_search::IndexConfig{.mappings = mappings.dump()});

    // NOTE: This test just verifies the code can be called, add deeper test
    // when improving extract_schema.
    // TODO(gitbuda): Implement full range of extract_schema options.
    mgcxx::text_search::SearchInput search_input = {
        .search_fields = {"prop1"},
        .search_query = "bla",
        .return_fields = {"data"}};
    mgcxx::text_search::search(context, search_input);
    mgcxx::text_search::drop_index(std::move(context));
  } catch (const ::rust::Error &error) {
    std::cout << error.what() << std::endl;
    EXPECT_STREQ(error.what(), "The field does not exist: 'data' inside "
                               "\"tantivy_index_mappings\" text search index");
  }
}

TEST(text_search_test_case, drop_index_stress_test) {
  try {
    constexpr auto index_name = "tantivy_index_stress_drop";
    
    nlohmann::json mappings = {};
    mappings["properties"] = {};
    mappings["properties"]["data"] = {
        {"type", "text"}, {"stored", true}, {"text", true}, {"fast", true}};
    
    auto context = mgcxx::text_search::create_index(
        index_name,
        mgcxx::text_search::IndexConfig{.mappings = mappings.dump()});

    // Use multiple threads to create maximum merging pressure
    constexpr auto thread_count = 10;
    constexpr auto docs_per_thread = 50;
    {
      std::vector<std::jthread> threads;
      std::mutex mutex;
      threads.reserve(thread_count);
      
      for (auto t = 0; t < thread_count; t++) {
        threads.emplace_back([&context, &mutex, t, docs_per_thread]() {
          for (auto i = 0; i < docs_per_thread; i++) {
            nlohmann::json doc_data = {};
            doc_data["data"] = "Thread " + std::to_string(t) + " document " + std::to_string(i) + 
                              " with substantial content to create larger segments that require merging " +
                              "when multiple threads are adding documents simultaneously creating pressure";
            
            mgcxx::text_search::DocumentInput doc = {
              .data = doc_data.dump()
            };
            
            std::lock_guard lock(mutex);
            // Commit every few documents to create many small segments
            bool skip_commit = (i % 9 != 0);
            mgcxx::text_search::add_document(context, doc, skip_commit);
          }
        });
      }
    }
    // Final commit to ensure all documents are processed
    mgcxx::text_search::commit(context); 
    mgcxx::text_search::drop_index(std::move(context));
  } catch (const ::rust::Error &error) {
    FAIL() << "Stress drop test failed: " << error.what();
  }
}

// TODO(gitbuda): Make a gtest main lib and link agains other test binaries.
int main(int argc, char *argv[]) {
  // init tantivy engine (actually logging setup, should be called once per
  // process, early)
  mgcxx::text_search::init("todo");
  ::testing::InitGoogleTest(&argc, argv);
  return RUN_ALL_TESTS();
}
