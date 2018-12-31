#[macro_use]
extern crate neon;

use neon::prelude::*;

// https://users.rust-lang.org/t/neon-electron-undefined-symbol-cxa-pure-virtual/21223
#[no_mangle]
pub extern fn __cxa_pure_virtual() {
    loop{};
}

fn open_file(mut cx: FunctionContext) -> JsResult<JsString> {
    // First argument is filename as a string
    let filename = cx.argument::<JsString>(0)?.value();

    panic!("File not found!");

    let ret = format!("Opening file: {}", filename);
    Ok(cx.string(ret)) 
}

register_module!(mut cx, {
    cx.export_function("openFile", open_file)?;
    Ok(())
});
