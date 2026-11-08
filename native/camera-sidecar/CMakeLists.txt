cmake_minimum_required(VERSION 3.20)
project(camera_sidecar LANGUAGES CXX)

# ---------------------------------------------------------------------------
# Session Manager camera sidecar (imported from the Sony camera lab's `crlab`).
#
#   Always built (no SDK needed):
#     crlab_core            logging, transport manager, sha256, diagnostics, percent-decoding
#     crlab-usb-watch       IOKit Sony USB hot-plug monitor (diagnostics)
#     camera_sidecar_tests  transport manager tests (mock camera), registered with ctest
#   camera-sidecar          the sidecar executable:
#     default               stub backend that refuses every camera operation (any Mac, CI)
#     -DCAMERA_SIDECAR_WITH_SDK=ON -DCRSDK_ROOT=<dir>
#                           built against an installed Sony Camera Remote SDK (build machine only)
#
# Nothing from the Sony SDK is ever part of this source tree. Build with tools/sidecar/build.sh,
# which builds under target/camera-sidecar/ and never inside this directory.
# ---------------------------------------------------------------------------

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE RelWithDebInfo)
endif()
if(NOT CMAKE_OSX_ARCHITECTURES)
  set(CMAKE_OSX_ARCHITECTURES arm64)
endif()
set(CMAKE_OSX_DEPLOYMENT_TARGET "14.1" CACHE STRING "macOS deployment target (SDK supports macOS 14.1+)")

if(NOT APPLE)
  message(FATAL_ERROR "camera-sidecar targets macOS (IOKit USB monitoring, macOS SDK libraries).")
endif()

option(CAMERA_SIDECAR_WITH_SDK "Build camera-sidecar against the Sony SDK found under CRSDK_ROOT" OFF)
set(CRSDK_ROOT "" CACHE PATH "Extracted Sony Camera Remote SDK (CrSDK_vX_Mac); required with CAMERA_SIDECAR_WITH_SDK=ON")

add_compile_options(-Wall -Wextra -Wno-unused-parameter)

# ----------------------------------------------------------------- core
add_library(crlab_core STATIC
  src/json_log.cpp
  src/transport_manager.cpp
  src/sha256_macos.cpp
  src/diagnostics.cpp
)
target_include_directories(crlab_core PUBLIC include)

add_library(crlab_usb STATIC src/usb_monitor_macos.cpp)
target_include_directories(crlab_usb PUBLIC include)
target_link_libraries(crlab_usb PUBLIC "-framework IOKit" "-framework CoreFoundation")

add_executable(crlab-usb-watch src/usb_watch_main.cpp)
target_link_libraries(crlab-usb-watch PRIVATE crlab_core crlab_usb)

enable_testing()
add_executable(camera_sidecar_tests tests/test_transport_manager.cpp)
target_link_libraries(camera_sidecar_tests PRIVATE crlab_core)
add_test(NAME transport_manager COMMAND camera_sidecar_tests)

set(CRLAB_SDK_SOURCES
  src/camera_discovery.cpp
  src/camera_connection.cpp
  src/camera_properties.cpp
  src/camera_capture.cpp
)
set(CRLAB_CLI_SOURCES src/main.cpp src/sdk_adapter.cpp)

if(NOT CAMERA_SIDECAR_WITH_SDK)
  message(STATUS "camera-sidecar: stub backend (no camera access). Pass -DCAMERA_SIDECAR_WITH_SDK=ON -DCRSDK_ROOT=<dir> for the SDK build.")
  add_executable(camera-sidecar ${CRLAB_CLI_SOURCES} src/sdk_stub.cpp)
  target_link_libraries(camera-sidecar PRIVATE crlab_core crlab_usb)
  return()
endif()

