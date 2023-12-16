#include <iostream>
#include <vector>

#include <nlohmann/json.hpp>

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
