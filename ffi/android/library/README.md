# Building the Android Bindings

* Install Swig 4.0
* Install cmake (used to build zmq)
* Install the Android NDK and set the env variable NDK_HOME to the its base path
* Install the four cargo targets:
```
rustup target add aarch64-linux-android x86_64-linux-android armv7-linux-androideabi i686-linux-android
```
* Update your `~/.cargo/config` file to set the correct linker and ar command for each target (expand `NDK_HOME` manually):
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
* Run `./gradlew build`
* The artifacts are in `./library/build/outputs/aar/`