# SwiftTokioKcp

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
git tag x.x.x
git push origin x.x.x
```

## TODO
- doc: tips to start a server
