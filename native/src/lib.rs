mod compiler;

extern crate log;

#[macro_use]
extern crate neon;

extern crate simplelog;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use log::{debug, info};

use neon::prelude::*;

use simplelog::*;

static mut PROGRAM_DATA: Option<ProgramData> = None;

struct ProgramData {
    file_data: HashMap<usize, FileData>,
}

impl ProgramData {
    fn new() -> ProgramData {
        ProgramData {
            file_data: HashMap::new(),
        }
    }

    fn get_next_handle(&self) -> usize {
        self.file_data.len()
    }

    fn set_file_data(&mut self, handle: usize, file_data: FileData) {
        if self.file_data.contains_key(&handle) {
            panic!(format!("Key already exists: {}", handle));
        }

        self.file_data.insert(handle, file_data);
    }
}

struct FileData {
    buffer: Vec<u8>,
}

// Get rid of 'undefined function __cxa_pure_virtual' error.
// See: https://users.rust-lang.org/t/neon-electron-undefined-symbol-cxa-pure-virtual/21223
#[no_mangle]
pub extern "C" fn __cxa_pure_virtual() {
    loop {}
}

fn get_global_program_data() -> &'static mut ProgramData {
    let ret;
    unsafe { ret = PROGRAM_DATA.get_or_insert_with(ProgramData::new) }
    ret
}

fn read_file(filename: &str) -> FileData {
    let mut f = File::open(filename).expect(&format!("File not found: {}", filename));
    let mut buffer = Vec::new();

    f.read_to_end(&mut buffer)
        .expect(&format!("Error reading file: {}", filename));

    FileData { buffer }
}

fn open_file(mut cx: FunctionContext) -> JsResult<JsNumber> {
    info!("Open file");

    // First argument is filename as a string
    let filename = cx.argument::<JsString>(0)?.value();
    debug!("Filename: {}", filename);

    let program_data = get_global_program_data();

    let handle = program_data.get_next_handle();

    let file_data = read_file(&filename);

    program_data.set_file_data(handle, file_data);

    Ok(cx.number(handle as f64))
}

fn get_binary_data(mut cx: FunctionContext) -> JsResult<JsArray> {
    info!("Get binary data");

    let handle = cx.argument::<JsNumber>(0)?.value() as usize;
    debug!("Handle: {}", handle);

    let num_elems = cx.argument::<JsNumber>(1)?.value() as usize;
    debug!("Num elements: {}", num_elems);

    let program_data = get_global_program_data();
    let file_data = program_data.file_data.get(&handle).unwrap();

    let num_elems = std::cmp::min(num_elems, file_data.buffer.len());

    let js_array = JsArray::new(&mut cx, num_elems as u32);

    for i in 0..num_elems {
        let js_number = cx.number(file_data.buffer[i] as f64);
        js_array.set(&mut cx, i as u32, js_number).unwrap();
    }

    Ok(js_array)
}

fn init_logging() -> Result<(), log::SetLoggerError> {
    WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        File::create("carta-backend.log").unwrap(),
    )
}

fn init(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let res = init_logging();

    if res.is_err() {
        let _: JsResult<JsError> = cx.throw_error("File not found");
    }

    info!("Init complete");

    Ok(cx.undefined())
}

fn compile_schema_file(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    compiler::compile_schema_file("schema.carta");
    Ok(cx.undefined())
}

register_module!(mut cx, {
    cx.export_function("init", init)?;
    cx.export_function("openFile", open_file)?;
    cx.export_function("getBinaryData", get_binary_data)?;
    cx.export_function("compileSchemaFile", compile_schema_file)?;
    Ok(())
});

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_init_logging() {
        let r = init_logging();
        assert!(r.is_ok());
        info!("Logging works");
    }
}
