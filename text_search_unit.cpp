#include "gtest/gtest.h"

#include "common.hpp"

TEST(text_search_test_case, simple_test) {
  try {
    // init index
    cxxtantivy::drop_index("tantivy_index_simple_test");
    auto context = cxxtantivy::create_index1("tantivy_index_simple_test");

    // add data
    for (const auto &doc : dummy_data1(5, 5)) {
      std::cout << doc.metadata_and_data << std::endl;
      measure_time_diff<int>("add", [&]() {
        cxxtantivy::add1(context, doc);
        return 0;
      });
    }

    // search example
    cxxtantivy::SearchInput search_input = {.search_query =
                                                "data.key1:AWESOME"};
    auto result1 = measure_time_diff<cxxtantivy::SearchOutput>(
        "search1", [&]() { return cxxtantivy::search(context, search_input); });
    ASSERT_EQ(result1.docs.size(), 5);
    for (const auto &doc : result1.docs) {
      std::cout << doc << std::endl;
    }

    for (uint64_t i = 0; i < 10; ++i) {
      auto result = measure_time_diff<cxxtantivy::SearchOutput>(
          fmt::format("search{}", i),
          [&]() { return cxxtantivy::search(context, search_input); });
    }

    // aggregation example
    nlohmann::json aggregation_query = {};
    aggregation_query["count"]["value_count"]["field"] = "metadata.txid";
    cxxtantivy::SearchInput aggregate = {
        .search_query = "data.key1:AWESOME",
        .aggregation_query = aggregation_query.dump(),
    };
    auto aggregation_result =
        nlohmann::json::parse(cxxtantivy::aggregate(context, aggregate).data);
    EXPECT_NEAR(aggregation_result["count"]["value"], 5, 1e-6);
    std::cout << aggregation_result << std::endl;

  } catch (const rust::Error &error) {
    std::cout << error.what() << std::endl;
    FAIL();
  }
}

// TODO(gitbuda): Make a gtest main lib and link agains other test binaries.
int main(int argc, char *argv[]) {
  // init tantivy engine (actually logging setup, should be called once per
  // process, early)
  cxxtantivy::init();
  ::testing::InitGoogleTest(&argc, argv);
  return RUN_ALL_TESTS();
}
