#include "cxx.hpp"
// NOTE: Error is not present under mgcxx_text_search.hpp (the way how cxx
// works), that's why cxx.hpp is required here.
#include "mgcxx_text_search.hpp"

// USAGE NOTE:
//   * Error returned from cxx calls are transformed into ::rust::Error
//     exception (that's by [cxx](https://cxx.rs/) design).
//   * All other text search functionality if located under ::memcxx::text_search namespace.
