extern crate libc;

use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::os::raw;
use std::slice;

#[allow(dead_code)]
mod ffi;
mod multi_map;

use multi_map::MultiMap;

// `MULTI_MAP_TYPE` is a value used to define what data type this module uses.
static mut MULTI_MAP_TYPE: *mut ffi::RedisModuleType = 0 as *mut ffi::RedisModuleType;

#[no_mangle]
/// Return a raw pointer to a `MultiMap` instance.
pub extern "C" fn new_multi_map() -> *mut MultiMap {
    Box::into_raw(Box::new(MultiMap::new()))
}

#[no_mangle]
/// Given a pointer to a `MultiMap`, make sure that it isn't null and then re-`Box` it so
/// it can be freed automatically.
pub unsafe extern "C" fn free_multi_map(map: *mut MultiMap) {
    if !map.is_null() {
        Box::from_raw(map);
    }
}

#[no_mangle]
/// Free a `MultiMap` type.
pub unsafe extern "C" fn MultiMapFree(value: *mut raw::c_void) {
    free_multi_map(value as *mut MultiMap)
}

#[no_mangle]
/// Free a `CString` object.
pub unsafe extern "C" fn free_ffi_string(s: *mut libc::c_char) {
    if !s.is_null() {
        CString::from_raw(s);
    }
}

#[no_mangle]
/// If the supplied key is not a `MultiMap` then return true.
pub unsafe extern "C" fn invalid_key_type(key: *mut ffi::RedisModuleKey) -> bool {
    let key_type = ffi::RedisModule_KeyType.unwrap()(key);
    key_type != (ffi::REDISMODULE_KEYTYPE_EMPTY as i32)
        && ffi::RedisModule_ModuleTypeGetType.unwrap()(key) != MULTI_MAP_TYPE
}

#[no_mangle]
/// Return early with the supplied error message.
pub unsafe extern "C" fn reply_with_error(
    ctx: *mut ffi::RedisModuleCtx,
    message: &str,
) -> libc::c_int {
    let m = CString::new(message).unwrap();
    ffi::RedisModule_ReplyWithError.unwrap()(ctx, m.as_ptr())
}

#[no_mangle]
/// Perform a lossy conversion of a module string into a `Cow<str>`.
pub unsafe extern "C" fn string_from_module_string(
    s: *const ffi::RedisModuleString,
) -> Cow<'static, str> {
    let mut len = 0;
    let c_str = ffi::RedisModule_StringPtrLen.unwrap()(s, &mut len);
    CStr::from_ptr(c_str).to_string_lossy()
}

#[no_mangle]
// The command to insert one or more elements into the multi-map.
pub unsafe extern "C" fn MultiMapInsert_RedisCommand(
    ctx: *mut ffi::RedisModuleCtx,
    argv: *mut *mut ffi::RedisModuleString,
    argc: libc::c_int,
) -> libc::c_int {
    ffi::RedisModule_AutoMemory.unwrap()(ctx);

    if argc < 4 {
        return ffi::RedisModule_WrongArity.unwrap()(ctx);
    }

    // Unpack the arguments into a slice and then open and validate the key.
    let args = slice::from_raw_parts(argv, argc as usize);
    let key = ffi::RedisModule_OpenKey.unwrap()(
        ctx,
        args[1],
        (ffi::REDISMODULE_READ as i32) | (ffi::REDISMODULE_WRITE as i32),
    ) as *mut ffi::RedisModuleKey;

    if invalid_key_type(key) {
        return reply_with_error(ctx, ffi::ERRORMSG_WRONGTYPE);
    }

    // Turn the rest of the arguments into strings to insert into the map.
    let map: *mut MultiMap;
    let key_type = ffi::RedisModule_KeyType.unwrap()(key);
    if key_type == (ffi::REDISMODULE_KEYTYPE_EMPTY as i32) {
        map = new_multi_map();
        ffi::RedisModule_ModuleTypeSetValue.unwrap()(
            key,
            MULTI_MAP_TYPE,
            map as *mut ::std::os::raw::c_void,
        );
    } else {
        map = ffi::RedisModule_ModuleTypeGetValue.unwrap()(key) as *mut MultiMap;
    }

    // We have to make some more casts to insert into our map.
    let map_key = string_from_module_string(args[2]);
    let map_values = args
        .iter()
        .skip(3)
        .map(|v| string_from_module_string(*v as *const ffi::RedisModuleString).into_owned());
    let m = &mut *map;
    m.insert(map_key, map_values);

    let resp = CString::new(ffi::SIMPLE_OK).unwrap();
    ffi::RedisModule_ReplyWithSimpleString.unwrap()(ctx, resp.as_ptr());
    ffi::RedisModule_ReplicateVerbatim.unwrap()(ctx);

    ffi::REDIS_OK
}

