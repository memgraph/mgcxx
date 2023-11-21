#include <iostream>

#include "cxx.hpp"
#include "rust.hpp"

int main() {
  try {
    auto context = cxxtantivy::init();
    text_search::TextInput text = {.data = "{key:value}"};
    auto is_added = cxxtantivy::add(context, text);
    text_search::SearchInput search = {.query = "value"};
    auto result = cxxtantivy::search(context, search);
    for (const auto &docId : result.doc_ids) {
      std::cout << "  FoundDocID: " << docId << std::endl;
    }
  } catch (const rust::Error &error) {
    std::cout << error.what() << std::endl;
  }
  return 0;
}
