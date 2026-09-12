# Aftiktuna Three-d

A version of Aftiktuna that uses the rust library Three-d for graphics.

The main binary is used for running the game as a standalone application.

## WebAssembly

It is also possible to build this library for the web as a WebAssembly module. To test this out locally, you can follow the these steps:

- Install `wasm-pack` and run `wasm-pack build --no-pack -t web` in this directory to build the WebAssembly module and generate the integrating JavaScript.
- Copy files from `./pkg/` to `./web/`.
- Start up a local server by running `python3 -m http.server 8080` in the `./web/` directory.
- Visit the page at `http://localhost:8080/index.html`.
