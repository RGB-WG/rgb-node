# RGB iOS Bindings

In order to include RGB into iPhone, iPad or Mac application, on Mac OS do:

    brew install zmq openssl
    export PKG_CONFIG_ALLOW_CROSS=1
    cd ffi
    cargo lipo --release
    cargo build

Then, add `./taget/universal/release/librgb.a` to your project as an external
framework/library and add `./ffi/rgb_node.h` file as Objective-C bridging header
(see `./ffi/ios/DemoApp` for a sample).

You will also need to add `libzmq.a` as a library dependency. For this you will 
need to do manually compile ZMQ library from sources for iOS target and copy
the resulting library as a dependency. Pls make sure that you are checking out
exactly the same version of the code as used by RGB library.
A good instructions may be found at <http://wiki.zeromq.org/build:iphone>