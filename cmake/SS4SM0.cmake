include(CheckIncludeFile)
# M0 source selection only. All SS4S implementation files stay upstream/unmodified.
# Avoid configuring unrelated SMP, NetCast, audio-device and legacy TV backends.
set(SS4S_ROOT "${CMAKE_SOURCE_DIR}/upstream/ss4s")
add_library(ss4s STATIC)
set(HAS_DLFCN TRUE)
set(HAS_PTHREAD TRUE)
target_include_directories(ss4s PUBLIC "${SS4S_ROOT}/include" PRIVATE "${SS4S_ROOT}/src")
add_subdirectory("${SS4S_ROOT}/modules/interface" "ss4s-interface")
target_link_libraries(ss4s PRIVATE ss4s-module-api)
add_subdirectory("${SS4S_ROOT}/src" "ss4s-core")
target_compile_definitions(ss4s PRIVATE SS4S_LIBRARY_SUFFIX=".so")
add_subdirectory("${SS4S_ROOT}/modules/webos/utils" "ss4s-utils")
set(ndl "${SS4S_ROOT}/modules/webos/ndl/webos5")
add_library(ss4s-module-ndl-webos5 SHARED
  "${ndl}/ndl_module.c" "${ndl}/ndl_player.c" "${ndl}/ndl_video.c" "${ndl}/ndl_audio.c"
  "${ndl}/opus_empty.c" "${ndl}/opus_fix.c"
  "${SS4S_ROOT}/modules/webos/common/aurora_frame_diag.c")
set_target_properties(ss4s-module-ndl-webos5 PROPERTIES PREFIX "" OUTPUT_NAME "ss4s-ndl-webos5" C_VISIBILITY_PRESET hidden INSTALL_RPATH "$ORIGIN")
target_include_directories(ss4s-module-ndl-webos5 PRIVATE "${ndl}" "${SS4S_ROOT}/modules/webos/common" "${SS4S_ROOT}/mocks/include/libndl-media" ${OPUS_INCLUDE_DIRS})
target_compile_definitions(ss4s-module-ndl-webos5 PRIVATE NDL_DIRECTMEDIA_API_VERSION=2 _GNU_SOURCE)
target_compile_options(ss4s-module-ndl-webos5 PRIVATE -Wall -Wextra -Wno-unused-parameter -Wno-attributes)
target_link_libraries(ss4s-module-ndl-webos5 PRIVATE ss4s-module-api ss4s-webos-utils NDL_directmedia ${OPUS_LIBRARIES} pthread dl m)
target_link_options(ss4s-module-ndl-webos5 PRIVATE -Wl,--no-undefined)
install(TARGETS ss4s-module-ndl-webos5 LIBRARY DESTINATION lib)
install(FILES "${CMAKE_SOURCE_DIR}/packaging/ss4s_modules.ini" DESTINATION lib)
