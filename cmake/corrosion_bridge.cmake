# Creates a target including rust lib and cxxbridge named ${NAMESPACE}::${NAME}
function(add_library_rust)
    set(ONE_VALUE_KEYWORDS NAMESPACE NAME)
    cmake_parse_arguments(_RUST_LIB "${OPTIONS}" "${ONE_VALUE_KEYWORDS}" "${MULTI_VALUE_KEYWORDS}" ${ARGN})

    ### Check function inputs
    if("${_RUST_LIB_NAME}" STREQUAL "")
        message(FATAL_ERROR "Rust library name missed")
    endif()

    if("${_RUST_LIB_NAMESPACE}" STREQUAL "")
        message(FATAL_ERROR "Rust library namespace missed")
    endif()

    set(CRATE_MANIFEST_PATH "${CMAKE_CURRENT_LIST_DIR}/../Cargo.toml")

    if(NOT EXISTS "${CRATE_MANIFEST_PATH}")
        message(FATAL_ERROR "No Cargo.toml in ${CMAKE_CURRENT_LIST_DIR}")
    else()
        message(STATUS "Importing crate CRATE_MANIFEST_PATH=${CRATE_MANIFEST_PATH}")
    endif()

    ## Simplyfy inputs
    set(_LIB_NAME ${_RUST_LIB_NAME})
    set(CXXBRIDGE_TARGET ${_RUST_LIB_NAME}_bridge)

    ## Import Rust target
    corrosion_import_crate(
            MANIFEST_PATH "${CRATE_MANIFEST_PATH}"
            FEATURES ${MEMORY_ALLOCATOR_FEATURE})

    corrosion_add_cxxbridge(${CXXBRIDGE_TARGET} CRATE ${_LIB_NAME} FILES lib.rs)

    if(NOT DEFINED VCPKG_TARGET_TRIPLET)
        if(APPLE)
            set(VCPKG_TARGET_TRIPLET "x64-osx")
        else()
            set(VCPKG_TARGET_TRIPLET "x64-linux")
        endif()
        message(STATUS "set vcpkg target triplet VCPKG_TARGET_TRIPLET=${VCPKG_TARGET_TRIPLET}")
    endif()

    install(TARGETS ${_LIB_NAME}
            EXPORT ${EXPORT_TARGET_NAME}
            )

    set_target_properties(${CXXBRIDGE_TARGET} PROPERTIES
            PUBLIC_HEADER "${CMAKE_CURRENT_BINARY_DIR}/corrosion_generated/cxxbridge/${CXXBRIDGE_TARGET}/include/${CXXBRIDGE_TARGET}/lib.h")

    install(TARGETS ${CXXBRIDGE_TARGET}
            EXPORT ${EXPORT_TARGET_NAME}
            PUBLIC_HEADER DESTINATION include/${CXXBRIDGE_TARGET}
            )
endfunction(add_library_rust)

if("${CMAKE_SYSTEM_NAME}" STREQUAL "Windows")
    set(Rust_CARGO_TARGET "x86_64-pc-windows-gnu")
elseif("${CMAKE_SYSTEM_NAME}" STREQUAL "Linux")
    set(Rust_CARGO_TARGET "x86_64-unknown-linux-gnu")
elseif("${CMAKE_SYSTEM_NAME}" STREQUAL "Darwin")
    # if specify explicitly to use x86_64
    if("${CMAKE_OSX_ARCHITECTURES}" STREQUAL "x86_64")
        set(Rust_CARGO_TARGET "x86_64-apple-darwin")
    else()
        # on macOS "uname -m" returns the architecture (x86_64 or arm64)
        execute_process(
                COMMAND uname -m
                RESULT_VARIABLE exit_code_or_error
                OUTPUT_VARIABLE OSX_NATIVE_ARCHITECTURE
                OUTPUT_STRIP_TRAILING_WHITESPACE
        )
        if(OSX_NATIVE_ARCHITECTURE STREQUAL "arm64")
            set(Rust_CARGO_TARGET "aarch64-apple-darwin")
        else()
            set(Rust_CARGO_TARGET "x86_64-apple-darwin")
        endif()
    endif()
else()
    message(FATAL_ERROR "hardcoded ${CMAKE_SYSTEM_NAME} platform checks not supported outside windows-gnu, linux-gnu and apple-darwin")
endif()

find_package(Corrosion REQUIRED)
add_library_rust(NAME car NAMESPACE car)