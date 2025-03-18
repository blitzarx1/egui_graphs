# Wasm Custom Draw

Basic example which demonstrates the usage of `GraphView` widget in a wasm enviroment.

## prepare

1. Copy this example out to its own folder to use.
2. You will need to have Trunk installed to run this example:

    ```bash
    cargo install --locked trunk
    ```

3. Install the required target with `rustup target add wasm32-unknown-unknown`.

## verify example with local run

```bash
cargo run --release
```

## run in web

1. Run

    ```bash
    trunk serve
    ```

    to build and serve on `http://127.0.0.1:8080`. Trunk will rebuild automatically if you edit the project.
2. Open `http://127.0.0.1:8080/index.html` in a browser.
