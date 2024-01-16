use anyhow::Result;
use std::env;
use std::fs;
use std::path;
use std::path::PathBuf;
use std::process;
use std::process::Stdio;

const TARGET_X64_IOS: &str = "x86_64-apple-ios";
const TARGET_AARCH64_IOS_SIM: &str = "aarch64-apple-ios-sim";
const TARGET_AARCH64_IOS: &str = "aarch64-apple-ios";
const TARGET_AARCH64_DARWIN: &str = "aarch64-apple-darwin";
const TARGET_X64_DARWIN: &str = "x86_64-apple-darwin";

const STATIC_LIB_NAME: &str = "libbindings.a";

fn all_targets() -> &'static [&'static str] {
  &[
    TARGET_X64_IOS,
    TARGET_AARCH64_IOS_SIM,
    TARGET_AARCH64_IOS,
    TARGET_AARCH64_DARWIN,
    TARGET_X64_DARWIN,
  ]
}

struct Builder {
  release: bool,
  workspace_path: path::PathBuf,
}

fn run(cmd_str: &str, cwd: Option<PathBuf>) -> Result<()> {
  println!("[builder] running: {}, cwd: {:?}", cmd_str, cwd);

  let parts: Vec<_> = cmd_str.split(" ").collect();

  let mut cmd = process::Command::new(parts[0]);
  if cwd.is_some() {
    cmd.current_dir(cwd.unwrap());
  }

  for i in 1..parts.len() {
    let part = parts[i];
    cmd.arg(part);
  }

  // pipe stdout/stderr
  cmd.stderr(Stdio::inherit());
  cmd.stdout(Stdio::inherit());

  let mut child = cmd.spawn()?;

  let res = child.wait()?;
  if !res.success() {
    return Err(anyhow::format_err!(
      "[builder] command failed, cmd: {}",
      cmd_str
    ));
  }

  Ok(())
}

impl Builder {
  fn build_directory(&self) -> &str {
    match self.release {
      true => "release",
      false => "debug",
    }
  }

  fn bindings_path(&self) -> PathBuf {
    let bindings_path = self.workspace_path.join("./bindings");
    bindings_path
  }

  fn output_folder_name(&self) -> &str {
    "generated"
  }

  fn output_path(&self) -> PathBuf {
    let output_path = self
      .workspace_path
      .join(format!("./{}", self.output_folder_name()));
    output_path
  }

  fn target_path(&self) -> PathBuf {
    let output_path = self.workspace_path.join("./target");
    output_path
  }

  // steps
  fn remove_output(&self) -> Result<()> {
    run(
      &format!(
        "rm -rf {}",
        self.output_path().to_string_lossy().to_string()
      ),
      None,
    )?;

    Ok(())
  }

  fn build_targets(&self) -> Result<()> {
    // - build targets
    for target in all_targets() {
      // dev(fast): cargo build --lib --target aarch64-apple-ios-sim
      // release(small): cargo +nightly build -Z build-std --release --lib --target aarch64-apple-ios-sim
      let cmd_str = match self.release {
        true => {
          format!(
            "cargo +nightly build -Z build-std --release --lib --target {}",
            target
          )
        }
        false => {
          format!("cargo build --lib --target {}", target)
        }
      };

      run(&cmd_str, Some(self.bindings_path()))?;
    }

    Ok(())
  }

  fn bindgen_swift(&self) -> Result<()> {
    let build_cmd = match self.release {
      true => "cargo build --release",
      false => "cargo build",
    };
    run(&build_cmd, Some(self.bindings_path()))?;

    // cargo run --release -p uniffi-bindgen generate --language swift --lib-file $(TARGET_DIR)/release/libbindings.dylib src/bindings.udl
    let cmd_str = format!("cargo run --release -p uniffi-bindgen generate --language swift --lib-file {}/{}/libbindings.dylib src/bindings.udl", self.target_path().to_string_lossy().to_string(), self.build_directory());
    run(&cmd_str, Some(self.bindings_path()))?;

    // sed -i '' 's/module\ BindingsFFI/framework\ module\ BindingsFFI/' src/BindingsFFI.modulemap
    let modulemap_p = self.bindings_path().join("./src/BindingsFFI.modulemap");

    let content = fs::read(modulemap_p.clone())?;
    let content = String::from_utf8(content)?;
    let content = content.replace("module BindingsFFI", "framework module BindingsFFI");

    fs::write(modulemap_p, content)?;

    Ok(())
  }

  fn assemble_frameworks(&self) -> Result<()> {
    for target in all_targets() {
      // cd $(TARGET_DIR)/x86_64-apple-ios/release && mkdir -p BindingsFFI.framework && cd BindingsFFI.framework
      let this_target_p =
        self
          .target_path()
          .join(format!("./{}/{}", target, self.build_directory()));

      run("rm -rf BindingsFFI.framework", Some(this_target_p.clone()))?;

      run(
        "mkdir -p BindingsFFI.framework",
        Some(this_target_p.clone()),
      )?;

      // mkdir Headers Modules Resources
      let this_target_framework_p = this_target_p.join("./BindingsFFI.framework");

      run(
        "mkdir Headers Modules Resources",
        Some(this_target_framework_p.clone()),
      )?;

      // cp ../../../../bindings/src/BindingsFFI.modulemap ./Modules/module.modulemap
      let bindings_p_str = self.bindings_path().to_string_lossy().to_string();
      run(
        &format!(
          "cp {}/src/BindingsFFI.modulemap ./Modules/module.modulemap",
          bindings_p_str
        ),
        Some(this_target_framework_p.clone()),
      )?;

      // cp ../../../../bindings/src/BindingsFFI.h ./Headers/BindingsFFI.h
      run(
        &format!(
          "cp {}/src/BindingsFFI.h ./Headers/BindingsFFI.h",
          bindings_p_str
        ),
        Some(this_target_framework_p.clone()),
      )?;

      // cp ../$(STATIC_LIB_NAME) ./BindingsFFI
      run(
        &format!(
          "cp {}/{} ./BindingsFFI",
          this_target_p.to_string_lossy().to_string(),
          STATIC_LIB_NAME,
        ),
        Some(this_target_framework_p.clone()),
      )?;

      // cp ../../../../misc/apple/Info.plist ./Resources
      run(
        "cp ../../../../misc/apple/Info.plist ./Resources",
        Some(this_target_framework_p.clone()),
      )?;
    }

    Ok(())
  }