#[no_mangle]
// Return the length of a key in a `MultiMap`.
pub unsafe extern "C" fn MultiMapLength_RedisCommand(
    ctx: *mut ffi::RedisModuleCtx,
    argv: *mut *mut ffi::RedisModuleString,
    argc: libc::c_int,
) -> libc::c_int {
    ffi::RedisModule_AutoMemory.unwrap()(ctx);

    if argc != 3 {
        return ffi::RedisModule_WrongArity.unwrap()(ctx);
    }

    // Unpack the arguments into a slice and then open and validate the key.
    let args = slice::from_raw_parts(argv, argc as usize);
    let key = ffi::RedisModule_OpenKey.unwrap()(
        ctx,
        args[1],
        (ffi::REDISMODULE_READ as i32) | (ffi::REDISMODULE_WRITE as i32),
    ) as *mut ffi::RedisModuleKey;

    if invalid_key_type(key) {
        return reply_with_error(ctx, ffi::ERRORMSG_WRONGTYPE);
    }

    let map = ffi::RedisModule_ModuleTypeGetValue.unwrap()(key) as *mut MultiMap;
    if map.is_null() {
        ffi::RedisModule_ReplyWithLongLong.unwrap()(ctx, 0);
    } else {
        let m = &mut *map;
        let map_key = string_from_module_string(args[2]);
        ffi::RedisModule_ReplyWithLongLong.unwrap()(ctx, m.key_len(map_key) as i64);
    }

    ffi::REDIS_OK
}

#[no_mangle]
// Return a list of the values for a given key.
pub unsafe extern "C" fn MultiMapValues_RedisCommand(
    ctx: *mut ffi::RedisModuleCtx,
    argv: *mut *mut ffi::RedisModuleString,
    argc: libc::c_int,
) -> libc::c_int {
    ffi::RedisModule_AutoMemory.unwrap()(ctx);

    if argc != 3 {
        return ffi::RedisModule_WrongArity.unwrap()(ctx);
    }

    let args = slice::from_raw_parts(argv, argc as usize);
    let key = ffi::RedisModule_OpenKey.unwrap()(
        ctx,
        args[1],
        (ffi::REDISMODULE_READ as i32) | (ffi::REDISMODULE_WRITE as i32),
    ) as *mut ffi::RedisModuleKey;

    if invalid_key_type(key) {
        return reply_with_error(ctx, ffi::ERRORMSG_WRONGTYPE);
    }

    let map = ffi::RedisModule_ModuleTypeGetValue.unwrap()(key) as *mut MultiMap;
    if map.is_null() {
        ffi::RedisModule_ReplyWithArray.unwrap()(ctx, 0);
    } else {
        let m = &mut *map;
        ffi::RedisModule_ReplyWithArray.unwrap()(ctx, ffi::POSTPONED_ARRAY_LEN);
        let map_key = string_from_module_string(args[2]);
        let values = m.values(map_key);

        match values {
            Some(values) => {
                ffi::RedisModule_ReplySetArrayLength.unwrap()(ctx, values.len() as i64);
                for v in values {
                    let s_len = v.len();
                    let s = CString::new(v.as_bytes()).unwrap();
                    let s_ptr = s.into_raw();
                    ffi::RedisModule_ReplyWithStringBuffer.unwrap()(ctx, s_ptr, s_len);

                    // Even with automatic memory mangement enabled, we must free these according
                    // to the Rust documentation.
                    let _ = CString::from_raw(s_ptr);
                }
            }
            None => {
                ffi::RedisModule_ReplySetArrayLength.unwrap()(ctx, 0);
            }
        }
    }

    ffi::REDIS_OK
}

