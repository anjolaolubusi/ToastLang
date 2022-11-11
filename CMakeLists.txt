cmake_minimum_required(VERSION 3.0.0)
project(ToastLang VERSION 0.1.0)

include(CTest)
enable_testing()

add_executable(ToastLang src/main.cpp
src/tokens.h src/tokens.cpp
src/lexer.h src/lexer.cpp 
src/parser.h src/parser.cpp
)

set(CPACK_PROJECT_NAME ${PROJECT_NAME})
set(CPACK_PROJECT_VERSION ${PROJECT_VERSION})
include(CPack)
