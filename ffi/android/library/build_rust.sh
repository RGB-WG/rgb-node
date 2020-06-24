#!/usr/bin/env sh
set -eo pipefail

# Update this line accordingly if you are not building *from* x86_64
export PATH=$PATH:$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin

CC="aarch64-linux-android21-clang" CFLAGS="--sysroot=$NDK_HOME/sysroot -I$NDK_HOME/sysroot/usr/include -I$NDK_HOME/sysroot/usr/include/aarch64-linux-android" CXX="aarch64-linux-android21-clang++" CXXFLAGS="$CFLAGS -nostdlib++ -I$NDK_HOME/sources/cxx-stl/llvm-libc++/include" LDFLAGS="--sysroot=$NDK_HOME/platforms/android-21/arch-arm64" cargo build --target=aarch64-linux-android
CC="x86_64-linux-android21-clang" CFLAGS="--sysroot=$NDK_HOME/sysroot -I$NDK_HOME/sysroot/usr/include -I$NDK_HOME/sysroot/usr/include/x86_64-linux-android" CXX="x86_64-linux-android21-clang++" CXXFLAGS="$CFLAGS -nostdlib++ -I$NDK_HOME/sources/cxx-stl/llvm-libc++/include" LDFLAGS="--sysroot=$NDK_HOME/platforms/android-21/arch-x86_64" cargo build --target=x86_64-linux-android
CC="armv7a-linux-androideabi21-clang" CFLAGS="--sysroot=$NDK_HOME/sysroot -I$NDK_HOME/sysroot/usr/include -I$NDK_HOME/sysroot/usr/include/arm-linux-androideabi" CXX="armv7a-linux-androideabi21-clang++" CXXFLAGS="$CFLAGS -nostdlib++ -I$NDK_HOME/sources/cxx-stl/llvm-libc++/include" LDFLAGS="--sysroot=$NDK_HOME/platforms/android-21/arch-arm -L$NDK_HOME/sources/cxx-stl/llvm-libc++/libs/armeabi-v7a" cargo build --target=armv7-linux-androideabi
CC="i686-linux-android21-clang" CFLAGS="--sysroot=$NDK_HOME/sysroot -I$NDK_HOME/sysroot/usr/include -I$NDK_HOME/sysroot/usr/include/i686-linux-android" CXX="i686-linux-android21-clang++" CXXFLAGS="$CFLAGS -nostdlib++ -I$NDK_HOME/sources/cxx-stl/llvm-libc++/include" LDFLAGS="--sysroot=$NDK_HOME/platforms/android-21/arch-x86" cargo build --target=i686-linux-android

mkdir -pv library/src/main/java/org/lnpbp/rgbnode_autogen
swig -java -c++ -package "org.lnpbp.rgbnode_autogen" -outdir library/src/main/java/org/lnpbp/rgbnode_autogen swig.i

mkdir -p library/src/main/jniLibs/arm64-v8a library/src/main/jniLibs/x86_64 library/src/main/jniLibs/armeabi-v7a library/src/main/jniLibs/x86

aarch64-linux-android21-clang++ -shared swig_wrap.cxx -L../../../target/aarch64-linux-android/debug/ -lrgb -o library/src/main/jniLibs/arm64-v8a/librgb_node.so
cp -v ../../../target/aarch64-linux-android/debug/librgb.so library/src/main/jniLibs/arm64-v8a/
cp -v $NDK_HOME/sources/cxx-stl/llvm-libc++/libs/arm64-v8a/libc++_shared.so library/src/main/jniLibs/arm64-v8a/

x86_64-linux-android21-clang++ -shared swig_wrap.cxx -L../../../target/x86_64-linux-android/debug/ -lrgb -o library/src/main/jniLibs/x86_64/librgb_node.so
cp -v ../../../target/x86_64-linux-android/debug/librgb.so library/src/main/jniLibs/x86_64/
cp -v $NDK_HOME/sources/cxx-stl/llvm-libc++/libs/x86_64/libc++_shared.so library/src/main/jniLibs/x86_64/

armv7a-linux-androideabi21-clang++ -shared swig_wrap.cxx -L../../../target/armv7-linux-androideabi/debug/ -lrgb -o library/src/main/jniLibs/armeabi-v7a/librgb_node.so
cp -v ../../../target/armv7-linux-androideabi/debug/librgb.so library/src/main/jniLibs/armeabi-v7a/
cp -v $NDK_HOME/sources/cxx-stl/llvm-libc++/libs/armeabi-v7a/libc++_shared.so library/src/main/jniLibs/armeabi-v7a/

i686-linux-android21-clang++ -shared swig_wrap.cxx -L../../../target/i686-linux-android/debug/ -lrgb -o library/src/main/jniLibs/x86/librgb_node.so
cp -v ../../../target/i686-linux-android/debug/librgb.so library/src/main/jniLibs/x86/
cp -v $NDK_HOME/sources/cxx-stl/llvm-libc++/libs/x86/libc++_shared.so library/src/main/jniLibs/x86/
