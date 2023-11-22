#include <iostream>

// TODO(gitbuda): Move includes to cxxtantivy/rust|cxx.hpp.
// TODO(gitbuda): cxxtantivy::function but rust::Error -> unify.
#include "cxx.hpp"
#include "rust.hpp"

int main() {
  // TODO(gitbuda): Introduce versions of each document.
  // TODO(gitbuda): Introduce propery graph document schema.
  try {
    auto context = cxxtantivy::init();
    text_search::TextInput text1 = {.data = "{key:value1}"};
    cxxtantivy::add(context, text1);
    text_search::SearchInput search = {.query = "value1"};
    auto result = cxxtantivy::search(context, search);
    // TODO(gitbuda): Search seems to be returning wrong results (a bunch of 0).
    for (const auto &docId : result.doc_ids) {
      std::cout << "  FoundDocID: " << docId << std::endl;
    }
  } catch (const rust::Error &error) {
    std::cout << error.what() << std::endl;
  }
  return 0;
}
