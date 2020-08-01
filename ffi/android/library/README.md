# Building the Android Bindings

* Install bash
* Install rustup
* Install Swig 4.0
* Install cmake (used to build zmq)
* Install the Android SDK and export the env variable `ANDROID_SDK_ROOT` to its base path
* Install the Android NDK (version `20.0.5594570`) and export the env variable `NDK_HOME` to its base path
* Install the four cargo targets:
```
rustup target add aarch64-linux-android x86_64-linux-android armv7-linux-androideabi i686-linux-android
```
* Update your `~/.cargo/config` file to set the correct linker and ar command for each target (expand `<NDK_HOME>` manually):
```
[target.aarch64-linux-android]
ar = "<NDK_HOME>/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android-ar"
linker = "<NDK_HOME>/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android26-clang"

[target.x86_64-linux-android]
ar = "<NDK_HOME>/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android-ar"
linker = "<NDK_HOME>/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android26-clang"

[target.armv7-linux-androideabi]
ar = "<NDK_HOME>/toolchains/llvm/prebuilt/linux-x86_64/bin/arm-linux-androideabi-ar"
linker = "<NDK_HOME>/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi26-clang"

[target.i686-linux-android]
ar = "<NDK_HOME>/toolchains/llvm/prebuilt/linux-x86_64/bin/i686-linux-android-ar"
linker = "<NDK_HOME>/toolchains/llvm/prebuilt/linux-x86_64/bin/i686-linux-android26-clang"
```
* Update the `PATH` in `build_rust.sh` script if you're not building from x86_64
* Run `./gradlew build` (if something fails, manually run the `build_rust.sh` script for a better error report)
* The artifacts (debug and release versions) will be generated in `./library/build/outputs/aar/`
