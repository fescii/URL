use crate::stores::Store;
use crate::{decode, encode, expand, shorten};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;
use std::sync::Mutex;

static STORE_LOCK: Mutex<()> = Mutex::new(());

fn cstr_to_str<'a>(ptr: *const c_char) -> Option<&'a str> {
  if ptr.is_null() {
    return None;
  }
  unsafe { CStr::from_ptr(ptr).to_str().ok() }
}

fn str_to_raw_cstr(s: &str) -> *mut c_char {
  CString::new(s).map(|c| c.into_raw()).unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn urls_shorten(
  raw_url: *const c_char,
  store_dir: *const c_char,
) -> *mut c_char {
  let url = match cstr_to_str(raw_url) {
    Some(u) => u,
    None => return std::ptr::null_mut(),
  };

  let dir_str = cstr_to_str(store_dir).unwrap_or(".urls_store");
  let _guard = STORE_LOCK.lock().unwrap();

  match Store::open(Path::new(dir_str)) {
    Ok(mut store) => match shorten(url, Some(&mut store)) {
      Ok(key) => str_to_raw_cstr(&key),
      Err(_) => std::ptr::null_mut(),
    },
    Err(_) => std::ptr::null_mut(),
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn urls_expand(
  key_ptr: *const c_char,
  store_dir: *const c_char,
) -> *mut c_char {
  let key = match cstr_to_str(key_ptr) {
    Some(k) => k,
    None => return std::ptr::null_mut(),
  };

  let dir_str = cstr_to_str(store_dir).unwrap_or(".urls_store");
  let _guard = STORE_LOCK.lock().unwrap();

  match Store::open(Path::new(dir_str)) {
    Ok(mut store) => match expand(key, &mut store) {
      Ok(bytes) => match std::str::from_utf8(&bytes) {
        Ok(s) => str_to_raw_cstr(s),
        Err(_) => std::ptr::null_mut(),
      },
      Err(_) => std::ptr::null_mut(),
    },
    Err(_) => std::ptr::null_mut(),
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn urls_encode(raw_url: *const c_char) -> *mut c_char {
  let url = match cstr_to_str(raw_url) {
    Some(u) => u,
    None => return std::ptr::null_mut(),
  };

  match encode(url, None) {
    Ok(code) => str_to_raw_cstr(&code),
    Err(_) => std::ptr::null_mut(),
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn urls_decode(code_ptr: *const c_char) -> *mut c_char {
  let code = match cstr_to_str(code_ptr) {
    Some(c) => c,
    None => return std::ptr::null_mut(),
  };

  match decode(code) {
    Ok(url) => str_to_raw_cstr(&url),
    Err(_) => std::ptr::null_mut(),
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn urls_store_stats(store_dir: *const c_char) -> *mut c_char {
  let dir_str = cstr_to_str(store_dir).unwrap_or(".urls_store");
  let _guard = STORE_LOCK.lock().unwrap();

  match Store::open(Path::new(dir_str)) {
    Ok(store) => {
      let keys = store.len();
      let ram_bytes = store.memory_size();
      let disk_bytes = store.disk_size();
      let bytes_per_key = if keys > 0 {
        ram_bytes as f64 / keys as f64
      } else {
        0.0
      };

      let json = format!(
        r#"{{"keys":{},"ramBytes":{},"diskBytes":{},"bytesPerKey":{:.2}}}"#,
        keys, ram_bytes, disk_bytes, bytes_per_key
      );
      str_to_raw_cstr(&json)
    }
    Err(_) => str_to_raw_cstr(r#"{"keys":0,"ramBytes":0,"diskBytes":0,"bytesPerKey":0.0}"#),
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn urls_string_free(ptr: *mut c_char) {
  if !ptr.is_null() {
    unsafe {
      let _ = CString::from_raw(ptr);
    }
  }
}
