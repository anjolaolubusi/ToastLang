cmake_minimum_required(VERSION 3.0.0)
project(ToastLang VERSION 0.1.0)


include(CTest C)
enable_testing()

add_executable(ToastLang src/main.c
src/tokens.h src/tokens.c 
src/astNode.h src/astNode.c
src/codeGen.h src/codeGen.c
)

set(CPACK_PROJECT_NAME ${PROJECT_NAME})
set(CPACK_PROJECT_VERSION ${PROJECT_VERSION})
include(CPack)
