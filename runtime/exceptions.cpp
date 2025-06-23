#include "js_value.hpp"

#include <exception>

#ifdef FEATURE_EXCEPTIONS
void js_throw(JSValue v) { throw v; }
#else
void js_throw(JSValue v) { std::terminate(); }
JSValue js_throw_noexcept(JSValue v){
    v.thrown = true;
    return v;
}
JSValue exn;
#endif
