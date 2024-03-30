#include "gtest/gtest.h"

#include "test_util.hpp"

TEST(text_search_test_case, simple_test1) {
  try {
    auto index_name = "tantivy_index_simple_test1";
    mgcxx::text_search::drop_index(index_name);
    auto index_config =
        mgcxx::text_search::IndexConfig{.mappings = dummy_mappings1().dump()};
    auto context = mgcxx::text_search::create_index(index_name, index_config);

    for (const auto &doc : dummy_data1(5, 5)) {
      std::cout << doc.data << std::endl;
      measure_time_diff<int>("add", [&]() {
        mgcxx::text_search::add_document(context, doc, false);
        return 0;
      });
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
    for (const auto &doc : result1.docs) {
      std::cout << doc << std::endl;
    }
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
    std::cout << aggregation_result << std::endl;
  } catch (const ::rust::Error &error) {
    std::cout << error.what() << std::endl;
    FAIL();
  }
}

TEST(text_search_test_case, simple_test2) {
  try {
    auto index_name = "tantivy_index_simple_test2";
    mgcxx::text_search::drop_index(index_name);
    auto index_config =
        mgcxx::text_search::IndexConfig{.mappings = dummy_mappings2().dump()};
    auto context = mgcxx::text_search::create_index(index_name, index_config);

    for (const auto &doc : dummy_data2(2, 1)) {
      measure_time_diff<int>("add", [&]() {
        mgcxx::text_search::add_document(context, doc, false);
        return 0;
      });
    }

    mgcxx::text_search::SearchInput search_input = {.search_fields = {"gid"},
                                                     .search_query =
                                                         fmt::format("{}", 0),
                                                     .return_fields = {"data"}};
    auto result = mgcxx::text_search::search(context, search_input);
    ASSERT_EQ(result.docs.size(), 1);
    for (const auto &doc : result.docs) {
      std::cout << doc << std::endl;
    }
  } catch (const ::rust::Error &error) {
    std::cout << error.what() << std::endl;
    FAIL();
  }
}

TEST(text_search_test_case, mappings) {
  try {
    auto index_name = "tantivy_index_mappings";
    mgcxx::text_search::drop_index(index_name);
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
  } catch (const ::rust::Error &error) {
    std::cout << error.what() << std::endl;
    EXPECT_STREQ(error.what(), "The field does not exist: 'data' inside "
                               "\"tantivy_index_mappings\" text search index");
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
