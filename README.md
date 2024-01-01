# SwiftTokioKcp

// TODO add 9.6MB note

## Example

## Building

### Prerequisite

Setup [rust](https://rustup.rs/).

### Build Binaries

> uniffi-rs setting up is modified from repo: https://github.com/imWildCat/uniffi-rs-fullstack-examples

```bash
make prepare-apple
cd bindings && make apple
```

Binaries will be generated into `output` folder.

## Publish Version

After finishing build, ensure all necesary files are incldued in `output` folder, then:

```bash
make move-dot-git-and-files
cd output
git commit -m "x.x.x"
git tag x.x.x
git push origin x.x.x
```

Version tags contain only files to be , i.e. Rust project files won't be included. Therefore never commit and merge publish changes in `main` branch.

## TODO
- doc: tips to start a server
