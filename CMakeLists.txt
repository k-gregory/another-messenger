cmake_minimum_required(VERSION 2.8)

project(messenger)

set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTOUIC ON)

find_package(Qt5Widgets REQUIRED)
find_package(Qt5Network REQUIRED)

SET(SOURCE_FILES
    "src/main.cpp"
    "src/dialerwindow.cpp")

SET(HEADER_FILES
    "src/dialerwindow.h"
   ) 

SET(UI_FILES)

add_executable(${PROJECT_NAME}
    ${SOURCE_FILES} ${UI_FILES}
    )

target_link_libraries(${PROJECT_NAME} Qt5::Widgets Qt5::Network)
add_custom_target(headers ${SOURCE_FILES} ${HEADER_FILES})
