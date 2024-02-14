cmake_minimum_required(VERSION 3.16)

project(stl2thumbnail VERSION 1.0)

option(GNOME "Gnome Thumbnailer" ON)
option(KDE "Kde Thumbnailer" ON)
option(DEB "Generate a DEB archive" ON)

include(GNUInstallDirs)

# compile lib
add_custom_target(
    stl2thumbnail-rs ALL
    COMMAND cargo build --release --manifest-path=${CMAKE_SOURCE_DIR}/stl2thumbnail_rs/Cargo.toml
    DEPENDS ${CMAKE_SOURCE_DIR}/stl2thumbnail_rs/target/release/libstl2thumbnail.so
)

# define lib
add_library(libstl2thumbnail SHARED IMPORTED)
target_include_directories(libstl2thumbnail INTERFACE ${CMAKE_SOURCE_DIR}/stl2thumbnail_rs/include)
set_target_properties(libstl2thumbnail PROPERTIES
    IMPORTED_LOCATION ${CMAKE_SOURCE_DIR}/stl2thumbnail_rs/target/release/libstl2thumbnail.so
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
    # generate a DEB when cpack is run
    # run with 'make package'
    SET(CPACK_GENERATOR "DEB")
    SET(CPACK_SET_DESTDIR TRUE)
    SET(CPACK_DEBIAN_PACKAGE_MAINTAINER "krepa098@gmail.com")
    SET(CPACK_PACKAGE_VERSION_MAJOR "1")
    SET(CPACK_PACKAGE_VERSION_MINOR "0")
    SET(CPACK_PACKAGE_VERSION_PATCH "0")
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