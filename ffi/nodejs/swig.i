%module rgb_node
%{
#include "../rgb_node.h"
%}

%typemap(out) CResult (v8::Local<v8::Promise::Resolver> resolver) %{
    resolver = v8::Promise::Resolver::New(args.GetIsolate());

    switch ($1.result) {
        case CResultValue::Ok:
            resolver->Resolve(SWIG_NewPointerObj((new COpaqueStruct(static_cast< const COpaqueStruct& >($1.inner))), SWIGTYPE_p_COpaqueStruct, SWIG_POINTER_OWN |  0 ));
            break;
        case CResultValue::Err:
            resolver->Reject(v8::String::NewFromUtf8(args.GetIsolate(), (const char*) $1.inner.ptr));
            break;
    }

    $result = resolver->GetPromise();
%}

%include "../rgb_node.h"
