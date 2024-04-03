use json::{object, JsonValue};

use crate::init::is_json;
use crate::usesm::{get_smb_d, get_text_d, write_bytes_a, write_smb_a};
use smcore::{smh, smu};
use smdton::{SmDtonBuffer, SmDtonBuilder, SmDtonMap, SmDtonPair, SmDtonReader};

use crate::support::SM_LIBR;
use crate::{base_dealloc_d, ISmLibrSupport};

#[inline]
fn _write_json(jn: JsonValue, sp: &Box<dyn ISmLibrSupport>) -> SmDtonBuffer {
    let call_txt = json::stringify(jn);
    let ba = call_txt.as_bytes();

    let ptr = write_bytes_a(0, ba, sp); // heap +
    let call_ret = sp.callsm(ptr);
    base_dealloc_d(ptr); // heap -

    let backtxt = get_text_d(call_ret); // heap -
    let jo = json::parse(&backtxt);
    match jo {
        Ok(v) => {
            let mut sb = SmDtonBuilder::new_from_json(&v);
            return sb.build();
        }
        _ => {}
    }
    return SmDtonBuffer::new();
}

fn _sm_call_out(_input: &SmDtonPair) -> SmDtonBuffer {
    let opsupport: &Option<Box<dyn ISmLibrSupport>> = &SM_LIBR.read().unwrap().support;
    match opsupport {
        Some(sp) => {
            let rd = SmDtonReader::new(_input.raw.get_buffer());
            let name = rd.get_string(1, "name").unwrap();
            if is_json() {
                let rd2 = SmDtonReader::new(_input.update.get_buffer());
                let opjsn = rd2.to_json(1);
                let mut jn;
                match opjsn {
                    Some(jsn) => {
                        jn = jsn;
                    }
                    _ => {
                        jn = JsonValue::new_object();
                    }
                }
                jn["$usage"] = JsonValue::from(name);

                return _write_json(jn, sp);
            } else {
                let ba = _input.update.get_buffer();
                let ptr = write_smb_a(0, name, ba, sp);
 
                let call_ret = sp.callsm(ptr);
                base_dealloc_d(ptr); // heap -
            
                let (_name, rsmb) = get_smb_d(call_ret); // heap -
                return rsmb;           
            }
      }
        _ => {}
    }

    let mut smp = SmDtonMap::new();
    smp.add_string("$error", "no return");
    return smp.build();
}

pub fn sm_init(name: &str) {
    smu.log(&format!("--- sm call out init --- {} ---", name));

    let _define1 = object! {
        "$usage" => "smker.callsm"
    };
    smh.register_by_json(&_define1, _sm_call_out);
}
