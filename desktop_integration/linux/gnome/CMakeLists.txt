cmake_minimum_required(VERSION 3.16)

project(stl2thumbnail-gnome VERSION 1.0)

install(
FILES 
    stl.thumbnailer
    3mf.thumbnailer
    gcode.thumbnailer
DESTINATION 
    ${CMAKE_INSTALL_DATADIR}/thumbnailers
COMPONENT 
    gnome
)
