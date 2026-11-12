cmake_minimum_required(VERSION 3.30)
project(kwxRust LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

include(FetchContent)

#-----------------------------------------------------------------------------
# Fetch kwxFFI (which includes wxWidgets)
#-----------------------------------------------------------------------------
FetchContent_Declare(kwxFFI
    # TEMPORARY: use the local sibling checkout so source changes show up immediately.
    SOURCE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../kwxFFI
    GIT_TAG origin/main
    GIT_SHALLOW TRUE
)
FetchContent_MakeAvailable(kwxFFI)

#-----------------------------------------------------------------------------
# Override kwxFFI output to place libs in project lib/ directory
#-----------------------------------------------------------------------------
set_target_properties(kwxFFI PROPERTIES
    ARCHIVE_OUTPUT_DIRECTORY "${PROJECT_SOURCE_DIR}/lib/$<CONFIG>"
    LIBRARY_OUTPUT_DIRECTORY "${PROJECT_SOURCE_DIR}/lib/$<CONFIG>"
    RUNTIME_OUTPUT_DIRECTORY "${PROJECT_SOURCE_DIR}/bin/$<CONFIG>"
)

#-----------------------------------------------------------------------------
# Print build configuration
#-----------------------------------------------------------------------------
message(STATUS "kwxRust CMake Configuration:")
message(STATUS "  Library output: ${PROJECT_SOURCE_DIR}/lib/$<CONFIG>")
message(STATUS "  kwxFFI will be built via FetchContent")
message(STATUS "  wxWidgets will be built via kwxFFI's FetchContent")
