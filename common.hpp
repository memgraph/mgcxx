#include <chrono>
#include <iostream>
#include <vector>

#include <fmt/format.h>
#include <nlohmann/json.hpp>

#include "cxx.hpp"
#include "rust.hpp"

std::vector<cxxtantivy::DocumentInput1> dummy_data(uint64_t docs_no = 1,
                                                   uint64_t props_no = 1) {
  std::vector<cxxtantivy::DocumentInput1> docs;
  for (uint64_t doc_index = 0; doc_index < docs_no; ++doc_index) {
    nlohmann::json data = {};
    nlohmann::json props = {};
    for (uint64_t prop_index = 0; prop_index < props_no; ++prop_index) {
      props[fmt::format("key{}", prop_index)] =
          fmt::format("value{} is AWESOME", prop_index);
    }
    data["data"] = props;
    data["metadata"] = {};
    data["metadata"]["gid"] = doc_index;
    data["metadata"]["txid"] = doc_index;
    data["metadata"]["deleted"] = false;
    data["metadata"]["is_node"] = false;
    cxxtantivy::DocumentInput1 doc = {
        .data = data.dump(),
    };
    // .gid = doc_index,
    // .txid = doc_index,
    // .deleted = false,
    // .is_node = false,
    // .props = props.dump()}};
    docs.push_back(doc);
  }
  return docs;
}

std::ostream &operator<<(std::ostream &os, const cxxtantivy::Element &element) {
  os << element.data;
  // os << "GID: " << element.gid << "; TXID: " << element.txid
  //    << "; DELETED: " << element.deleted << "; IS_NODE: " << element.is_node
  //    << "; PROPS: " << element.props;
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
