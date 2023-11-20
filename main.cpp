#include <iostream>

#include "rust.hpp"

int main() {
  auto context = cxxtantivy::init();
  text_search::TextInput text = {.data = "{key:value}"};
  auto is_added = cxxtantivy::add(context, text);
  text_search::SearchInput search = {.query = "value"};
  auto result = cxxtantivy::search(context, search);
  for (const auto &docId : result.doc_ids) {
    std::cout << "  FoundDocID: " << docId << std::endl;
  }
  return 0;
}
