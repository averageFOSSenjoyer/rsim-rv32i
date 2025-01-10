# rsim-rv32i: A cycle accurate multi-stage RV32i simulator
---
## Demo
Try out the WASM based demo [here](https://averagefossenjoyer.github.io/rsim-rv32i-demo/)

![](./figures/intro.png)

## Getting Started
Install the [rust toolchain](https://rustup.rs/)

### Native
Build the package
```
$ cargo run --release --target [check https://doc.rust-lang.org/nightly/rustc/platform-support.html]
```
Or if you prefer to install it
```
$ cargo install --target [see above]
```

### WASM
Install trunk
```
$ cargo install --locked trunk
```

Serve the wasm
```
$ trunk serve --release
```
