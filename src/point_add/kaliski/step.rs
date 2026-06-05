
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn kal_vent_modadd_enabled() -> bool {
    std::env::var("KAL_VENT_MODADD").ok().as_deref() == Some("1")
}

pub(crate) fn kal_vent_halve_enabled() -> bool {
    std::env::var("KAL_VENT_HALVE").ok().as_deref() == Some("1")
}
