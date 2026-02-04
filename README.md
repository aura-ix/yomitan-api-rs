# yomitan-api-rs
An implementation of the native messaging component and it's installer for yomitan. Functions identically to [the official yomitan-api](github.com/yomidevs/yomitan-api), but removes the python dependency.

# Installation
Download a binary from [here](https://github.com/aura-ix/yomitan-api-rs/releases) for Windows and macOS users. For linux users, it is suggested to build it yourself (`cargo build --release; ./target/release/installer`).

**macOS users**: Make sure to read the README.txt contained in the downloaded zip file!

# Tested platforms
- Firefox, macOS 26.2
- Firefox, Windows 11

Please open an issue or pull request if you have successfully installed it on another platform!

## TODO
- Support for custom installation parameters
- Test on all supported browsers/OS pairs
- Uninstallation