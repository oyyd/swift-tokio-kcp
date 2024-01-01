prepare-apple:
	rustup update
	rustup target add aarch64-apple-ios-sim aarch64-apple-ios x86_64-apple-ios x86_64-apple-darwin aarch64-apple-darwin

prepare-android:
	rustup toolchain install stable
	rustup target add x86_64-linux-android
	rustup target add x86_64-unknown-linux-gnu
	rustup target add aarch64-linux-android
	rustup target add armv7-linux-androideabi
	rustup target add i686-linux-android

move-dot-git-and-files:
	cp ./README.md ./output/
	cp ./LICENSE ./output/
	cp -r ./.git ./output/
