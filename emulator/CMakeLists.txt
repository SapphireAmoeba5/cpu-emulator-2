cmake_minimum_required(VERSION 3.21.3 FATAL_ERROR)
project(emulator)

set(CMAKE_C_STANDARD 23)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)

set(CMAKE_EXPORT_COMPILE_COMMANDS TRUE)

# set(CMAKE_CXX_FLAGS, "-Wall")

add_executable(${PROJECT_NAME}
    src/main.c
    src/cpu.c
    # src/memory.c
    src/instructions.c
    src/address_bus.c
    src/bus_device.c
    src/devices/memory.c
)

target_include_directories(${PROJECT_NAME} PUBLIC
    src/
)
