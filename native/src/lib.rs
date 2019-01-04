#[macro_use]
extern crate neon;

extern crate simplelog;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use neon::prelude::*;

use simplelog::*;

static mut PROGRAM_DATA: Option<ProgramData> = None;

struct ProgramData {
    file_data: HashMap<usize, FileData>,
}

impl ProgramData {
    fn new() -> ProgramData {
        ProgramData { file_data: HashMap::new() }
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
    buffer: Vec<u8>
}

// Get rid of 'undefined function __cxa_pure_virtual' error.
// See: https://users.rust-lang.org/t/neon-electron-undefined-symbol-cxa-pure-virtual/21223
#[no_mangle]
pub extern fn __cxa_pure_virtual() {
    loop{};
}

fn get_program_data() -> &'static mut ProgramData {
    let ret;
    unsafe {
        ret = PROGRAM_DATA.get_or_insert_with(ProgramData::new)
    }
    ret
}

fn read_file(filename: &str) -> FileData {
    let mut f = File::open(filename).expect(&format!("File not found: {}", filename));
    let mut buffer = Vec::new();

    f.read_to_end(&mut buffer).expect(&format!("Error reading file: {}", filename));

    FileData { buffer }
}

fn open_file(mut cx: FunctionContext) -> JsResult<JsNumber> {

    // First argument is filename as a string
    let filename = cx.argument::<JsString>(0)?.value();

    let program_data = get_program_data();

    let handle = program_data.get_next_handle();

    let file_data = read_file(&filename);

    program_data.set_file_data(handle, file_data);

    Ok(cx.number(handle as f64))
}

fn init(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let res = WriteLogger::init(LevelFilter::Debug, Config::default(), File::create("carta-backend.log").unwrap());

    if let Err(res) = res {
        cx.throw_error("File not found");
    }

    Ok(cx.undefined())
}

register_module!(mut cx, {
    cx.export_function("openFile", open_file)?;
    Ok(())
});
