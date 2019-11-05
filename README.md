# SPV implementation for Tapyrus

This repository is WIP.

# How to Build

## Build Rust library

```
$ cargo build --release
```

## Build for Android

```
$ ./scripts/build-ndk.sh
```

Then you can find Shared Object files in following places.

* `./target/aarch64-linux-android/release/libtapyrus_spv.so`
* `./target/armv7-linux-androideabi/release/libtapyrus_spv.so`
* `./target/i686-linux-android/release/libtapyrus_spv.so`

## Build for iOS

```$xslt
$ rustup target add aarch64-apple-ios armv7-apple-ios armv7s-apple-ios x86_64-apple-ios i386-apple-ios
$ cargo install cargo-lipo
$ cargo lipo --release
```

Then you can find universal iOS library file in

* `./target/universal/release/libtapyrus_spv.a`

# License

Codes in this repository is licensed as MIT License.