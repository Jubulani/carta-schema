#[macro_use]
extern crate neon;

use neon::prelude::*;

// https://users.rust-lang.org/t/neon-electron-undefined-symbol-cxa-pure-virtual/21223
#[no_mangle]
pub extern fn __cxa_pure_virtual() {
    loop{};
}

fn thread_count(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(cx.number(num_cpus::get() as f64))
}

register_module!(mut cx, {
    cx.export_function("threadCount", thread_count)
});
