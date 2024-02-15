cmake_minimum_required(VERSION 3.16)

project(stl2thumbnail VERSION 1.0)

option(GNOME "Gnome Thumbnailer" ON)
option(KDE "Kde Thumbnailer" ON)
option(DEB "Generate a DEB archive" ON)

find_package(Git)

set(${PROJECT_NAME}_VERSION_MAJOR 1)
set(${PROJECT_NAME}_VERSION_MINOR 0)
set(${PROJECT_NAME}_VERSION_PATCH 0)

if (GIT_FOUND)
    execute_process(
        COMMAND ${GIT_EXECUTABLE} describe --abbrev=0 --tags
        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
        OUTPUT_VARIABLE ${PROJECT_NAME}_VERSION_STRING
        OUTPUT_STRIP_TRAILING_WHITESPACE
    )

    string(REGEX MATCHALL "-.*$|[0-9]+" ${PROJECT_NAME}_PARTIAL_VERSION_LIST ${${PROJECT_NAME}_VERSION_STRING})
    list(LENGTH ${PROJECT_NAME}_PARTIAL_VERSION_LIST ${PROJECT_NAME}_PARTIAL_VERSION_LIST_LEN)

    list(GET ${PROJECT_NAME}_PARTIAL_VERSION_LIST 0 ${PROJECT_NAME}_VERSION_MAJOR)
    list(GET ${PROJECT_NAME}_PARTIAL_VERSION_LIST 1 ${PROJECT_NAME}_VERSION_MINOR)
    if(${PROJECT_NAME}_PARTIAL_VERSION_LIST_LEN GREATER_EQUAL 3)
        list(GET ${PROJECT_NAME}_PARTIAL_VERSION_LIST 2 ${PROJECT_NAME}_VERSION_PATCH)
    endif()

    message(STATUS "Version from git tag: ${${PROJECT_NAME}_VERSION_MAJOR}.${${PROJECT_NAME}_VERSION_MINOR}.${${PROJECT_NAME}_VERSION_PATCH}")
endif()

include(GNUInstallDirs)

# compile lib
add_custom_target(
    stl2thumbnail-rs ALL
    COMMAND cargo build --release --manifest-path=${CMAKE_SOURCE_DIR}/stl2thumbnail_rs/Cargo.toml
)

# define lib
add_library(libstl2thumbnail SHARED IMPORTED)
add_dependencies(libstl2thumbnail stl2thumbnail-rs)
target_include_directories(libstl2thumbnail INTERFACE ${CMAKE_SOURCE_DIR}/stl2thumbnail_rs/include)
set_target_properties(libstl2thumbnail PROPERTIES
    IMPORTED_LOCATION ${CMAKE_SOURCE_DIR}/stl2thumbnail_rs/target/release/libstl2thumbnail.so
    IMPORTED_NO_SONAME TRUE
)

# install lib
install(
    FILES ${CMAKE_SOURCE_DIR}/stl2thumbnail_rs/target/release/libstl2thumbnail.so
    DESTINATION lib
    COMPONENT core
)

# install cli
install(
    FILES ${CMAKE_SOURCE_DIR}/stl2thumbnail_rs/target/release/stl2thumbnail
    DESTINATION bin
    COMPONENT core
)

# install ffi header
install(
    FILES ${CMAKE_SOURCE_DIR}/stl2thumbnail_rs/include/stl2thumbnail.h
    DESTINATION include
    COMPONENT core
)

if(KDE)
    add_subdirectory(desktop_integration/linux/kde)
endif()

if(GNOME)
    add_subdirectory(desktop_integration/linux/gnome)
endif()

if(DEB)
    # generate a DEB with cpack
    # run with 'make package'
    SET(CPACK_GENERATOR "DEB")
    SET(CPACK_SET_DESTDIR TRUE)
    SET(CPACK_DEBIAN_PACKAGE_MAINTAINER "krepa098@gmail.com")
    SET(CPACK_PACKAGE_VERSION_MAJOR ${${PROJECT_NAME}_VERSION_MAJOR})
    SET(CPACK_PACKAGE_VERSION_MINOR ${${PROJECT_NAME}_VERSION_MINOR})
    SET(CPACK_PACKAGE_VERSION_PATCH ${${PROJECT_NAME}_VERSION_PATCH})
    set(CPACK_ARCHIVE_COMPONENT_INSTALL ON)
    SET(CPACK_DEB_COMPONENT_INSTALL ON)
    set(CPACK_COMPONENTS_GROUPING ONE_PER_GROUP)
    set(CPACK_COMPONENTS_ALL core)

    if(GNOME)
        list(APPEND CPACK_COMPONENTS_ALL gnome)
    endif()

    if(KDE)
        list(APPEND CPACK_COMPONENTS_ALL kde)
    endif()

    SET(CPACK_DEBIAN_CORE_PACKAGE_DEPENDS "lbzip2")
    SET(CPACK_DEBIAN_GNOME_PACKAGE_DEPENDS "stl2thumbnail-core")
    SET(CPACK_DEBIAN_KDE_PACKAGE_DEPENDS "stl2thumbnail-core, libkf5kiocore5")

    include(CPack)

endif()