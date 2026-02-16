cmake_minimum_required(VERSION 3.21.3 FATAL_ERROR)
project(emulator)

set(CMAKE_C_STANDARD 23)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)

set(CMAKE_EXPORT_COMPILE_COMMANDS TRUE)

if(APPLE)
    set(
        PLATFORM_SRC
        src/platform/macos/timer.c
    )
set(
        PLATFORM_INCLUDE
        src/platform/macos/
    )
elseif(LINUX)
    set(
        PLATFORM_SRC
        src/platform/linux/timer.c
    )
set(
        PLATFORM_INCLUDE
        src/platform/linux/
    )

endif()

include(CheckIPOSupported)
check_ipo_supported(RESULT IPO_SUPPORTED OUTPUT IPO_OUTPUT)
if(IPO_SUPPORTED)
    message(STATUS "IPO supported: ${IPO_OUTPUT}")
    set(CMAKE_INTERPROCEDURAL_OPTIMIZATION TRUE)
else()
    message(STATUS "IPO not supported: ${IPO_OUTPUT}")
endif()

# set(CMAKE_CXX_FLAGS, "-Wall")

add_executable(${PROJECT_NAME}
    src/main.c
    src/cpu.c
    src/decode.c
    src/execute.c
    src/instruction_cache.c
    # src/memory.c
    src/address_bus.c
    src/bus_device.c
    src/devices/memory.c
    # TODO: Platform specific check
    xxhash/xxhash.c
    ${PLATFORM_SRC}
)

target_include_directories(${PROJECT_NAME} PUBLIC
    src/
    xxhash/
    ${PLATFORM_INCLUDE}
)
