# iroh browser demo

Run this yourself:
```sh
$ cargo install wasm-bindgen-cli
$ rustup target install wasm32-unknown-unknown
$ npm i
$ npm run build
$ npm run serve
```

Then:
- Go to https://localhost:8080
- Open DevTools
- Look at the `window.wasm` object.
- Run `accept = wasm.accept()`. This will print a NodeId in the logs, copy that node ID.
- Run `await wasm.connect("<node ID>")`.
- Watch the logs - voila!
- If there's errors with connect, they will be returned from the above call.
- If there's errors with accept, run `await accept` to take a look.

