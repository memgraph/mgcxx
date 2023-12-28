cmake_minimum_required(VERSION 3.15)
set (CMAKE_CXX_STANDARD 20)
if (NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE "Debug")
endif()
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
include(ExternalProject)
include(FetchContent)
project(cxxtantivy)

file(DOWNLOAD https://raw.githubusercontent.com/memgraph/cmake/main/modules/external-fmt.cmake
     ${CMAKE_BINARY_DIR}/external-fmt.cmake)
include(${CMAKE_BINARY_DIR}/external-fmt.cmake)
file(DOWNLOAD https://raw.githubusercontent.com/memgraph/cmake/main/modules/fetch-nlohmann.cmake
     ${CMAKE_BINARY_DIR}/fetch-nlohmann.cmake)
include(${CMAKE_BINARY_DIR}/fetch-nlohmann.cmake)
file(DOWNLOAD https://raw.githubusercontent.com/memgraph/cmake/main/modules/fetch-gbenchmark.cmake
     ${CMAKE_BINARY_DIR}/fetch-gbenchmark.cmake)
include(${CMAKE_BINARY_DIR}/fetch-gbenchmark.cmake)
file(DOWNLOAD https://raw.githubusercontent.com/memgraph/cmake/main/modules/fetch-gtest.cmake
     ${CMAKE_BINARY_DIR}/fetch-gtest.cmake)
include(${CMAKE_BINARY_DIR}/fetch-gtest.cmake)

# TODO(gitbuda): Make LTO work on Mac because ldd (consider options and default).
message(STATUS "CMake build type: ${CMAKE_BUILD_TYPE}")
option(CXXTANTIVY_ENABLE_LTO "Enable cross language linking time optimization" OFF)
if (APPLE AND CXXTANTIVY_ENABLE_LTO)
  message(STATUS "LTO on Apple not possible -> disabling LTO...")
  set(CXXTANTIVY_ENABLE_LTO OFF)
endif()
if (CXXTANTIVY_ENABLE_LTO)
  include(CheckIPOSupported)
  check_ipo_supported(RESULT supported OUTPUT error)
  if(supported)
      message(STATUS "IPO / LTO enabled")
      set(CMAKE_INTERPROCEDURAL_OPTIMIZATION TRUE)
      add_link_options(-fuse-ld=lld)
  else()
      message(STATUS "IPO / LTO not supported: <${error}>")
  endif()
endif()

option(ENABLE_TESTS "Enable tests" ON)
if (${ENABLE_TESTS})
    enable_testing()
endif()

add_subdirectory(text_search)