#[no_mangle]
// Delete a key from a `MultiMap`.
pub unsafe extern "C" fn MultiMapDelete_RedisCommand(
    ctx: *mut ffi::RedisModuleCtx,
    argv: *mut *mut ffi::RedisModuleString,
    argc: libc::c_int,
) -> libc::c_int {
    ffi::RedisModule_AutoMemory.unwrap()(ctx);

    if argc != 3 {
        return ffi::RedisModule_WrongArity.unwrap()(ctx);
    }

    let args = slice::from_raw_parts(argv, argc as usize);
    let key = ffi::RedisModule_OpenKey.unwrap()(
        ctx,
        args[1],
        (ffi::REDISMODULE_READ as i32) | (ffi::REDISMODULE_WRITE as i32),
    ) as *mut ffi::RedisModuleKey;

    if invalid_key_type(key) {
        return reply_with_error(ctx, ffi::ERRORMSG_WRONGTYPE);
    }

    let map = ffi::RedisModule_ModuleTypeGetValue.unwrap()(key) as *mut MultiMap;
    if map.is_null() {
        ffi::RedisModule_ReplyWithLongLong.unwrap()(ctx, 0);
    } else {
        let m = &mut *map;
        let map_key = string_from_module_string(args[2]);
        let deleted = m.delete_key(map_key);
        ffi::RedisModule_ReplyWithLongLong.unwrap()(ctx, deleted);
    }

    ffi::REDIS_OK
}

#[allow(unused_variables)]
#[no_mangle]
pub unsafe extern "C" fn MultiMapRdbLoad(
    rdb: *mut ffi::RedisModuleIO,
    encver: libc::c_int,
) -> *mut raw::c_void {
    let m = new_multi_map();
    let map = &mut *m;
    let count = ffi::RedisModule_LoadUnsigned.unwrap()(rdb);

    // We have to load `count` keys and then do a load of how many values it had after.
    for _ in 0..count {
        let mut key_len = 0;
        let loaded_key = ffi::RedisModule_LoadStringBuffer.unwrap()(rdb, &mut key_len);
        let key = CStr::from_ptr(loaded_key).to_string_lossy();
        let num_values = ffi::RedisModule_LoadUnsigned.unwrap()(rdb);

        for _ in 0..num_values {
            let mut val_len = 0;
            let loaded_value = ffi::RedisModule_LoadStringBuffer.unwrap()(rdb, &mut val_len);
            let value = CStr::from_ptr(loaded_value).to_string_lossy().into_owned();
            map.insert(key.clone(), vec![value]);
            ffi::RedisModule_Free.unwrap()(loaded_value as *mut raw::c_void);
        }

        ffi::RedisModule_Free.unwrap()(loaded_key as *mut raw::c_void);
    }

    m as *mut raw::c_void
}

