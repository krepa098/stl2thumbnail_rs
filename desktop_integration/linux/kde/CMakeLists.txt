cmake_minimum_required(VERSION 3.16)

project(stl2thumbnail-kde VERSION 1.0)

find_package(ECM REQUIRED NO_MODULE)
set(CMAKE_MODULE_PATH ${ECM_MODULE_PATH} ${ECM_KDE_MODULE_DIR} ${CMAKE_CURRENT_SOURCE_DIR}/cmake)

find_package(KF5 REQUIRED COMPONENTS KIO)

include(FeatureSummary)
include(WriteBasicConfigVersionFile)
include(KDEInstallDirs)
include(KDECMakeSettings)
include(KDECompilerSettings)

include_directories(${CMAKE_SOURCE_DIR}/lib)

set(SRC_FILES
    "thumbcreator.cpp"
)

set(LIBS
    KF5::KIOWidgets
)

add_library(${PROJECT_NAME} MODULE ${SRC_FILES})
add_dependencies(${PROJECT_NAME} libstl2thumbnail)
target_link_libraries(${PROJECT_NAME} ${LIBS} libstl2thumbnail)

install(
    TARGETS 
        ${PROJECT_NAME} 
    DESTINATION 
        ${PLUGIN_INSTALL_DIR} 
    COMPONENT 
        kde
)

install(
    FILES 
        "stl-thumbnailer.desktop" 
    DESTINATION 
        ${SERVICES_INSTALL_DIR} 
    COMPONENT 
        kde
)
