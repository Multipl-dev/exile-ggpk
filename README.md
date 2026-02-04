# exile-ggpk

Rust library for reading Path of Exile GGPK and Bundle game files.

**Forked from [ggpk-explorer](https://github.com/juddisjudd/ggpk-explorer) by JuddIsJudd**

## Features

- **Classic GGPK format** - Legacy format (pre-3.11.2)
- **Bundle format** - Modern format with Oodle compression (3.11.2+)
- **DAT file parsing** - Game data tables (.dat/.dat64)
- **Hash algorithm support** - FNV1a (legacy) and MurmurHash64A (3.21.2+)

## Usage

```rust
use exile_ggpk::GgpkReader;

// Read classic GGPK file
let reader = GgpkReader::open("Content.ggpk")?;
let file = reader.read_file_by_path("Data/Items.dat")?;
```

## Building

Requires:
- Rust 2021 edition
- C++17 compiler (for ooz decompression library)

```bash
git clone --recurse-submodules https://github.com/Multipl-dev/exile-ggpk.git
cd exile-ggpk
cargo build
```

## Native Dependencies

This library includes the [ooz](https://github.com/zao/ooz) decompression library as a git submodule. The ooz library provides Oodle/Kraken decompression required for Bundle format files.

If you didn't clone with `--recurse-submodules`, run:
```bash
git submodule update --init --recursive
```

## License

GPL-3.0 (inherited from ggpk-explorer)

## Credits

- [JuddIsJudd](https://github.com/juddisjudd) - Original ggpk-explorer
- [zao](https://github.com/zao) - ooz decompression library
- [dat-schema](https://github.com/poe-tool-dev/dat-schema) - Community-maintained DAT schemas
- [LibGGPK3](https://github.com/aianlinb/LibGGPK3) - GGPK format reference
