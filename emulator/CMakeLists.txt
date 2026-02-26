cmake_minimum_required(VERSION 3.21.3 FATAL_ERROR)
project(emulator)

set(CMAKE_C_STANDARD 23)
set(CMAKE_C_STANDARD_REQUIRED ON)

set(CMAKE_EXPORT_COMPILE_COMMANDS TRUE)

if(MSVC)
    message(FATAL_ERROR "MSVC not supported")
endif()



# Link time optimization
include(CheckIPOSupported)
check_ipo_supported(RESULT IPO_SUPPORTED OUTPUT IPO_OUTPUT)
if(IPO_SUPPORTED)
    message(STATUS "IPO supported: ${IPO_OUTPUT}")
    set(CMAKE_INTERPROCEDURAL_OPTIMIZATION TRUE)
else()
    message(STATUS "IPO not supported: ${IPO_OUTPUT}")
endif()

add_executable(${PROJECT_NAME}
    src/main.c
    src/cpu.c
    src/decode.c
    src/execute.c
    src/instruction_cache.c
    src/block.c
    src/address_bus.c
    src/bus_device.c
    src/sync/spinlock.c
    thirdparty/xxhash/xxhash.c
)


target_include_directories(${PROJECT_NAME} PUBLIC
    src/
    thirdparty/
    thirdparty/xxhash/
)

# Compiler flags
target_compile_options(${PROJECT_NAME} PRIVATE -Wall -Wextra)

# Toggle platform specific files
if(APPLE)
    target_sources(${PROJECT_NAME} PRIVATE
        thirdparty/tinycthread/tinycthread.c
        src/platform/macos/timer.c
    )

    target_include_directories(${PROJECT_NAME} PRIVATE
        src/platform/macos
    )
elseif(LINUX)
    target_sources(${PROJECT_NAME} PRIVATE
        src/platform/linux/timer.h
    )
    target_include_directories(${PROJECT_NAME} PRIVATE
        src/platform/linux
    )
endif()
