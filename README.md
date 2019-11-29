## opfs 
A simple utility for manipulating xv6-riscv file system images

this is a partial re-implementation of [titech-os/opfs](https://github.com/titech-os/opfs) by Rust.

### Installation
#### System requirements
+ Rust stable 1.38.0 or later is recommended.
+ [`libc`](https://rust-lang.github.io/libc/#platform-specific-documentation) support is required.

#### How to build
```sh
cd opfs
cargo build --release
```

`target/release/opfs` is a target binary. 

#### Run
Another way to run:
```sh
cargo run --release
```
This command builds project then run the product.
The produced binary will be placed in `target/release` as same.
If you use this, options should be written after `--`.
e.g.) `cargo run --release -- ./fs.img ls /`

### Usage
```
opfs img_file command [args]
```
`img_file` is a path to image file to manipulate.

currently no command is implemented.
