cmake_minimum_required(VERSION 3.16)

project(stl2thumbnail VERSION 1.2.0)

option(GNOME "Gnome Thumbnailer" ON)
option(KDE "Kde Thumbnailer" ON)
option(DEB "Generate a DEB archive" ON)

include(GNUInstallDirs)

# compile static lib and cli binary
add_custom_target(
    stl2thumbnail-rs ALL
    COMMAND cargo build --release --manifest-path=${CMAKE_SOURCE_DIR}/stl2thumbnail_rs/Cargo.toml
)

# define lib
add_library(libstl2thumbnail STATIC IMPORTED)
add_dependencies(libstl2thumbnail stl2thumbnail-rs)
target_include_directories(libstl2thumbnail INTERFACE ${CMAKE_SOURCE_DIR}/stl2thumbnail_rs/include)
set_target_properties(libstl2thumbnail PROPERTIES
    IMPORTED_LOCATION ${CMAKE_SOURCE_DIR}/stl2thumbnail_rs/target/release/libstl2thumbnail.a
)

# install cli
install(
    PROGRAMS ${CMAKE_SOURCE_DIR}/stl2thumbnail_rs/target/release/stl2thumbnail
    DESTINATION ${CMAKE_INSTALL_BINDIR}
    COMPONENT cli
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
    set(CPACK_COMPONENTS_ALL cli)

    if(GNOME)
        list(APPEND CPACK_COMPONENTS_ALL gnome)
    endif()

    if(KDE)
        list(APPEND CPACK_COMPONENTS_ALL kde)
    endif()

    SET(CPACK_DEBIAN_CLI_PACKAGE_DEPENDS "")
    SET(CPACK_DEBIAN_GNOME_PACKAGE_DEPENDS "${PROJECT_NAME}-cli")
    SET(CPACK_DEBIAN_KDE_PACKAGE_DEPENDS "libkf5kiocore5")

    include(CPack)

endif()