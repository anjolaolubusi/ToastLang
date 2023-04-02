cmake_minimum_required(VERSION 3.0.0)
project(ToastLang VERSION 0.1.0)

find_package(LLVM REQUIRED CONFIG)

message(STATUS "Found LLVM ${LLVM_PACKAGE_VERSION}")
message(STATUS "Using LLVMConfig.cmake in: ${LLVM_DIR}")

include_directories(${LLVM_INCLUDE_DIRS})
separate_arguments(LLVM_DEFINITIONS_LIST NATIVE_COMMAND ${LLVM_DEFINITIONS})
add_definitions(${LLVM_DEFINITIONS_LIST})

include(CTest)
enable_testing()

add_executable(ToastLang src/main.cpp
src/tokens.h src/tokens.cpp
src/lexer.h src/lexer.cpp 
src/astNode.h
src/parser.h src/parser.cpp
src/codegen.h
)

llvm_map_components_to_libnames(llvm_libs support core irreader ${LLVM_TARGETS_TO_BUILD})

target_link_libraries(ToastLang ${llvm_libs})


set(CPACK_PROJECT_NAME ${PROJECT_NAME})
set(CPACK_PROJECT_VERSION ${PROJECT_VERSION})
include(CPack)
