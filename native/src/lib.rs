#[macro_use]
extern crate neon;

use neon::prelude::*;

static mut COUNTER: usize = 0;

// Get rid of 'undefined function __cxa_pure_virtual' error.
// See: https://users.rust-lang.org/t/neon-electron-undefined-symbol-cxa-pure-virtual/21223
#[no_mangle]
pub extern fn __cxa_pure_virtual() {
    loop{};
}

fn open_file(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // First argument is filename as a string
    let filename = cx.argument::<JsString>(0)?.value();

    let _: JsResult<JsError> = cx.throw_error("File not found");

    let ret = format!("Opening file: {}", filename);
    println!("Open file: {}", filename);
    unsafe {
        COUNTER += 1;
        Ok(cx.number(COUNTER as f64)) 
    }
}

register_module!(mut cx, {
    cx.export_function("openFile", open_file)?;
    Ok(())
});
