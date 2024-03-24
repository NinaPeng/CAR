mod handler;
mod no_op_handler;
mod record_batch_vector_reader;
use anyhow::Result;
use arrow::ffi_stream::{ArrowArrayStreamReader, FFI_ArrowArrayStream};
use handler::DynHandler;

#[cxx::bridge(namespace = "car")]
mod ffi {
    extern "Rust" {
        type DynHandler;
        fn create_handler(handler_name: &str) -> Result<Box<DynHandler>>;
        fn get_handler_name(handler: &Box<DynHandler>) -> String;
    }
}

fn create_handler(handler_name: &str) -> Result<Box<DynHandler>> {
    match handler_name {
        "NoOp" => Ok(Box::new(Box::new(no_op_handler::NoOpHandler::new()))),
        _ => Err(anyhow::anyhow!("Unknown handler: {}", handler_name)),
    }
}

fn get_handler_name(handler: &Box<DynHandler>) -> String {
    (**handler).get_name()
}

#[no_mangle]
pub extern "C" fn process(
    handler: &mut Box<DynHandler>,
    arrow_array_stream: *mut FFI_ArrowArrayStream,
) -> Box<FFI_ArrowArrayStream> {
    let stream = unsafe { FFI_ArrowArrayStream::from_raw(arrow_array_stream) };
    let stream_reader = ArrowArrayStreamReader::try_new(stream).unwrap();
    let reader = (**handler).process(Box::new(stream_reader));
    return Box::new(FFI_ArrowArrayStream::new(reader));
}

// free arrow stream
#[no_mangle]
pub extern "C" fn free_arrow_stream(stream: *mut FFI_ArrowArrayStream) {
    unsafe {
        if stream.is_null() {
            return;
        }
        (*stream).release.unwrap()(stream);
    }
}
