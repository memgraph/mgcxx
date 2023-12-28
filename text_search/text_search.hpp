#include "cxx.hpp"
// NOTE: Error is not present under cxxtantivy.hpp, that's why cxx.hpp is
// required here.
#include "cxxtantivy.hpp"

// USAGE NOTE:
//   * Error returned from cxx calls are transformed into ::rust::Error
//     exception (that's by [cxx](https://cxx.rs/) design).
//   * All other text search functionality if located under ::memcxx::text_search namespace.
