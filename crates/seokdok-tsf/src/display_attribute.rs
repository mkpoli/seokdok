//! Display-attribute provider: three attributes style the preedit. Hangul that
//! has not been converted gets a dotted underline, converted segments a
//! solid one, and the focused segment a bold one.

use std::cell::Cell;

use windows::core::{implement, OutRef, BSTR, GUID};
use windows::Win32::UI::TextServices::{
    IEnumTfDisplayAttributeInfo, IEnumTfDisplayAttributeInfo_Impl, ITfDisplayAttributeInfo,
    ITfDisplayAttributeInfo_Impl, ITfDisplayAttributeProvider_Impl, TF_ATTR_CONVERTED,
    TF_ATTR_INPUT, TF_ATTR_TARGET_CONVERTED, TF_CT_NONE, TF_DA_COLOR, TF_DISPLAYATTRIBUTE,
    TF_LS_DOT, TF_LS_SOLID,
};

use crate::guids::{GUID_DA_CONVERTED, GUID_DA_FOCUS, GUID_DA_INPUT};
use crate::text_service::TextService_Impl;

#[derive(Clone, Copy)]
enum Kind {
    Input,
    Converted,
    Focus,
}

const KINDS: [Kind; 3] = [Kind::Input, Kind::Converted, Kind::Focus];

impl Kind {
    fn guid(self) -> GUID {
        match self {
            Kind::Input => GUID_DA_INPUT,
            Kind::Converted => GUID_DA_CONVERTED,
            Kind::Focus => GUID_DA_FOCUS,
        }
    }

    fn description(self) -> String {
        let what = match self {
            Kind::Input => "input",
            Kind::Converted => "converted",
            Kind::Focus => "focused segment",
        };
        format!("{} {what}", seokdok_product::CURRENT.id)
    }

    fn attribute(self) -> TF_DISPLAYATTRIBUTE {
        let none = TF_DA_COLOR { r#type: TF_CT_NONE, ..Default::default() };
        let (style, bold, attr) = match self {
            Kind::Input => (TF_LS_DOT, false, TF_ATTR_INPUT),
            Kind::Converted => (TF_LS_SOLID, false, TF_ATTR_CONVERTED),
            Kind::Focus => (TF_LS_SOLID, true, TF_ATTR_TARGET_CONVERTED),
        };
        TF_DISPLAYATTRIBUTE {
            crText: none,
            crBk: none,
            lsStyle: style,
            fBoldLine: bold.into(),
            crLine: none,
            bAttr: attr,
        }
    }
}

impl ITfDisplayAttributeProvider_Impl for TextService_Impl {
    fn EnumDisplayAttributeInfo(&self) -> windows::core::Result<IEnumTfDisplayAttributeInfo> {
        Ok(EnumDisplayAttributeInfo::new().into())
    }

    fn GetDisplayAttributeInfo(&self, guid: *const GUID) -> windows::core::Result<ITfDisplayAttributeInfo> {
        // SAFETY: TSF passes a valid GUID pointer.
        let requested = unsafe { *guid };
        match KINDS.iter().find(|k| k.guid() == requested) {
            Some(&kind) => Ok(DisplayAttributeInfo { kind }.into()),
            None => Err(windows::Win32::Foundation::E_INVALIDARG.into()),
        }
    }
}

#[implement(ITfDisplayAttributeInfo)]
pub struct DisplayAttributeInfo {
    kind: Kind,
}

impl ITfDisplayAttributeInfo_Impl for DisplayAttributeInfo_Impl {
    fn GetGUID(&self) -> windows::core::Result<GUID> {
        Ok(self.kind.guid())
    }

    fn GetDescription(&self) -> windows::core::Result<BSTR> {
        Ok(BSTR::from(self.kind.description()))
    }

    fn GetAttributeInfo(&self, pda: *mut TF_DISPLAYATTRIBUTE) -> windows::core::Result<()> {
        if pda.is_null() {
            return Err(windows::Win32::Foundation::E_POINTER.into());
        }
        // SAFETY: pda validated non-null; TSF provides a writable location.
        unsafe {
            *pda = self.kind.attribute();
        }
        Ok(())
    }

    fn SetAttributeInfo(&self, _pda: *const TF_DISPLAYATTRIBUTE) -> windows::core::Result<()> {
        Ok(())
    }

    fn Reset(&self) -> windows::core::Result<()> {
        Ok(())
    }
}

/// Enumerator over the three display attributes.
#[implement(IEnumTfDisplayAttributeInfo)]
pub struct EnumDisplayAttributeInfo {
    next: Cell<usize>,
}

impl EnumDisplayAttributeInfo {
    pub fn new() -> Self {
        Self { next: Cell::new(0) }
    }
}

impl IEnumTfDisplayAttributeInfo_Impl for EnumDisplayAttributeInfo_Impl {
    fn Clone(&self) -> windows::core::Result<IEnumTfDisplayAttributeInfo> {
        Ok(EnumDisplayAttributeInfo { next: Cell::new(self.next.get()) }.into())
    }

    fn Next(
        &self,
        ulcount: u32,
        rginfo: OutRef<'_, ITfDisplayAttributeInfo>,
        pcfetched: *mut u32,
    ) -> windows::core::Result<()> {
        // TSF asks for one element at a time.
        let mut fetched: u32 = 0;
        if ulcount >= 1 && self.next.get() < KINDS.len() {
            let kind = KINDS[self.next.get()];
            self.next.set(self.next.get() + 1);
            let info: ITfDisplayAttributeInfo = DisplayAttributeInfo { kind }.into();
            rginfo.write(Some(info))?;
            fetched = 1;
        }
        if !pcfetched.is_null() {
            // SAFETY: caller-provided writable location when non-null.
            unsafe {
                *pcfetched = fetched;
            }
        }
        if fetched == ulcount {
            Ok(())
        } else {
            Err(windows::Win32::Foundation::S_FALSE.into())
        }
    }

    fn Reset(&self) -> windows::core::Result<()> {
        self.next.set(0);
        Ok(())
    }

    fn Skip(&self, ulcount: u32) -> windows::core::Result<()> {
        let target = self.next.get() + ulcount as usize;
        self.next.set(target.min(KINDS.len()));
        if target > KINDS.len() {
            Err(windows::Win32::Foundation::S_FALSE.into())
        } else {
            Ok(())
        }
    }
}
