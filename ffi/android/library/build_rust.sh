#!/usr/bin/env sh
set -eo pipefail

echo "Starting"

# TODO: every platform

CC="aarch64-linux-android21-clang" CFLAGS="--sysroot=$NDK_HOME/sysroot -I$NDK_HOME/sysroot/usr/include -I$NDK_HOME/sysroot/usr/include/aarch64-linux-android" CXX="aarch64-linux-android21-clang++" CXXFLAGS="$CFLAGS -nostdlib++ -I$NDK_HOME/sources/cxx-stl/llvm-libc++/include" LDFLAGS="--sysroot=$NDK_HOME/platforms/android-21/arch-arm64" cargo build --target=aarch64-linux-android
# CC="x86_64-linux-android21-clang" cargo build --target=x86_64-linux-android
# CC="armv7a-linux-androideabi21-clang" cargo build --target=armv7-linux-androideabi
# CC="i686-linux-android21-clang" cargo build --target=i686-linux-android

swig -java -c++ -package "org.lnpbp.rgbnode" -outdir library/src/main/java/org/lnpbp/rgbnode swig.i

mkdir -p library/src/main/jniLibs/arm64-v8a library/src/main/jniLibs/x86_64 library/src/main/jniLibs/armeabi-v7a library/src/main/jniLibs/x86

aarch64-linux-android21-clang++ -shared swig_wrap.cxx -L../../../target/aarch64-linux-android/debug/ -lffi -o library/src/main/jniLibs/arm64-v8a/librgb_node.so
cp -v ../../../target/aarch64-linux-android/debug/libffi.so library/src/main/jniLibs/arm64-v8a/
cp -v $NDK_HOME/sources/cxx-stl/llvm-libc++/libs/arm64-v8a/libc++_shared.so library/src/main/jniLibs/arm64-v8a/
