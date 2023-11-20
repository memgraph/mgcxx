#include <iostream>

#include "rust.hpp"

int main() {
  text_search::TextInput text = {.data = "data"};
  auto is_added = cxxtantivy::add(text);
  text_search::SearchInput search = {.query = "query"};
  auto result = cxxtantivy::search(search);
  std::cout << "  DummyDocID: " << result.docId << std::endl;
  return 0;
}