# ----------------------------------------------------------------- SDK discovery (SDK build only)
set(CRLAB_SDK_FOUND OFF)
if(CRSDK_ROOT AND EXISTS "${CRSDK_ROOT}")
  file(GLOB_RECURSE _hdr "${CRSDK_ROOT}/*CameraRemote_SDK.h")
  file(GLOB_RECURSE _core "${CRSDK_ROOT}/*libCr_Core.dylib")
  if(_hdr AND _core)
    list(GET _hdr 0 _h0)
    list(GET _core 0 _c0)
    get_filename_component(CRLAB_SDK_INCLUDE_DIR "${_h0}" DIRECTORY)
    set(CRLAB_SDK_CORE_LIB "${_c0}")
    get_filename_component(CRLAB_SDK_LIB_DIR "${_c0}" DIRECTORY)
    set(CRLAB_SDK_FOUND ON)
  endif()
endif()
if(NOT CRLAB_SDK_FOUND)
  message(FATAL_ERROR
    "\nSony Camera Remote SDK not found under CRSDK_ROOT:\n  '${CRSDK_ROOT}'\n"
    "Expected CameraRemote_SDK.h and libCr_Core.dylib somewhere inside it.\n"
    "Download the macOS package from https://support.d-imaging.sony.co.jp/app/sdk/en/index.html\n"
    "(accept Sony's license yourself), extract it OUTSIDE this repository, then run\n"
    "  tools/sidecar/build.sh --sdk <that directory>\n")
endif()
message(STATUS "Sony SDK headers: ${CRLAB_SDK_INCLUDE_DIR}")
message(STATUS "Sony SDK Cr_Core: ${CRLAB_SDK_CORE_LIB}")
execute_process(COMMAND lipo -info "${CRLAB_SDK_CORE_LIB}" OUTPUT_VARIABLE _lipo OUTPUT_STRIP_TRAILING_WHITESPACE ERROR_QUIET)
message(STATUS "Sony SDK architectures: ${_lipo}")
if(NOT _lipo MATCHES "arm64")
  message(WARNING "libCr_Core.dylib does not report arm64. Apple Silicon build will fail to link; check the SDK package.")
endif()

# ----------------------------------------------------------------- symbol probes
include(CheckCXXSourceCompiles)
# Re-run configuration (and the probes) whenever the symbol list changes.
set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS "${CMAKE_CURRENT_SOURCE_DIR}/src/sdk_symbols.def")
function(crlab_probe_symbols include_dir out_header)
  set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)
  set(CMAKE_REQUIRED_INCLUDES "${include_dir}")
  set(CMAKE_REQUIRED_FLAGS "-std=c++17")
  set(CMAKE_REQUIRED_QUIET ON)
  set(prelude "#include \"CameraRemote_SDK.h\"\n#include \"CrDeviceProperty.h\"\n#include \"ICrCameraObjectInfo.h\"\n#include \"IDeviceCallback.h\"\n")
  file(STRINGS "${CMAKE_CURRENT_SOURCE_DIR}/src/sdk_symbols.def" lines)
  set(content "// Generated by CMake from src/sdk_symbols.def against:\n//   ${include_dir}\n#pragma once\n")
  set(n_yes 0)
  set(n_no 0)
  set(missing "")
  foreach(line IN LISTS lines)
    if(line MATCHES "^[ \t]*#" OR line MATCHES "^[ \t]*$")
      continue()
    endif()
    if(NOT line MATCHES "^([a-z]+)[ \t]+([A-Za-z0-9_]+)[ \t]*(.*)$")
      continue()
    endif()
    set(kind "${CMAKE_MATCH_1}")
    set(name "${CMAKE_MATCH_2}")
    set(rest "${CMAKE_MATCH_3}")
    if(kind STREQUAL "prop")
      set(macro "CRLAB_HAS_PROP_${name}")
      set(src "${prelude}int main(){ auto v = SCRSDK::CrDeviceProperty_${name}; (void)v; return 0; }")
    elseif(kind STREQUAL "sym")
      set(macro "CRLAB_HAS_${name}")
      set(src "${prelude}int main(){ auto v = ${rest}; (void)v; return 0; }")
    elseif(kind STREQUAL "callback")
      set(macro "CRLAB_HAS_${name}")
      set(src "${prelude}struct CrlabProbe : SCRSDK::IDeviceCallback { ${rest} };\nint main(){ return 0; }")
    else()
      continue()
    endif()
    unset(${macro} CACHE)
    check_cxx_source_compiles("${src}" ${macro})
    if(${macro})
      string(APPEND content "#define ${macro} 1\n")
      math(EXPR n_yes "${n_yes}+1")
    else()
      string(APPEND content "#define ${macro} 0\n")
      math(EXPR n_no "${n_no}+1")
      list(APPEND missing "${name}")
    endif()
  endforeach()
  file(WRITE "${out_header}" "${content}")
  message(STATUS "SDK symbol probes (${include_dir}): ${n_yes} present, ${n_no} absent")
  if(missing)
    string(REPLACE ";" ", " missing_s "${missing}")
    message(STATUS "  absent: ${missing_s}")
  endif()
