use data::{affix::Affix, settings::Settings};
use optimizer_core::{start, start_with_heuristics};

use wasm_bindgen::prelude::*;
use web_sys::console;

// public so that the benches can access it
pub mod data;
pub mod optimizer_core;
mod result;
mod utils;

fn parse_args(
    js_chunks: String,
    js_combinations: String,
) -> Option<(Vec<Vec<Affix>>, Vec<Settings>)> {
    let opt_chunks = utils::parse_string_to_vector(&js_chunks);
    let chunks = match opt_chunks {
        Some(chunks) => chunks,
        None => {
            console::log_1(&JsValue::from_str("Error parsing chunks"));
            return None;
        }
    };
    let chunks = utils::vec_i8_to_affix(chunks);
    let combinations: Vec<Settings> = serde_json::from_str(&js_combinations).unwrap();

    Some((chunks, combinations))
}

/// entry point from JS
///
/// # Arguments
/// - `js_chunks` - a stringified JSON array of arrays of i8
/// - `js_combinations` - a stringified JSON array of combination objects
///
#[wasm_bindgen]
pub fn calculate(js_chunks: String, js_combinations: String) -> Option<String> {
    let args = parse_args(js_chunks, js_combinations);
    let (chunks, combinations) = match args {
        Some(args) => args,
        None => return None,
    };

    // needed to post messages to js. this is done via the global scope
    // since we might also want to execute the core on another target (x86, ... ) where wasm-bindgen
    // is not available, we pass through an optional global scope
    let workerglobal = js_sys::global()
        .dyn_into::<web_sys::DedicatedWorkerGlobalScope>()
        .unwrap();

    // receive messages from js
    // let on_message_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
    //     let data = event.data();
    //     console::log_1(&data);
    // }) as Box<dyn FnMut(MessageEvent)>);

    // workerglobal.set_onmessage(Some(on_message_callback.as_ref().unchecked_ref()));

    // calculate the result (maxResult best characters) for the given chunks
    let mut result = start(&chunks, &combinations, Some(&workerglobal));
    result.on_complete(&combinations);

    // parse to string
    let result_str = serde_json::to_string(&result.best_characters);

    match result_str {
        Ok(result_str) => return Some(result_str),
        Err(_) => None,
    }
}

#[wasm_bindgen]
pub fn calculate_with_heuristics(js_chunks: String, js_combinations: String) -> Option<String> {
    let args = parse_args(js_chunks, js_combinations);
    let (chunks, combinations) = match args {
        Some(args) => args,
        None => return None,
    };

    let workerglobal = js_sys::global()
        .dyn_into::<web_sys::DedicatedWorkerGlobalScope>()
        .unwrap();

    let mut result = start_with_heuristics(&chunks, &combinations, Some(&workerglobal));
    result.on_complete(&combinations);

    // parse to string
    let result_str = serde_json::to_string(&result.best_characters);

    match result_str {
        Ok(result_str) => return Some(result_str),
        Err(_) => None,
    }
}
