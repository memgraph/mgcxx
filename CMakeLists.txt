project(cxxtantivy)
cmake_minimum_required(VERSION 3.15)
set (CMAKE_CXX_STANDARD 20)
if (NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE "Debug")
endif()
message(STATUS "CMake build type: ${CMAKE_BUILD_TYPE}")

include_directories(${CMAKE_BINARY_DIR}/rust)
add_subdirectory(rust)

add_executable(main main.cpp)
target_link_libraries(main cxxtantivy_rust)
