use json::JsonValue;
use smcore::{smh, smu};
use smdton::{SmDtonBuffer, SmDtonBuilder};

use crate::{base_alloc_a, base_dealloc_d, ISmLibrSupport};

macro_rules! get_buf_len {
    ($ptr: expr, $len: ident) => {
        let len_u8a = std::slice::from_raw_parts($ptr, SZ::LEN as usize);
        let $len = i32::from_le_bytes(len_u8a.try_into().expect("slice length != 4")) as usize;
    };
}

macro_rules! get_buf_name {
    ($p_nlenoff: expr, $p_nmoff: expr, $nmlen: ident, $name: expr) => {
        let nmsize = std::slice::from_raw_parts($p_nlenoff, 2);
        let $nmlen = u16::from_le_bytes(nmsize.try_into().expect("slice length != 2")) as usize;
        if $nmlen > 1 {
            let slice = std::slice::from_raw_parts($p_nmoff, $nmlen - 1);
            $name = String::from_utf8(slice.to_vec()).expect("Invalid UTF-8");
        }
    };
}

pub struct SZ {}

impl SZ {
    pub const LEN: i32 = 4;
    pub const TY: i32 = 1;
    pub const NM: i32 = 2;
    pub const TY_NM: i32 = Self::TY + Self::NM;
    pub const LEN_TY: i32 = Self::LEN + Self::TY;
    pub const LEN_TY_NM: i32 = Self::LEN + Self::TY + Self::NM;
}

pub struct FL {}

impl FL {
    pub const INJSON: i32 = 0x100;
}

#[inline]
pub fn act(parameter: SmDtonBuffer, work: bool) -> SmDtonBuffer {
    if work {
        let _ret = smh.call(parameter);
        return _ret;
    } else {
        let txt = parameter.stringify().unwrap();
        let mut jsn = json::parse(&txt).unwrap();
        jsn["$work"] = JsonValue::from(false);
        let current = format!("{}", smu.get_current_ms());
        jsn["$ms"] = JsonValue::from(current);

        let mut sb = SmDtonBuilder::new_from_json(&jsn);
        return sb.build();
    }
}

pub fn get_text_d(poff: i32) -> String {
    let txt: String;

    let ptr: *mut u8 = poff as *mut u8;
    let p_txt: *mut u8 = (poff + SZ::LEN) as *mut u8;

    unsafe {
        get_buf_len!(ptr, len);
        let slice = std::slice::from_raw_parts(p_txt, len);
        txt = String::from_utf8(slice.to_vec()).expect("Invalid UTF-8");
    }
    base_dealloc_d(poff); // heap -
    return txt;
}

pub fn get_smb_d(poff: i32) -> (String, SmDtonBuffer) {
    let mut name = "".to_string();
    let smb;

    let ptr: *mut u8 = poff as *mut u8;
    let p_nlenoff: *mut u8 = (poff + SZ::LEN_TY) as *mut u8;
    let p_nmoff: *mut u8 = (poff + SZ::LEN_TY_NM) as *mut u8;

    unsafe {
        get_buf_len!(ptr, total);
        get_buf_name!(p_nlenoff, p_nmoff, nmlen, name);

        let p_data: *mut u8 = (poff + SZ::LEN_TY_NM + nmlen as i32) as *mut u8;
        let slice = std::slice::from_raw_parts(p_data, total - 2 - nmlen);
        smb = SmDtonBuffer {
            off: 0,
            buf: slice.to_vec(),
        };
    }
    base_dealloc_d(poff); // heap -

    return (name, smb);
}

pub fn write_bytes_a(ty: i32, ba: &[u8], sp: &Box<dyn ISmLibrSupport>) -> i32 {
    let len = ba.len();

    let ptr = base_alloc_a(len); // heap +
    let p_data: *mut u8 = (ptr + SZ::LEN) as *mut u8;

    unsafe {
        std::ptr::copy_nonoverlapping(ba.as_ptr(), p_data, len as usize);
    }

    sp.put_memory(ptr, ty);
    return ptr;
}

pub fn write_smb_a(ty: i32, name: &str, ba: &[u8], sp: &Box<dyn ISmLibrSupport>) -> i32 {
    let fb = (2 as u8).to_le_bytes();

    let mut nmbytes = name.as_bytes().to_vec();
    nmbytes.push(0);
    let nmlen = nmbytes.len();
    let nmlen_bytes = (nmlen as u16).to_le_bytes();

    let len = ba.len() + nmlen + SZ::TY_NM as usize;
    let poff = base_alloc_a(len); // heap +

    let p_flag: *mut u8 = (poff + SZ::LEN) as *mut u8;
    let p_nmlen: *mut u8 = (poff + SZ::LEN_TY) as *mut u8;
    let p_nm: *mut u8 = (poff + SZ::LEN_TY_NM) as *mut u8;
    let p_data: *mut u8 = (poff + SZ::LEN_TY_NM + nmlen as i32) as *mut u8;

    unsafe {
        std::ptr::copy_nonoverlapping(fb.as_ptr(), p_flag, SZ::TY as usize);
        std::ptr::copy_nonoverlapping(nmlen_bytes.as_ptr(), p_nmlen, SZ::NM as usize);
        std::ptr::copy_nonoverlapping(nmbytes.as_ptr(), p_nm, nmlen);
        std::ptr::copy_nonoverlapping(ba.as_ptr(), p_data, ba.len());
    }

    sp.put_memory(poff, ty);
    return poff;
}

// jinzr
/*
pub fn _host_debug(_d1: i32, _d2: i32) {
    let opsupport = &SM_LIBR.read().unwrap().support;
    match opsupport {
        Some(_sp) => {
            _sp.smdebug(_d1, _d2);
        }
        _ => {}
    }
}
*/
