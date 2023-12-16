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
    cxxtantivy::init();
    cxxtantivy::drop_index();
    auto context = cxxtantivy::create_index();
    for (const auto &doc : dummy_data(5)) {
      cxxtantivy::add(context, doc);
    }
    cxxtantivy::SearchInput search = {.query = "value1"};
    auto result = cxxtantivy::search(context, search);
    for (const auto &doc : result.docs) {
      std::cout << doc << std::endl;
    }
  } catch (const rust::Error &error) {
    std::cout << error.what() << std::endl;
  }
  return 0;
}