#[no_mangle]
pub unsafe extern "C" fn MultiMapRdbSave(rdb: *mut ffi::RedisModuleIO, value: *mut raw::c_void) {
    if value.is_null() {
        return;
    }

    let m = &*(value as *mut MultiMap);

    // We will save our map in this order:
    // 1. Number of items in the map 2. Key name 3. Value length 4. Values
    ffi::RedisModule_SaveUnsigned.unwrap()(rdb, m.len() as u64);

    for (k, v) in m {
        let k_len = k.len();
        let key = CString::new(k.as_bytes()).unwrap();
        ffi::RedisModule_SaveStringBuffer.unwrap()(rdb, key.as_ptr(), k_len + 1);

        ffi::RedisModule_SaveUnsigned.unwrap()(rdb, v.len() as u64);
        for value in v {
            let v_len = value.len();
            let value_str = CString::new(value.as_bytes()).unwrap();
            ffi::RedisModule_SaveStringBuffer.unwrap()(rdb, value_str.as_ptr(), v_len + 1);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn MultiMapAofRewrite(
    aof: *mut ffi::RedisModuleIO,
    key: *mut ffi::RedisModuleString,
    value: *mut raw::c_void,
) {
    let map = &*(value as *mut MultiMap);
    let insert_cmd = CString::new("multimap.insert").unwrap();
    let flags = CString::new("scc").unwrap();

    for (k, v) in map {
        let actual_key = CString::new(k.as_bytes()).unwrap();

        for value in v {
            let actual_value = CString::new(value.as_bytes()).unwrap();

            ffi::RedisModule_EmitAOF.unwrap()(
                aof,
                insert_cmd.as_ptr(),
                flags.as_ptr(),
                key,
                actual_key.as_ptr(),
                actual_value.as_ptr(),
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn RedisModule_OnLoad(
    ctx: *mut ffi::RedisModuleCtx,
    _argv: *mut *mut ffi::RedisModuleString,
    _argc: libc::c_int,
) -> libc::c_int {
    if ffi::ExportedRedisModule_Init(ctx, "rmultimap\0".as_ptr(), 1, ffi::APIVER_1) == 1 {
        return ffi::REDIS_ERR;
    }

    let mut type_functions = ffi::RedisModuleTypeMethods {
        version: 1,
        rdb_load: Some(MultiMapRdbLoad),
        rdb_save: Some(MultiMapRdbSave),
        aof_rewrite: Some(MultiMapAofRewrite),
        free: Some(MultiMapFree),
        mem_usage: None,
        digest: None,
    };
    // We reuse this flag so declare it at the beginning of the function.
    let write_flag = CString::new("write").unwrap();

    let type_name = CString::new("rmultimap").unwrap();
    let out_type =
        ffi::RedisModule_CreateDataType.unwrap()(ctx, type_name.as_ptr(), 0, &mut type_functions);
    if out_type.is_null() {
        return ffi::REDIS_ERR;
    } else {
        MULTI_MAP_TYPE = out_type;
    }

    let insert_cmd = CString::new("multimap.insert").unwrap();
    if ffi::RedisModule_CreateCommand.unwrap()(
        ctx,
        insert_cmd.as_ptr(),
        Some(MultiMapInsert_RedisCommand),
        write_flag.as_ptr(),
        1,
        1,
        1,
    ) == ffi::REDIS_ERR
    {
        return ffi::REDIS_ERR;
    }

    let len_cmd = CString::new("multimap.len").unwrap();
    let read_fast_flag = CString::new("readonly fast").unwrap();
    if ffi::RedisModule_CreateCommand.unwrap()(
        ctx,
        len_cmd.as_ptr(),
        Some(MultiMapLength_RedisCommand),
        read_fast_flag.as_ptr(),
        1,
        1,
        1,
    ) == ffi::REDIS_ERR
    {
        return ffi::REDIS_ERR;
    }

    let values_cmd = CString::new("multimap.values").unwrap();
    let read_flag = CString::new("readonly").unwrap();
    if ffi::RedisModule_CreateCommand.unwrap()(
        ctx,
        values_cmd.as_ptr(),
        Some(MultiMapValues_RedisCommand),
        read_flag.as_ptr(),
        1,
        1,
        1,
    ) == ffi::REDIS_ERR
    {
        return ffi::REDIS_ERR;
    }

    let del_cmd = CString::new("multimap.del").unwrap();
    if ffi::RedisModule_CreateCommand.unwrap()(
        ctx,
        del_cmd.as_ptr(),
        Some(MultiMapDelete_RedisCommand),
        write_flag.as_ptr(),
        1,
        1,
        1,
    ) == ffi::REDIS_ERR
    {
        return ffi::REDIS_ERR;
    }

    ffi::REDIS_OK
}
