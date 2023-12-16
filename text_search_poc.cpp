#include <iostream>
#include <vector>

// TODO(gitbuda): Add benchmark (add|retrieve simple|complex, filtering,
// aggregations).
// TODO(gitbuda): init -> create_index + add the ability to inject schema.
// TODO(gitbuda): Move includes to cxxtantivy/rust|cxx.hpp.
// TODO(gitbuda): cxxtantivy::function but rust::Error -> unify.
#include "cxx.hpp"
#include "rust.hpp"

std::vector<text_search::DocumentInput> dummy_data(uint64_t size = 1) {
  std::vector<text_search::DocumentInput> data;
  for (uint64_t index = 0; index < size; ++index) {
    text_search::DocumentInput doc = {
        .data = text_search::Element{.gid = index,
                                     .txid = index,
                                     .deleted = false,
                                     .is_node = false,
                                     .props = "{key:value1}"}};
    data.push_back(doc);
  }
  return data;
}
std::ostream &operator<<(std::ostream &os,
                         const text_search::Element &element) {
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
    text_search::SearchInput search = {.query = "value1"};
    auto result = cxxtantivy::search(context, search);
    for (const auto &doc : result.docs) {
      std::cout << doc << std::endl;
    }
  } catch (const rust::Error &error) {
    std::cout << error.what() << std::endl;
  }
  return 0;
}
