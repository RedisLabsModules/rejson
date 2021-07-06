use libc::size_t;
use std::ffi::CString;
use std::os::raw::{c_double, c_int, c_long};
use std::ptr::{null, null_mut};
use std::{
    ffi::CStr,
    os::raw::{c_char, c_void},
};

use crate::commands::KeyValue;
use jsonpath_lib::select::select_value::{SelectValue, SelectValueType};
use jsonpath_lib::select::Selector;
use redis_module::key::verify_type;
use redis_module::{raw as rawmod, RedisError};
use redis_module::{Context, RedisString, Status};
use serde_json::Value;

use crate::manager::{Manager, ReadHolder, RedisJsonKeyManager};
use crate::{redisjson::RedisJSON, REDIS_JSON_TYPE};

// extern crate readies_wd40;
// use crate::readies_wd40::{BB, _BB, getenv};

//
// structs
//

#[repr(C)]
pub enum JSONType {
    String = 0,
    Int = 1,
    Double = 2,
    Bool = 3,
    Object = 4,
    Array = 5,
    Null = 6,
}

struct ResultsIterator<'a, V: SelectValue> {
    results: Vec<&'a V>,
    pos: usize,
}

//---------------------------------------------------------------------------------------------

pub fn create_rmstring(
    ctx: *mut rawmod::RedisModuleCtx,
    from_str: &str,
    str: *mut *mut rawmod::RedisModuleString,
) -> c_int {
    if let Ok(s) = CString::new(from_str) {
        let p = s.as_bytes_with_nul().as_ptr() as *const c_char;
        let len = s.as_bytes().len();
        unsafe { *str = rawmod::RedisModule_CreateString.unwrap()(ctx, p, len) };
        return Status::Ok as c_int;
    }
    Status::Err as c_int
}

fn json_api_open_key_internal<M: Manager>(
    manager: M,
    ctx: *mut rawmod::RedisModuleCtx,
    key: RedisString,
) -> *const M::V {
    let ctx = Context::new(ctx);
    if let Ok(h) = manager.open_key_read(&ctx, &key) {
        if let Ok(v) = h.get_value() {
            if let Some(v) = v {
                return v;
            }
        }
    }
    null()
}

#[no_mangle]
pub extern "C" fn JSONAPI_openKey(
    ctx: *mut rawmod::RedisModuleCtx,
    key_str: *mut rawmod::RedisModuleString,
) -> *mut c_void {
    json_api_open_key_internal(RedisJsonKeyManager, ctx, RedisString::new(ctx, key_str))
        as *mut c_void
}

#[no_mangle]
pub extern "C" fn JSONAPI_openKeyFromStr(
    ctx: *mut rawmod::RedisModuleCtx,
    path: *const c_char,
) -> *mut c_void {
    let key = unsafe { CStr::from_ptr(path).to_str().unwrap() };
    json_api_open_key_internal(RedisJsonKeyManager, ctx, RedisString::create(ctx, key))
        as *mut c_void
}

fn json_api_get_at<M: Manager>(_: M, json: *const c_void, index: size_t) -> *const c_void {
    let json = unsafe { &*(json as *const M::V) };
    match json.get_type() {
        SelectValueType::Array => match json.get_index(index) {
            Some(v) => v as *const M::V as *const c_void,
            _ => null(),
        },
        _ => null(),
    }
}

#[no_mangle]
pub extern "C" fn JSONAPI_getAt(json: *const c_void, index: size_t) -> *const c_void {
    json_api_get_at(RedisJsonKeyManager, json, index)
}

fn json_api_get_len<M: Manager>(_: M, json: *const c_void, count: *mut libc::size_t) -> c_int {
    let json = unsafe { &*(json as *const M::V) };
    let len = match json.get_type() {
        SelectValueType::String => Some(json.get_str().len()),
        SelectValueType::Array => Some(json.len().unwrap()),
        SelectValueType::Object => Some(json.len().unwrap()),
        _ => None,
    };
    match len {
        Some(l) => {
            unsafe { *count = l };
            Status::Ok as c_int
        }
        None => Status::Err as c_int,
    }
}

#[no_mangle]
pub extern "C" fn JSONAPI_getLen(json: *const c_void, count: *mut size_t) -> c_int {
    json_api_get_len(RedisJsonKeyManager, json, count)
}

fn json_api_get_type<M: Manager>(_: M, json: *const c_void) -> c_int {
    json_api_get_type_internal(unsafe { &*(json as *const M::V) }) as c_int
}

#[no_mangle]
pub extern "C" fn JSONAPI_getType(json: *const c_void) -> c_int {
    json_api_get_type(RedisJsonKeyManager, json)
}

