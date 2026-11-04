//! Whether the field in focus is one nothing should be learned from: a
//! password, an address or a number. TSF says so through the context's
//! keyboard compartments and input scopes.

use std::mem::ManuallyDrop;

use windows::core::{Interface, IUnknown};
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::System::Variant::{VariantClear, VARIANT, VT_I4, VT_UNKNOWN};
use windows::Win32::UI::TextServices::{
    ITfCompartmentMgr, ITfContext, ITfInputScope, InputScope, GUID_COMPARTMENT_EMPTYCONTEXT,
    GUID_COMPARTMENT_KEYBOARD_DISABLED, GUID_PROP_INPUTSCOPE, IS_DIGITS, IS_EMAILNAME_OR_ADDRESS,
    IS_EMAIL_SMTPEMAILADDRESS, IS_EMAIL_USERNAME, IS_NUMBER, IS_NUMBER_FULLWIDTH, IS_PASSWORD,
    IS_PRIVATE, IS_TELEPHONE_AREACODE, IS_TELEPHONE_COUNTRYCODE, IS_TELEPHONE_FULLTELEPHONENUMBER,
    IS_TELEPHONE_LOCALNUMBER, IS_URL, TF_DEFAULT_SELECTION, TF_SELECTION,
};

use crate::edit_session::run_sync_read;

fn scope_is_private(scope: InputScope) -> bool {
    matches!(
        scope,
        IS_PASSWORD
            | IS_PRIVATE
            | IS_EMAIL_SMTPEMAILADDRESS
            | IS_EMAIL_USERNAME
            | IS_EMAILNAME_OR_ADDRESS
            | IS_URL
            | IS_DIGITS
            | IS_NUMBER
            | IS_NUMBER_FULLWIDTH
            | IS_TELEPHONE_FULLTELEPHONENUMBER
            | IS_TELEPHONE_COUNTRYCODE
            | IS_TELEPHONE_AREACODE
            | IS_TELEPHONE_LOCALNUMBER
    )
}

/// A compartment of the context holds a non-zero number.
fn compartment_on(context: &ITfContext, guid: &windows::core::GUID) -> bool {
    let Ok(mgr) = context.cast::<ITfCompartmentMgr>() else { return false };
    // SAFETY: mgr is a valid compartment manager; the GUID outlives the call,
    // and the variant is cleared once read.
    unsafe {
        let Ok(mut value) = mgr.GetCompartment(guid).and_then(|c| c.GetValue()) else { return false };
        let on = value.Anonymous.Anonymous.vt == VT_I4 && value.Anonymous.Anonymous.Anonymous.lVal != 0;
        let _ = VariantClear(&mut value);
        on
    }
}

/// The input scopes at the selection; empty when the application names none.
fn input_scopes(context: &ITfContext, ec: u32) -> windows::core::Result<Vec<InputScope>> {
    // SAFETY: ec is valid inside the edit session; every interface comes from
    // a call that succeeded, and the array and the variant are released here.
    unsafe {
        let mut sel = [TF_SELECTION::default()];
        let mut fetched = 0;
        context.GetSelection(ec, TF_DEFAULT_SELECTION, &mut sel, &mut fetched)?;
        let Some(range) = ManuallyDrop::into_inner(std::mem::take(&mut sel[0].range)) else { return Ok(Vec::new()) };
        let mut value: VARIANT = context.GetAppProperty(&GUID_PROP_INPUTSCOPE)?.GetValue(ec, &range)?;
        let inner = &value.Anonymous.Anonymous;
        let scope: Option<ITfInputScope> = if inner.vt == VT_UNKNOWN {
            let unknown: &Option<IUnknown> = &inner.Anonymous.punkVal;
            unknown.as_ref().and_then(|u| u.cast().ok())
        } else {
            None
        };
        let mut scopes = Vec::new();
        if let Some(scope) = scope {
            let (mut list, mut count) = (std::ptr::null_mut(), 0u32);
            scope.GetInputScopes(&mut list, &mut count)?;
            if !list.is_null() {
                scopes.extend_from_slice(std::slice::from_raw_parts(list, count as usize));
                CoTaskMemFree(Some(list as *const _));
            }
        }
        let _ = VariantClear(&mut value);
        Ok(scopes)
    }
}

/// Whether to keep what is typed in `context` out of the table of picks. Call
/// it from a key event, where a synchronous edit session is allowed. A field
/// that cannot be read counts as private.
pub(crate) fn is_private(client_id: u32, context: &ITfContext) -> bool {
    if compartment_on(context, &GUID_COMPARTMENT_KEYBOARD_DISABLED) || compartment_on(context, &GUID_COMPARTMENT_EMPTYCONTEXT) {
        return true;
    }
    let ctx = context.clone();
    match run_sync_read(client_id, context, move |ec| input_scopes(&ctx, ec)) {
        Ok(scopes) => scopes.into_iter().any(scope_is_private),
        Err(e) => {
            crate::debug_log::log(|| format!("input scopes unread, nothing learned here: {e:?}"));
            true
        }
    }
}
