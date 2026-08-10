cmake_minimum_required(VERSION 3.16)
project(wallpiper-kde-portal LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

find_package(ECM REQUIRED NO_MODULE)
set(CMAKE_MODULE_PATH
    ${ECM_MODULE_PATH}
    ${ECM_FIND_MODULE_DIR}
    ${CMAKE_MODULE_PATH}
)

find_package(Qt6 6.7 REQUIRED COMPONENTS Core Gui Qml Quick)
find_package(EGL REQUIRED)

qt_standard_project_setup(REQUIRES 6.7)

qt_add_qml_module(wallpiper_capture_plugin
    URI dev.wallpiper.capture
    VERSION 1.0
    PLUGIN_TARGET wallpiper_capture_plugin

    SOURCES
        include/protocol.h
        include/capture_socket.h
        include/ctl_listener.h
        include/capture_coordinator.h
        include/egl_dmabuf_importer.h
        include/capture_item.h

        src/protocol.cpp
        src/capture_socket.cpp
        src/ctl_listener.cpp
        src/capture_coordinator.cpp
        src/egl_dmabuf_importer.cpp
        src/capture_item.cpp
)

target_include_directories(wallpiper_capture_plugin PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/src
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)

target_link_libraries(wallpiper_capture_plugin PRIVATE
    Qt6::Core
    Qt6::Gui
    Qt6::Qml
    Qt6::Quick
    EGL::EGL
)

qt6_query_qml_module(wallpiper_capture_plugin QMLDIR wallpiper_capture_qmldir)

install(TARGETS wallpiper_capture_plugin
    LIBRARY DESTINATION "${QT6_INSTALL_QML}/dev/wallpiper/capture"
    RUNTIME DESTINATION "${QT6_INSTALL_QML}/dev/wallpiper/capture"
)

install(
    FILES ${wallpiper_capture_qmldir}
    DESTINATION "${QT6_INSTALL_QML}/dev/wallpiper/capture"
)
