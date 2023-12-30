# uniffi-template

## Build

```bash
# once
make prepare-apple
```

Build release:
```bash
cd bindings
make apple
```

## Import to xcode project

### 1. Add files

Click `Add files to` and add `output` to the project, allow copy.
Or import as local dependencies.

### 2. Import Local Package

Click `Add package dependencies` and `Add local` and select local `output` folder.

Use in code:

```swift
import Bindings

var result = rustGreeting("hello")
```

## TODO

- [] 支持 build release
- [x] 移除 app 代码
- [?] 支持修改 xcframework 的名称
  - 先统一修改成 Bindings

需要支持两种 build mode:
1. prod
2. dev: 需要满足开发时快速改动，只 build 必要的 target
