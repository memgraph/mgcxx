#include <chrono>
#include <iostream>
#include <vector>

#include <fmt/format.h>
#include <nlohmann/json.hpp>

#include "cxx.hpp"
#include "rust.hpp"

std::vector<cxxtantivy::DocumentInput1> dummy_data1(uint64_t docs_no = 1,
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
        .metadata_and_data = data.dump(),
    };
    docs.push_back(doc);
  }
  return docs;
}

std::vector<cxxtantivy::DocumentInput2> dummy_data2(uint64_t docs_no = 1,
                                                    uint64_t props_no = 1) {
  std::vector<cxxtantivy::DocumentInput2> docs;
  for (uint64_t doc_index = 0; doc_index < docs_no; ++doc_index) {
    nlohmann::json data = {};
    nlohmann::json props = {};
    for (uint64_t prop_index = 0; prop_index < props_no; ++prop_index) {
      props[fmt::format("key{}", prop_index)] =
          fmt::format("value{} is AWESOME", prop_index);
    }
    data["data"] = props;
    cxxtantivy::DocumentInput2 doc = {
        .gid = doc_index,
        .data = data.dump(),
    };
    docs.push_back(doc);
  }
  return docs;
}

std::ostream &operator<<(std::ostream &os,
                         const cxxtantivy::DocumentOutput &element) {
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
