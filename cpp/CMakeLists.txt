cmake_minimum_required(VERSION 3.27)

project(LLVMTutorial LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

find_package(Boost REQUIRED)
find_package(LLVM REQUIRED CONFIG)

include_directories(${Boost_INCLUDE_DIRS} ${LLVM_INCLUDE_DIRS})
separate_arguments(LLVM_DEFINITIONS_LIST NATIVE_COMMAND ${LLVM_DEFINITIONS})
add_definitions(${LLVM_DEFINITIONS_LIST})

add_executable(sdc main.cc)

llvm_map_components_to_libnames(llvm_libs support core irreader)
target_link_libraries(sdc ${llvm_libs})


