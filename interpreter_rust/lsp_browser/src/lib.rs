#![deny(unsafe_code)]

use futures::stream::TryStreamExt;
use lsp::backend::Backend;
use tower_lsp::{LspService, Server};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::stream::JsStream;

#[wasm_bindgen]
pub struct ServerConfig {
    into_server: js_sys::AsyncIterator,
    from_server: web_sys::WritableStream,
}

#[wasm_bindgen]
impl ServerConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(into_server: js_sys::AsyncIterator, from_server: web_sys::WritableStream) -> Self {
        Self {
            into_server,
            from_server,
        }
    }
}

#[wasm_bindgen]
pub async fn serve(config: ServerConfig) -> Result<(), JsValue> {
    let ServerConfig {
        into_server,
        from_server,
    } = config;

    let input = JsStream::from(into_server);
    let input = input
        .map_ok(|value| {
            value
                .dyn_into::<js_sys::Uint8Array>()
                .expect("could not cast stream item to Uint8Array")
                .to_vec()
        })
        .map_err(|_err| std::io::Error::from(std::io::ErrorKind::Other))
        .into_async_read();

    let output = JsCast::unchecked_into::<wasm_streams::writable::sys::WritableStream>(from_server);
    let output = wasm_streams::WritableStream::from_raw(output);
    let output = output.try_into_async_write().map_err(|err| err.0)?;

    let (service, messages) = LspService::new(Backend::new);
    Server::new(input, output, messages).serve(service).await;

    Ok(())
}