  fn xcframeowrk(&self) -> Result<()> {
    // lipo used to create universal(aarch64 + x86_64) files

    let dir = self.build_directory();

    // lipo -create x86_64-apple-ios/release/BindingsFFI.framework/BindingsFFI \
    //   aarch64-apple-ios-sim/release/BindingsFFI.framework/BindingsFFI \
    //   -output aarch64-apple-ios-sim/release/BindingsFFI.framework/BindingsFFI
    let cmd_str = format!("lipo -create {}/{}/BindingsFFI.framework/BindingsFFI {}/{}/BindingsFFI.framework/BindingsFFI -output {}/{}/BindingsFFI.framework/BindingsFFI", TARGET_X64_IOS, dir, TARGET_AARCH64_IOS_SIM, dir, TARGET_AARCH64_IOS_SIM, dir);
    run(&cmd_str, Some(self.target_path()))?;

    // lipo -create aarch64-apple-darwin/release/BindingsFFI.framework/BindingsFFI \
    //   x86_64-apple-darwin/release/BindingsFFI.framework/BindingsFFI \
    //   -output aarch64-apple-darwin/release/BindingsFFI.framework/BindingsFFI
    let cmd_str = format!("lipo -create {}/{}/BindingsFFI.framework/BindingsFFI {}/{}/BindingsFFI.framework/BindingsFFI -output {}/{}/BindingsFFI.framework/BindingsFFI", TARGET_AARCH64_DARWIN, dir, TARGET_X64_DARWIN, dir, TARGET_AARCH64_DARWIN, dir);
    run(&cmd_str, Some(self.target_path()))?;

    // rm -rf BindingsFFI.xcframework || echo "skip removing"
    run("rm -rf BindingsFFI.xcframework", Some(self.target_path()))?;

    // xcodebuild -create-xcframework \
    //   -framework aarch64-apple-ios/release/BindingsFFI.framework \
    //   -framework aarch64-apple-ios-sim/release/BindingsFFI.framework \
    //   -framework aarch64-apple-darwin/release/BindingsFFI.framework \
    //   -output BindingsFFI.xcframework
    let cmd_str = format!("xcodebuild -create-xcframework -framework {}/{}/BindingsFFI.framework -framework {}/{}/BindingsFFI.framework -framework {}/{}/BindingsFFI.framework -output BindingsFFI.xcframework", TARGET_AARCH64_IOS, dir, TARGET_AARCH64_IOS_SIM, dir, TARGET_AARCH64_DARWIN, dir);
    run(&cmd_str, Some(self.target_path()))?;

    Ok(())
  }

  fn cp_xcframeowrk_source(&self) -> Result<()> {
    // mkdir -p output/Sources/Bindings
    //   cp -r target/BindingsFFI.xcframework output/Sources
    //   cp bindings/src/Bindings.swift output/Sources/Bindings
    //   cp misc/apple/Package.swift output/
    let output = self.output_folder_name();

    run(
      &format!("mkdir -p {}/Sources/Bindings", output),
      Some(self.workspace_path.clone()),
    )?;
    run(
      &format!("cp -r target/BindingsFFI.xcframework {}/Sources", output),
      Some(self.workspace_path.clone()),
    )?;
    run(
      &format!("cp bindings/src/Bindings.swift {}/Sources/Bindings", output),
      Some(self.workspace_path.clone()),
    )?;
    // run(
    //   &format!("cp misc/apple/Package.swift {}", output),
    //   Some(self.workspace_path.clone()),
    // )?;

    Ok(())
  }

  fn run(&self) -> Result<()> {
    self.remove_output()?;

    self.build_targets()?;

    self.bindgen_swift()?;

    self.assemble_frameworks()?;

    self.xcframeowrk()?;

    self.cp_xcframeowrk_source()?;

    Ok(())
  }
}

fn copy_generated(generated_sources: PathBuf) -> Result<()> {
  run(
    "rm -rf ../../output/Sources/Bindings",
    Some(generated_sources.clone()),
  )?;
  run(
    "rm -rf ../../output/Sources/BindingsFFI.xcframework",
    Some(generated_sources.clone()),
  )?;

  run(
    "cp -r Bindings ../../output/Sources",
    Some(generated_sources.clone()),
  )?;

  run(
    "cp -r BindingsFFI.xcframework ../../output/Sources",
    Some(generated_sources.clone()),
  )?;

  Ok(())
}

fn main() {
  let manifest_path = path::Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).to_owned();
  let workspace_path = manifest_path.join("..");

  let mut builder = Builder {
    release: false,
    workspace_path,
  };

  for i in env::args() {
    let release_arg = "--release";
    if i.starts_with(release_arg) {
      builder.release = true
    }
  }

  builder.run().unwrap();

  let generated_sources = builder.workspace_path.join("./generated/Sources");
  copy_generated(generated_sources).unwrap();
}
