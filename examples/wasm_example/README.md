# Basic
Basic example which demonstrates the usage of `GraphView` widget in a wasm enviroment.
You will need to have Trunk installed to run this example.

Copy this example out to is own folder to use.

1. Install the required target with `rustup target add wasm32-unknown-unknown`.
2. Install Trunk with cargo install --locked trunk.
3. Run `trunk serve` to build and serve on `http://127.0.0.1:8080`. Trunk will rebuild automatically if you edit the project.
4. Open `http://127.0.0.1:8080/index.html` in a browser.

## run
```bash
cargo run --release
```
## run in local web
```bash
trunk serve
```