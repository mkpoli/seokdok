//! The keyboard compartments that tell Windows and IMM32 applications the
//! input mode: open, and native (Hangul) or alphanumeric conversion. The
//! profile registers GUID_TFCAT_TIPCAP_INPUTMODECOMPARTMENT, which promises
//! these are kept up to date.

use seokdok_session::session::Mode;
use std::mem::ManuallyDrop;
use windows::core::Interface;
use windows::Win32::System::Variant::{VARIANT, VARIANT_0, VARIANT_0_0, VARIANT_0_0_0, VT_I4};
use windows::Win32::UI::TextServices::{
    ITfCompartmentMgr, ITfThreadMgr, GUID_COMPARTMENT_KEYBOARD_INPUTMODE_CONVERSION,
    GUID_COMPARTMENT_KEYBOARD_OPENCLOSE, TF_CONVERSIONMODE_ALPHANUMERIC, TF_CONVERSIONMODE_NATIVE,
};

fn i4(v: i32) -> VARIANT {
    VARIANT {
        Anonymous: VARIANT_0 {
            Anonymous: ManuallyDrop::new(VARIANT_0_0 {
                vt: VT_I4,
                wReserved1: 0,
                wReserved2: 0,
                wReserved3: 0,
                Anonymous: VARIANT_0_0_0 { lVal: v },
            }),
        },
    }
}

pub fn publish(thread_mgr: &ITfThreadMgr, client_id: u32, mode: Mode) {
    let Ok(mgr) = thread_mgr.cast::<ITfCompartmentMgr>() else { return };
    let conversion = match mode {
        Mode::Latin => TF_CONVERSIONMODE_ALPHANUMERIC,
        Mode::Seokdok | Mode::Hangul => TF_CONVERSIONMODE_NATIVE,
    };
    // SAFETY: mgr is a valid compartment manager; the GUIDs are static.
    unsafe {
        if let Ok(c) = mgr.GetCompartment(&GUID_COMPARTMENT_KEYBOARD_OPENCLOSE) {
            let r = c.SetValue(client_id, &i4(1));
            crate::debug_log::log(|| format!("compartment open=1: {r:?}"));
        }
        if let Ok(c) = mgr.GetCompartment(&GUID_COMPARTMENT_KEYBOARD_INPUTMODE_CONVERSION) {
            let r = c.SetValue(client_id, &i4(conversion as i32));
            crate::debug_log::log(|| format!("compartment conversion={conversion}: {r:?}"));
        }
    }
}
