use std::collections::HashMap;
use std::sync::RwLock;

use std::sync::LazyLock;

use crate::smcallout::sm_init;
use crate::usesm::FL;

struct ForAll {
    pub map: HashMap<String, bool>,
}

static INIT: LazyLock<RwLock<ForAll>> =
    LazyLock::new(|| RwLock::new(ForAll {
        map: HashMap::new(),
    }));

static FMT_JSON: RwLock<bool> = RwLock::new(false);

pub fn is_init(name: &String) -> bool {
    let fa = INIT.read().unwrap();
    let opitm = fa.map.get(name);
    match opitm {
        Some(_itm) => {
            return true;
        }
        _ => {}
    }
    return false;
}

pub fn is_json() -> bool {
    let fa = FMT_JSON.read().unwrap();
    return *fa;
}

pub fn set_init(name: &String, _way: i32) {
    if is_init(name) {
        return;
    }

    {
        let mut fa = INIT.write().unwrap();
        fa.map.insert(name.to_string(), true);
    }

    {
        let mut fmt_json = true;
        if (_way & FL::INJSON) == FL::INJSON {
            fmt_json = false;
        }
        let mut in_json = FMT_JSON.write().unwrap();
        *in_json = fmt_json;
    }

    sm_init(name);
}