fn json_api_get_string<M: Manager>(
    _: M,
    json: *const c_void,
    str: *mut *const c_char,
    len: *mut size_t,
) -> c_int {
    let json = unsafe { &*(json as *const M::V) };
    match json.get_type() {
        SelectValueType::String => {
            let s = json.as_str();
            set_string(s, str, len);
            Status::Ok as c_int
        }
        _ => Status::Err as c_int,
    }
}

#[no_mangle]
pub extern "C" fn JSONAPI_getString(
    json: *const c_void,
    str: *mut *const c_char,
    len: *mut size_t,
) -> c_int {
    json_api_get_string(RedisJsonKeyManager, json, str, len)
}

fn json_api_get_json<M: Manager>(
    _: M,
    json: *const c_void,
    ctx: *mut rawmod::RedisModuleCtx,
    str: *mut *mut rawmod::RedisModuleString,
) -> c_int {
    let json = unsafe { &*(json as *const M::V) };
    let res = KeyValue::new(json).to_value(json).to_string();
    create_rmstring(ctx, &res, str)
}

#[no_mangle]
pub extern "C" fn JSONAPI_getJSON(
    json: *const c_void,
    ctx: *mut rawmod::RedisModuleCtx,
    str: *mut *mut rawmod::RedisModuleString,
) -> c_int {
    json_api_get_json(RedisJsonKeyManager, json, ctx, str)
}

#[no_mangle]
pub extern "C" fn JSONAPI_isJSON(key: *mut rawmod::RedisModuleKey) -> c_int {
    match verify_type(key, &REDIS_JSON_TYPE) {
        Ok(_) => 1,
        Err(_) => 0,
    }
}

fn json_api_get_int<M: Manager>(_: M, json: *const c_void, val: *mut c_long) -> c_int {
    let json = unsafe { &*(json as *const M::V) };
    match json.get_type() {
        SelectValueType::Long => {
            unsafe { *val = json.get_long() };
            Status::Ok as c_int
        }
        _ => Status::Err as c_int,
    }
}

#[no_mangle]
pub extern "C" fn JSONAPI_getInt(json: *const c_void, val: *mut c_long) -> c_int {
    json_api_get_int(RedisJsonKeyManager, json, val)
}

fn json_api_get_double<M: Manager>(_: M, json: *const c_void, val: *mut c_double) -> c_int {
    let json = unsafe { &*(json as *const M::V) };
    match json.get_type() {
        SelectValueType::Double => {
            unsafe { *val = json.get_double() };
            Status::Ok as c_int
        }
        _ => Status::Err as c_int,
    }
}

#[no_mangle]
pub extern "C" fn JSONAPI_getDouble(json: *const c_void, val: *mut c_double) -> c_int {
    json_api_get_double(RedisJsonKeyManager, json, val)
}

fn json_api_get_boolean<M: Manager>(_: M, json: *const c_void, val: *mut c_int) -> c_int {
    let json = unsafe { &*(json as *const M::V) };
    match json.get_type() {
        SelectValueType::Bool => {
            unsafe { *val = json.get_bool() as c_int };
            Status::Ok as c_int
        }
        _ => Status::Err as c_int,
    }
}

#[no_mangle]
pub extern "C" fn JSONAPI_getBoolean(json: *const c_void, val: *mut c_int) -> c_int {
    json_api_get_boolean(RedisJsonKeyManager, json, val)
}

//---------------------------------------------------------------------------------------------

pub fn value_from_index(value: &Value, index: size_t) -> Result<&Value, RedisError> {
    match value {
        Value::Array(ref vec) => {
            if index < vec.len() {
                Ok(vec.get(index).unwrap())
            } else {
                Err(RedisError::Str("JSON index is out of range"))
            }
        }
        Value::Object(ref map) => {
            if index < map.len() {
                Ok(map.iter().nth(index).unwrap().1)
            } else {
                Err(RedisError::Str("JSON index is out of range"))
            }
        }
        _ => Err(RedisError::Str("Not a JSON Array or Object")),
    }
}

pub fn get_type_and_size(value: &Value) -> (JSONType, size_t) {
    RedisJSON::get_type_and_size(value)
}

pub fn set_string(from_str: &str, str: *mut *const c_char, len: *mut size_t) -> c_int {
    if !str.is_null() {
        unsafe {
            *str = from_str.as_ptr() as *const c_char;
            *len = from_str.len();
        }
        return Status::Ok as c_int;
    }
    Status::Err as c_int
}

fn json_api_get_type_internal<V: SelectValue>(v: &V) -> JSONType {
    match v.get_type() {
        SelectValueType::Null => JSONType::Null,
        SelectValueType::Bool => JSONType::Bool,
        SelectValueType::Long => JSONType::Int,
        SelectValueType::Double => JSONType::Double,
        SelectValueType::String => JSONType::String,
        SelectValueType::Array => JSONType::Array,
        SelectValueType::Object => JSONType::Object,
    }
}

