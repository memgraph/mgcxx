#include <chrono>
#include <iostream>
#include <vector>

#include <fmt/format.h>
#include <nlohmann/json.hpp>

#include "cxx.hpp"
#include "rust.hpp"

std::vector<cxxtantivy::DocumentInput> dummy_data(uint64_t docs_no = 1,
                                                  uint64_t props_no = 1) {
  std::vector<cxxtantivy::DocumentInput> data;
  for (uint64_t doc_index = 0; doc_index < docs_no; ++doc_index) {
    nlohmann::json props = {};
    for (uint64_t prop_index = 0; prop_index < props_no; ++prop_index) {
      props[fmt::format("key{}", prop_index)] =
          fmt::format("value{}", prop_index);
    }
    cxxtantivy::DocumentInput doc = {
        .data = cxxtantivy::Element{.gid = doc_index,
                                    .txid = doc_index,
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

auto now() { return std::chrono::steady_clock::now(); }
template <typename T>
auto print_time_diff(std::string_view prefix, T start, T end) {
  std::cout << prefix << " dt = "
            << std::chrono::duration_cast<std::chrono::microseconds>(end -
                                                                     start)
                   .count()
            << "[µs]" << std::endl;
  // << "[ms]" << std::endl;
}
template <typename T>
auto measure_time_diff(std::string_view prefix, std::function<T()> f) {
  auto start = now();
  T result = f();
  auto end = now();
  print_time_diff(prefix, start, end);
  return result;
}
