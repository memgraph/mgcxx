string(TOLOWER ${CMAKE_BUILD_TYPE} cmake_lower_build_type)
option(CXXTANTIVY_VERBOSE "Make cargo build part verbose" OFF)
set(CXXTANTIVY_CARGO_FLAGS "")
if (CXXTANTIVY_VERBOSE)
  set(CXXTANTIVY_CARGO_FLAGS ${CXXTANTIVY_CARGO_FLAGS} --verbose)
endif()
if (cmake_lower_build_type STREQUAL "debug")
    set(CARGO_CMD cargo build ${CXXTANTIVY_CARGO_FLAGS})
    set(CXXTANTIVY_RUST_TARGET_DIR "debug")
else ()
    set(CARGO_CMD cargo build --release ${CXXTANTIVY_CARGO_FLAGS})
    set(CXXTANTIVY_RUST_TARGET_DIR "release")
endif ()

if(CXXTANTIVY_ENABLE_LTO)
    set(CXXTANTIVY_RUST_FLAGS "-Clinker-plugin-lto" "-Clinker=clang" "-Clink-arg=-fuse-ld=lld")
endif()

set(CXXTANTIVY_BRIDGE_LIB_RS ${CMAKE_CURRENT_SOURCE_DIR}/src/lib.rs)
set(CXXTANTIVY_BRIDGE_CC ${CMAKE_CURRENT_BINARY_DIR}/cxxbridge/text_search/src/lib.rs.cc)
set(CXXTANTIVY_BRIDGE_H ${CMAKE_CURRENT_BINARY_DIR}/cxxbridge/text_search/src/lib.rs.h)
set(CXXTANTIVY_BRIDGE_CXX ${CMAKE_CURRENT_BINARY_DIR}/cxxbridge/rust/cxx.h)
set(CXXTANTIVY_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/${CXXTANTIVY_RUST_TARGET_DIR}/libtext_search.a")
set(CXXTANTIVY_RUST_CPP "${CMAKE_CURRENT_BINARY_DIR}/cxxtantivy.cpp")
set(CXXTANTIVY_RUST_HPP "${CMAKE_CURRENT_BINARY_DIR}/cxxtantivy.hpp")
set(CXXTANTIVY_RUST_CXX "${CMAKE_CURRENT_BINARY_DIR}/cxx.hpp")
add_library(cxxtantivy_rust STATIC ${CXXTANTIVY_RUST_CPP})
add_custom_command(
    OUTPUT ${CXXTANTIVY_RUST_CPP}
    COMMAND CARGO_TARGET_DIR=${CMAKE_CURRENT_BINARY_DIR} RUSTFLAGS="${CXXTANTIVY_RUST_FLAGS}" ${CARGO_CMD}
    COMMAND cp ${CXXTANTIVY_BRIDGE_CC} ${CXXTANTIVY_RUST_CPP}
    COMMAND cp ${CXXTANTIVY_BRIDGE_H} ${CXXTANTIVY_RUST_HPP}
    COMMAND cp ${CXXTANTIVY_BRIDGE_CXX} ${CXXTANTIVY_RUST_CXX}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    DEPENDS ${CXXTANTIVY_BRIDGE_LIB_RS}
)
target_link_libraries(cxxtantivy_rust ${CXXTANTIVY_RUST_LIB})
add_custom_command(TARGET cxxtantivy_rust POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy
        ${CMAKE_CURRENT_SOURCE_DIR}/text_search.hpp
        ${CMAKE_CURRENT_BINARY_DIR}/text_search.hpp)
set_target_properties(cxxtantivy_rust PROPERTIES PUBLIC_HEADER "${CMAKE_CURRENT_BINARY_DIR}/text_search.hpp;${CXXTANTIVY_RUST_HPP};${CXXTANTIVY_RUST_CXX}")
INSTALL(TARGETS cxxtantivy_rust
        LIBRARY DESTINATION lib
        PUBLIC_HEADER DESTINATION include
)

if (${ENABLE_TESTS})
  include_directories(${CMAKE_BINARY_DIR}/text_search) # TODO(gitbuda): include_directories should be done per target + include the .hpp file
  add_test(NAME cxxtantivy_rust_test
           COMMAND cargo test
           WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
  add_executable(test_stress test_stress.cpp)
  target_link_libraries(test_stress PRIVATE fmtlib_static nlohmann_json::nlohmann_json cxxtantivy_rust)
  add_executable(test_bench test_bench.cpp)
  target_link_libraries(test_bench PRIVATE fmtlib_static benchmark::benchmark nlohmann_json::nlohmann_json cxxtantivy_rust)
  add_executable(test_unit test_unit.cpp)
  target_link_libraries(test_unit gtest gtest_main fmtlib_static nlohmann_json::nlohmann_json cxxtantivy_rust)
  add_test(NAME test_unit COMMAND test_unit)
endif()