pub fn json_api_next<M: Manager>(_: M, iter: *mut c_void) -> *const c_void {
    let iter = unsafe { &mut *(iter as *mut ResultsIterator<M::V>) };
    if iter.pos >= iter.results.len() {
        null_mut()
    } else {
        let res = iter.results[iter.pos] as *const M::V as *const c_void;
        iter.pos = iter.pos + 1;
        res
    }
}

pub fn json_api_len<M: Manager>(_: M, iter: *const c_void) -> size_t {
    let iter = unsafe { &*(iter as *mut ResultsIterator<M::V>) };
    iter.results.len() as size_t
}

pub fn json_api_free_iter<M: Manager>(_: M, iter: *mut c_void) {
    unsafe {
        Box::from_raw(iter as *mut ResultsIterator<M::V>);
    }
}

pub fn json_api_get<M: Manager>(_: M, val: *const c_void, path: *const c_char) -> *const c_void {
    let v = unsafe { &*(val as *const M::V) };
    let mut selector = Selector::new();
    selector.value(v);
    let path = unsafe { CStr::from_ptr(path).to_str().unwrap() };
    if selector.str_path(path).is_err() {
        return null();
    }
    match selector.select() {
        Ok(s) => Box::into_raw(Box::new(ResultsIterator { results: s, pos: 0 })) as *mut c_void,
        Err(_) => null(),
    }
}

#[no_mangle]
pub extern "C" fn JSONAPI_get(key: *const c_void, path: *const c_char) -> *const c_void {
    json_api_get(RedisJsonKeyManager, key, path)
}

#[no_mangle]
pub extern "C" fn JSONAPI_len(iter: *const c_void) -> size_t {
    json_api_len(RedisJsonKeyManager, iter)
}

#[no_mangle]
pub extern "C" fn JSONAPI_freeIter(iter: *mut c_void) {
    json_api_free_iter(RedisJsonKeyManager, iter)
}

#[no_mangle]
pub extern "C" fn JSONAPI_next(iter: *mut c_void) -> *const c_void {
    json_api_next(RedisJsonKeyManager, iter)
}

static REDISJSON_GETAPI: &str = concat!("RedisJSON_V1", "\0");

pub fn export_shared_api(ctx: &Context) {
    ctx.log_notice("Exported RedisJSON_V1 API");
    ctx.export_shared_api(
        &JSONAPI as *const RedisJSONAPI_V1 as *const c_void,
        REDISJSON_GETAPI.as_ptr() as *const c_char,
    );
}

static JSONAPI: RedisJSONAPI_V1 = RedisJSONAPI_V1 {
    openKey: JSONAPI_openKey,
    openKeyFromStr: JSONAPI_openKeyFromStr,
    get: JSONAPI_get,
    next: JSONAPI_next,
    len: JSONAPI_len,
    freeIter: JSONAPI_freeIter,
    getAt: JSONAPI_getAt,
    getLen: JSONAPI_getLen,
    getType: JSONAPI_getType,
    getInt: JSONAPI_getInt,
    getDouble: JSONAPI_getDouble,
    getBoolean: JSONAPI_getBoolean,
    getString: JSONAPI_getString,
    getJSON: JSONAPI_getJSON,
    isJSON: JSONAPI_isJSON,
};

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct RedisJSONAPI_V1 {
    pub openKey: extern "C" fn(
        ctx: *mut rawmod::RedisModuleCtx,
        key_str: *mut rawmod::RedisModuleString,
    ) -> *mut c_void,
    pub openKeyFromStr:
        extern "C" fn(ctx: *mut rawmod::RedisModuleCtx, path: *const c_char) -> *mut c_void,
    pub get: extern "C" fn(val: *const c_void, path: *const c_char) -> *const c_void,
    pub next: extern "C" fn(iter: *mut c_void) -> *const c_void,
    pub len: extern "C" fn(iter: *const c_void) -> size_t,
    pub freeIter: extern "C" fn(iter: *mut c_void),
    pub getAt: extern "C" fn(json: *const c_void, index: size_t) -> *const c_void,
    pub getLen: extern "C" fn(json: *const c_void, len: *mut size_t) -> c_int,
    pub getType: extern "C" fn(json: *const c_void) -> c_int,
    pub getInt: extern "C" fn(json: *const c_void, val: *mut c_long) -> c_int,
    pub getDouble: extern "C" fn(json: *const c_void, val: *mut c_double) -> c_int,
    pub getBoolean: extern "C" fn(json: *const c_void, val: *mut c_int) -> c_int,
    pub getString:
        extern "C" fn(json: *const c_void, str: *mut *const c_char, len: *mut size_t) -> c_int,
    pub getJSON: extern "C" fn(
        json: *const c_void,
        ctx: *mut rawmod::RedisModuleCtx,
        str: *mut *mut rawmod::RedisModuleString,
    ) -> c_int,
    pub isJSON: extern "C" fn(key: *mut rawmod::RedisModuleKey) -> c_int,
}
