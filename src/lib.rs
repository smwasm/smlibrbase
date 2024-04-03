mod init;
mod smcallout;
mod support;
mod usesm;

// alloc HashMap lazy_static RwLock json::

use std::alloc::{alloc, dealloc, Layout};
use std::ptr::null_mut;

use crate::init::is_json;
use crate::support::{handle_call_in, handle_call_in_text, SmSupportForWasm, SM_LIBR};
use crate::usesm::{get_smb_d, get_text_d, SZ};
use smcore::smu;

macro_rules! get_buf_len {
    ($ptr: expr, $len: ident) => {
        let len_u8a = std::slice::from_raw_parts($ptr, SZ::LEN as usize);
        let $len = i32::from_le_bytes(len_u8a.try_into().expect("slice length != 4")) as usize;
    };
}

// ISmLibrSupport
pub trait ISmLibrSupport: Sync + Send {
    // for debug
    fn smdebug(&self, d1: i32, d2: i32);
    // for ms
    fn get_ms(&self) -> i64;
    // for memory --- d2 --- 0 input --- 1 output --- 10 log ---
    fn put_memory(&self, d1: i32, d2: i32);
    // for sm
    fn callsm(&self, d1: i32) -> i32;
}

pub fn init(opsupport: Option<Box<dyn ISmLibrSupport>>) -> bool {
    match opsupport {
        Some(ref sp) => {
            sp.smdebug(0, 0);

            {
                let mut libr = SM_LIBR.write().unwrap();
                libr.support = opsupport;
            }

            let s = SmSupportForWasm {};
            smu.set_wasm(20, Some(Box::new(s)));
        }
        _ => {}
    }
    return true;
}

pub fn is_init(name: &String) -> bool {
    return init::is_init(name);
}

pub fn set_init(name: &String, _way: i32) {
    init::set_init(name, _way);
}

pub fn base_alloc_a(len: usize) -> i32 {
    let layout = Layout::from_size_align(len + SZ::LEN as usize, 1).unwrap();
    unsafe {
        let ptr = alloc(layout);
        if ptr != null_mut() {
            let bytes = (len as i32).to_le_bytes();
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, SZ::LEN as usize);
            return ptr as i32;
        }
    }

    return 0;
}

pub fn base_dealloc_d(poff: i32) {
    let ptr: *mut u8 = poff as *mut u8;
    if !ptr.is_null() {
        unsafe {
            get_buf_len!(ptr, len);
            let layout = Layout::from_size_align(len + SZ::LEN as usize, 1).unwrap();
            dealloc(ptr, layout);
            return;
        }
    }
}

pub fn call(poff: i32, work: i32) -> i32 {
    let opsupport: &Option<Box<dyn ISmLibrSupport>> = &SM_LIBR.read().unwrap().support;
    match opsupport {
        Some(sp) => {
            if is_json() {
                let intxt = get_text_d(poff); // heap -
                return handle_call_in_text(&intxt, work, sp);
            } else {
                let (name, smb) = get_smb_d(poff); // heap -
                return handle_call_in(&name, smb, work, sp);
            }
        }
        _ => {}
    }

    return 0;
}
