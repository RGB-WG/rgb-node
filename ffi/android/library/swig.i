%module rgb_node
%{
#include "../../rgb_node.h"
%}

%typemap(jstype) CResult "COpaqueStruct"
%typemap(javaout) CResult {
    return new COpaqueStruct($jnicall, $owner);
}

%typemap(out) CResult %{
    switch ($1.result) {
        case CResultValue::Ok:
            *(COpaqueStruct **)&$result = new COpaqueStruct((const COpaqueStruct &) $1.inner);
            break;
        case CResultValue::Err:
            SWIG_JavaThrowException(jenv, SWIG_JavaRuntimeException, (const char*) $1.inner.ptr);
            break;
    }
%}

%include "../../rgb_node.h"
