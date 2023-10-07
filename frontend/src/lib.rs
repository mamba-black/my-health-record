// use wasm_bindgen::prelude::wasm_bindgen;

pub mod di;
pub mod domain;
pub mod services;
pub mod ui;

pub mod api {
    tonic::include_proto!("api");
}

// #[wasm_bindgen]
// #[derive(Clone, Debug)]
// pub struct CodeClientConfig {
//     client_id: String,
//     scope: String,
//     ux_mode: String,
//     redirect_uri: String,
//     // state: String,
// }
//
// #[wasm_bindgen]
// impl CodeClientConfig {
//     #[wasm_bindgen(constructor)]
//     pub fn new(
//         client_id: String,
//         scope: String,
//         ux_mode: String,
//         redirect_uri: String,
//     ) -> CodeClientConfig {
//         CodeClientConfig {
//             client_id,
//             scope,
//             ux_mode,
//             redirect_uri,
//         }
//     }
//
//     #[wasm_bindgen(getter)]
//     pub fn client_id(&self) -> String {
//         self.client_id.clone()
//     }
//     #[wasm_bindgen(getter)]
//     pub fn scope(&self) -> String {
//         self.scope.clone()
//     }
//     #[wasm_bindgen(getter)]
//     pub fn ux_mode(&self) -> String {
//         self.ux_mode.clone()
//     }
//     #[wasm_bindgen(getter)]
//     pub fn redirect_uri(&self) -> String {
//         self.redirect_uri.clone()
//     }
// }
//
// #[wasm_bindgen]
// extern "C" {
//     pub type Oauth2;
//
//     // client_id: '937186309482-uc7dm6bc6o6p3disa546hq25n8dbov42.apps.googleusercontent.com',
//     // scope: 'https://www.googleapis.com/auth/calendar.readonly',
//     // ux_mode: 'redirect',
//     // redirect_uri: "https://your.domain/code_callback_endpoint",
//     // state: "YOUR_BINDING_VALUE"
//     #[wasm_bindgen(js_namespace = ["google", "accounts", "oauth2"], js_name = "initCodeClient")]
//     // pub fn initCodeClient(config: CodeClientConfig);
//     pub fn initCodeClient(config: CodeClientConfig) -> CodeClient;
//
//     pub type CodeClient;
//
// }