endfunction()

# ----------------------------------------------------------------- camera-sidecar (SDK)
set(_gen "${CMAKE_CURRENT_BINARY_DIR}/generated/sdk")
crlab_probe_symbols("${CRLAB_SDK_INCLUDE_DIR}" "${_gen}/crlab_sdk_features.h")
add_executable(camera-sidecar ${CRLAB_CLI_SOURCES} ${CRLAB_SDK_SOURCES})
target_include_directories(camera-sidecar SYSTEM PRIVATE "${CRLAB_SDK_INCLUDE_DIR}")
target_include_directories(camera-sidecar PRIVATE "${_gen}")
target_link_libraries(camera-sidecar PRIVATE crlab_core crlab_usb "${CRLAB_SDK_CORE_LIB}")
set_target_properties(camera-sidecar PROPERTIES BUILD_WITH_INSTALL_RPATH TRUE BUILD_RPATH "@executable_path" INSTALL_RPATH "@executable_path")

# Runtime layout for running the build-directory binary on the build machine only: the SDK loads
# libCr_Core and the CrAdapter plug-ins relative to the executable. These copies stay in the build
# directory; they never enter the repository or the app bundle (the user installs the Sony runtime
# as a camera support package).
file(GLOB _sdk_dylibs "${CRLAB_SDK_LIB_DIR}/*.dylib")
file(GLOB_RECURSE _adapter_probe "${CRSDK_ROOT}/*.dylib")
list(FILTER _adapter_probe INCLUDE REGEX "/CrAdapter/[^/]+\\.dylib$")
set(_copy_cmds "")
foreach(lib IN LISTS _sdk_dylibs)
  list(APPEND _copy_cmds COMMAND ${CMAKE_COMMAND} -E copy_if_different "${lib}" "$<TARGET_FILE_DIR:camera-sidecar>/")
endforeach()
if(_adapter_probe)
  list(GET _adapter_probe 0 _a0)
  get_filename_component(_adapter_dir "${_a0}" DIRECTORY)
  message(STATUS "Sony SDK CrAdapter: ${_adapter_dir}")
  list(APPEND _copy_cmds
    COMMAND ${CMAKE_COMMAND} -E copy_directory "${_adapter_dir}" "$<TARGET_FILE_DIR:camera-sidecar>/CrAdapter"
    COMMAND ${CMAKE_COMMAND} -E copy_directory "${_adapter_dir}" "$<TARGET_FILE_DIR:camera-sidecar>/Contents/Frameworks/CrAdapter")
else()
  message(WARNING "No CrAdapter/*.dylib found in the SDK. Connections will fail with an adaptor/handle-plugin error.")
endif()
add_custom_command(TARGET camera-sidecar POST_BUILD ${_copy_cmds} COMMENT "Staging Sony SDK runtime libraries next to camera-sidecar (build directory only)")
