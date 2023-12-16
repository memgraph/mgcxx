#include <iostream>
#include <vector>

#include <nlohmann/json.hpp>

// TODO(gitbuda): Add benchmark (add|retrieve simple|complex, filtering,
// aggregations).
// TODO(gitbuda): init -> create_index + add the ability to inject schema.
// TODO(gitbuda): Move includes to cxxtantivy/rust|cxx.hpp (consider
// mgcxxtantivy because of ffi).
// TODO(gitbuda): cxxtantivy::function but rust::Error -> unify.
#include "cxx.hpp"
#include "rust.hpp"

std::vector<cxxtantivy::DocumentInput> dummy_data(uint64_t size = 1) {
  std::vector<cxxtantivy::DocumentInput> data;
  for (uint64_t index = 0; index < size; ++index) {
    nlohmann::json props = {
        {"key", "value1"},
    };
    cxxtantivy::DocumentInput doc = {
        .data = cxxtantivy::Element{.gid = index,
                                    .txid = index,
                                    .deleted = false,
                                    .is_node = false,
                                    .props = props.dump()}};
    data.push_back(doc);
  }
  return data;
}
std::ostream &operator<<(std::ostream &os, const cxxtantivy::Element &element) {
  os << "GID: " << element.gid << "; TXID: " << element.txid
     << "; DELETED: " << element.deleted << "; IS_NODE: " << element.is_node
     << "; PROPS: " << element.props;
  return os;
}

int main() {
  try {
    cxxtantivy::drop_index();
    auto context = cxxtantivy::init();
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
