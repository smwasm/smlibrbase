use lazy_static::lazy_static;
use smdton::{SmDtonBuffer, SmDtonBuilder, SmDtonReader};
use std::sync::RwLock;

use crate::usesm;
use crate::{base_dealloc_d, ISmLibrSupport};
use smcore::ISmCoreSupport;

lazy_static! {
    pub static ref SM_LIBR: RwLock<SmLibrOption> = RwLock::new(SmLibrOption { support: None });
}

pub struct SmLibrOption {
    pub support: Option<Box<dyn ISmLibrSupport>>,
}

pub struct SmSupportForWasm {}

impl ISmCoreSupport for SmSupportForWasm {
    fn sm_log(&self, txt: &str) {
        let opsupport = &SM_LIBR.read().unwrap().support;
        match opsupport {
            Some(sp) => {
                let ba = txt.as_bytes();
                let ptr = usesm::write_bytes_a(10, ba, sp); // heap +
                base_dealloc_d(ptr); // heap -
            }
            _ => {}
        }
    }

    fn get_current_ms(&self) -> u128 {
        let opsupport = &SM_LIBR.read().unwrap().support;
        match opsupport {
            Some(sp) => {
                return sp.get_ms() as u128;
            }
            _ => {}
        }
        return 0;
    }
}

pub fn handle_call_in_text(txt: &str, work: i32, sp: &Box<dyn ISmLibrSupport>) -> i32 {
    let jo = json::parse(txt);
    match jo {
        Ok(v) => {
            let name = v["$usage"].as_str().unwrap();
            let mut smb = SmDtonBuilder::new_from_json(&v);
            let r = usesm::act(&name, smb.build(), work > 0);
            let rd = SmDtonReader::new(&r.get_buffer());
            let out_txt = rd.to_json(1).unwrap().pretty(4);
            let ba = out_txt.as_bytes();

            let ptr = usesm::write_bytes_a(1, ba, sp); // heap +
            return ptr;
        }
        _ => {}
    }

    return 0;
}

pub fn handle_call_in(
    name: &str,
    smb: SmDtonBuffer,
    work: i32,
    sp: &Box<dyn ISmLibrSupport>,
) -> i32 {
    let r = usesm::act(name, smb, work > 0);
    let ptr = usesm::write_smb_a(1, "", &r.get_buffer(), sp); // heap +
    return ptr;
}
