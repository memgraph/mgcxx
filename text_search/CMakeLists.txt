include_directories(${CMAKE_BINARY_DIR}/text_search/rust)
add_subdirectory(rust)

if (${ENABLE_TESTS})
  add_executable(stress stress.cpp)
  target_link_libraries(stress PRIVATE fmtlib_static nlohmann_json::nlohmann_json cxxtantivy_rust)

  add_executable(bench bench.cpp)
  target_link_libraries(bench PRIVATE fmtlib_static benchmark::benchmark nlohmann_json::nlohmann_json cxxtantivy_rust)

  add_executable(unit unit.cpp)
  target_link_libraries(unit gtest gtest_main fmtlib_static nlohmann_json::nlohmann_json cxxtantivy_rust)
  add_test(NAME unit COMMAND unit)
endif()
