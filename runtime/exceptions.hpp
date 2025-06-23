#pragma once
class JSValue;

void js_throw(JSValue v);

#ifndef FEATURE_EXCEPTIONS
JSValue js_throw_noexcept(JSValue v);
extern JSValue exn;
#endif