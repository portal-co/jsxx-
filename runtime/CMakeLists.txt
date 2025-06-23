add_library(JS STATIC ${CMAKE_CURRENT_SOURCE_DIR}/exceptions.cpp ${CMAKE_CURRENT_SOURCE_DIR}/global_io.cpp ${CMAKE_CURRENT_SOURCE_DIR}/global_json.cpp ${CMAKE_CURRENT_SOURCE_DIR}/global_symbol.cpp ${CMAKE_CURRENT_SOURCE_DIR}/js_primitives.cpp ${CMAKE_CURRENT_SOURCE_DIR}/js_value.cpp)
target_include_directories(JS PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})
target_compile_options(JS PUBLIC -fcoroutines)
target_compile_features(JS PUBLIC cxx_std_20)