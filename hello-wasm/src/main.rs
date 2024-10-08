use utoipa::OpenApi;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::mem;
#[derive(OpenApi)]
#[openapi(
    paths(
        hello_endpoint,
        greet_endpoint
    ),
    components(
        schemas(HelloResponse, GreetResponse)
    )
)]
struct ApiDoc;

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
struct HelloResponse {
    message: String,
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
struct GreetResponse {
    greeting: String,
    name: String,
}

#[utoipa::path(
    get,
    path = "/hello",
    responses(
        (status = 200, description = "Successful response", body = HelloResponse)
    )
)]
fn hello_endpoint() -> String {
    serde_json::to_string(&HelloResponse {
        message: "World1212121".to_string(),
    }).unwrap()
}

#[utoipa::path(
    get,
    path = "/greet/{name}",
    params(
        ("name" = String, Path, description = "Name to greet")
    ),
    responses(
        (status = 200, description = "Successful greeting", body = GreetResponse)
    )
)]
fn greet_endpoint(name: &str) -> String {
    serde_json::to_string(&GreetResponse {
        greeting: "Hello".to_string(),
        name: name.to_string(),
    }).unwrap()
}

fn get_routes() -> String {
    let routes = vec!["/hello", "/greet/{name}"];
    serde_json::to_string(&routes).unwrap()
}

fn handle_request(route: &str, params: &str) -> String {
    let params: HashMap<String, String> = serde_json::from_str(params).unwrap_or_default();
    
    match route {
        "/hello" => hello_endpoint(),
        "/greet/{name}" => {
            let name = params.get("name").cloned().unwrap_or_else(|| "Guest".to_string());
            greet_endpoint(&name)
        },
        _ => serde_json::to_string(&HashMap::from([("error", "Not Found")])).unwrap(),
    }
}

fn get_openapi_spec() -> String {
    serde_json::to_string_pretty(&ApiDoc::openapi()).unwrap()
}

// Exported functions using extern "C"

#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut u8, size: usize) {
    unsafe {
        let _ = Vec::from_raw_parts(ptr, 0, size);
    }
}

#[no_mangle]
pub extern "C" fn get_routes_c() -> *mut u64 {
    let result = get_routes();
    let c_string = CString::new(result).unwrap();
    let ptr = c_string.into_raw();
    let len = unsafe { CStr::from_ptr(ptr) }.to_bytes().len();
    
    let result_with_len = Box::new([ptr as u64, len as u64]);
    Box::into_raw(result_with_len) as *mut u64
}

#[no_mangle]
pub extern "C" fn hello_endpoint_c() -> *mut u64 {
    let result = hello_endpoint();
    let c_string = CString::new(result).unwrap();
    let ptr = c_string.into_raw();
    let len = unsafe { CStr::from_ptr(ptr) }.to_bytes().len();
    
    let result_with_len = Box::new([ptr as u64, len as u64]);
    Box::into_raw(result_with_len) as *mut u64
}

#[no_mangle]
pub extern "C" fn free_result(ptr: *mut u64) {
    if !ptr.is_null() {
        unsafe {
            let box_array = Box::from_raw(ptr as *mut [u64; 2]);
            let [string_ptr, _] = *box_array;
            free_string(string_ptr as *mut c_char);
        }
    }
}
#[no_mangle]
pub extern "C" fn handle_request_c(route: *const c_char, params: *const c_char) -> *mut u64 {
    let route = unsafe { CStr::from_ptr(route).to_str().unwrap() };
    let params = unsafe { CStr::from_ptr(params).to_str().unwrap() };
    let result = handle_request(route, params);
    let c_string = CString::new(result).unwrap();
    let ptr = c_string.into_raw();
    let len = unsafe { CStr::from_ptr(ptr) }.to_bytes().len();
    
    let result_with_len = Box::new([ptr as u64, len as u64]);
    Box::into_raw(result_with_len) as *mut u64
}

#[no_mangle]
pub extern "C" fn get_openapi_spec_c() -> *mut u64 {
    let result = get_openapi_spec();
    let c_string = CString::new(result).unwrap();
    let ptr = c_string.into_raw();
    let len = unsafe { CStr::from_ptr(ptr) }.to_bytes().len();
    
    let result_with_len = Box::new([ptr as u64, len as u64]);
    Box::into_raw(result_with_len) as *mut u64
}

#[no_mangle]
pub extern "C" fn free_string(ptr: *mut c_char) {
    unsafe {
        if ptr.is_null() { return }
        CString::from_raw(ptr)
    };
}

// Main function (optional, depending on your WebAssembly runtime)
fn main() {
    // This main function can be used for initialization if needed
    println!("WebAssembly module initialized {}", handle_request("/hello", "David"));
    // println!("OpenAPI spec: {}", get_openapi_spec());
}
