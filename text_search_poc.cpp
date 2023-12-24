#include <iostream>
#include <vector>

#include "common.hpp"

// TODO(gitbuda): Add benchmark (add|retrieve simple|complex, filtering,
// aggregations).
// TODO(gitbuda): init -> create_index + add the ability to inject schema.
// TODO(gitbuda): Move includes to cxxtantivy/rust|cxx.hpp (consider
// mgcxxtantivy because of ffi).
// TODO(gitbuda): cxxtantivy::function but rust::Error -> unify.

int main() {
  try {
    // init tantivy engine (actually logging setup, should be called once per
    // process, early)
    cxxtantivy::init();

    // init index
    cxxtantivy::drop_index("tantivy_index_poc");
    auto context = cxxtantivy::create_index1("tantivy_index_poc");

    // add data
    for (const auto &doc : dummy_data1(5, 5)) {
      std::cout << doc.metadata_and_data << std::endl;
      measure_time_diff<int>("add", [&]() {
        cxxtantivy::add1(context, doc);
        return 0;
      });
    }

    // search example
    // cxxtantivy::SearchInput search_input = {.search_query = "key1:value1"};
    cxxtantivy::SearchInput search_input = {.search_query =
                                                "data.key1:AWESOME"};
    auto result1 = measure_time_diff<cxxtantivy::SearchOutput>(
        "search1", [&]() { return cxxtantivy::search(context, search_input); });
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
    aggregation_query["count"]["value_count"]["field"] = "txid";
    cxxtantivy::SearchInput aggregate = {
        .search_query = "value12",
        .aggregation_query = aggregation_query.dump(),
    };
    auto aggregation_result =
        nlohmann::json::parse(cxxtantivy::aggregate(context, aggregate).data);
    std::cout << aggregation_result << std::endl;

  } catch (const rust::Error &error) {
    std::cout << error.what() << std::endl;
  }
  return 0;
}
