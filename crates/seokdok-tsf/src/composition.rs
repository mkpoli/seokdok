//! Applies session [`Effect`]s to the document: start, update, commit and
//! cancel the composition, style the segments, and drive the candidate
//! window. Also `ITfCompositionSink`.

use std::mem::ManuallyDrop;

use windows::core::{IUnknownImpl, Interface, Ref};
use windows::Win32::Foundation::{E_FAIL, RECT};
use windows::Win32::System::Variant::{
    VARIANT, VARIANT_0, VARIANT_0_0, VARIANT_0_0_0, VT_I4,
};
use windows::Win32::UI::TextServices::{
    ITfComposition, ITfCompositionSink, ITfCompositionSink_Impl, ITfContext, ITfContextComposition,
    ITfInsertAtSelection, ITfRange, GUID_PROP_ATTRIBUTE, TF_AE_NONE, TF_ANCHOR_END,
    TF_ANCHOR_START, TF_IAS_QUERYONLY, TF_SELECTION, TF_SELECTIONSTYLE, TF_ST_CORRECTION,
};

use crate::edit_session::run_sync;
use crate::session::{Attr, Effect, Popup, Run};
use crate::text_service::TextService_Impl;

/// Build a `VT_I4` VARIANT holding `value` (a display-attribute atom).
fn variant_i4(value: i32) -> VARIANT {
    VARIANT {
        Anonymous: VARIANT_0 {
            Anonymous: ManuallyDrop::new(VARIANT_0_0 {
                vt: VT_I4,
                wReserved1: 0,
                wReserved2: 0,
                wReserved3: 0,
                Anonymous: VARIANT_0_0_0 { lVal: value },
            }),
        },
    }
}

/// Set the selection to a single collapsed range.
///
/// `TF_SELECTION::range` is `ManuallyDrop<Option<ITfRange>>`, so the clone put
/// into it is released by hand once `SetSelection` has taken its own reference.
///
/// # Safety
/// `ec` must be a valid edit cookie for `ctx` inside an edit session.
unsafe fn set_single_selection(ctx: &ITfContext, ec: u32, range: &ITfRange) -> windows::core::Result<()> {
    let mut selection = TF_SELECTION {
        range: ManuallyDrop::new(Some(range.clone())),
        style: TF_SELECTIONSTYLE { ase: TF_AE_NONE, fInterimChar: false.into() },
    };
    let result = ctx.SetSelection(ec, std::slice::from_ref(&selection));
    ManuallyDrop::drop(&mut selection.range);
    result
}

fn utf16_len(s: &str) -> i32 {
    s.encode_utf16().count() as i32
}

impl TextService_Impl {
    /// Apply effects in order.
    pub(crate) fn apply(&self, context: &ITfContext, effects: Vec<Effect>) -> windows::core::Result<()> {
        let mut result = Ok(());
        for effect in effects {
            let r = match effect {
                Effect::Preedit(runs) => self.set_preedit(context, &runs),
                Effect::Commit(text) => self.commit(context, &text),
                Effect::Cancel => self.cancel(context),
                Effect::Popup(p) => {
                    self.show_popup(p);
                    Ok(())
                }
            };
            if r.is_err() {
                result = r;
            }
        }
        result
    }

    fn start_composition(&self, context: &ITfContext) -> windows::core::Result<()> {
        let cc: ITfContextComposition = context.cast()?;
        let insert: ITfInsertAtSelection = context.cast()?;
        let sink: ITfCompositionSink = self.to_interface();
        let cid = self.inner().client_id;

        let comp = run_sync(cid, context, move |ec| {
            // SAFETY: ec is a valid edit cookie inside the session.
            unsafe {
                let range = insert.InsertTextAtSelection(ec, TF_IAS_QUERYONLY, &[])?;
                cc.StartComposition(ec, &range, &sink)
            }
        })?;
        self.inner_mut().composition = Some(comp);
        Ok(())
    }

    /// Replace the composition text by `runs`, each with its display attribute.
    fn set_preedit(&self, context: &ITfContext, runs: &[Run]) -> windows::core::Result<()> {
        if self.inner().composition.is_none() {
            self.start_composition(context)?;
        }
        let comp = self.inner().composition.clone().ok_or(E_FAIL)?;
        let text: Vec<u16> = runs.iter().flat_map(|r| r.text.encode_utf16()).collect();
        let atoms = self.inner().display_atoms;
        let cid = self.inner().client_id;
        let ctx = context.clone();
        let layout: Vec<(i32, i32, Attr)> = {
            let mut offset = 0;
            runs.iter()
                .map(|r| {
                    let item = (offset, utf16_len(&r.text), r.attr);
                    offset += item.1;
                    item
                })
                .collect()
        };

        let anchor = run_sync(cid, context, move |ec| {
            // SAFETY: ec valid; comp/ctx/ranges are valid TSF objects.
            unsafe {
                let range = comp.GetRange()?;
                range.SetText(ec, TF_ST_CORRECTION, &text)?;

                let prop = ctx.GetProperty(&GUID_PROP_ATTRIBUTE)?;
                let mut anchor = None;
                for (offset, len, attr) in layout {
                    if len == 0 {
                        continue;
                    }
                    let sub = range.Clone()?;
                    let mut shifted = 0;
                    sub.Collapse(ec, TF_ANCHOR_START)?;
                    sub.ShiftEnd(ec, offset, &mut shifted, std::ptr::null())?;
                    sub.Collapse(ec, TF_ANCHOR_END)?;
                    sub.ShiftEnd(ec, len, &mut shifted, std::ptr::null())?;
                    let atom = match attr {
                        Attr::Input => atoms[0],
                        Attr::Converted => atoms[1],
                        Attr::Focus => atoms[2],
                    };
                    prop.SetValue(ec, &sub, &variant_i4(atom as i32))?;
                    if attr == Attr::Focus || anchor.is_none() {
                        anchor = text_position(&ctx, ec, &sub);
                    }
                }

                // Caret after the preedit.
                range.Collapse(ec, TF_ANCHOR_END)?;
                set_single_selection(&ctx, ec, &range)?;
                Ok(anchor)
            }
        })?;
        self.inner_mut().anchor = anchor;
        Ok(())
    }

    /// Replace the composition by final text and end it. With no composition
    /// the text is inserted at the selection.
    fn commit(&self, context: &ITfContext, text: &str) -> windows::core::Result<()> {
        if self.inner().composition.is_none() {
            if text.is_empty() {
                return Ok(());
            }
            self.start_composition(context)?;
        }
        let comp = self.inner().composition.clone().ok_or(E_FAIL)?;
        let wide: Vec<u16> = text.encode_utf16().collect();
        let cid = self.inner().client_id;
        let ctx = context.clone();

        run_sync(cid, context, move |ec| {
            // SAFETY: ec valid; comp/ctx/range valid.
            unsafe {
                let range = comp.GetRange()?;
                range.SetText(ec, TF_ST_CORRECTION, &wide)?;
                ctx.GetProperty(&GUID_PROP_ATTRIBUTE)?.Clear(ec, &range)?;
                range.Collapse(ec, TF_ANCHOR_END)?;
                set_single_selection(&ctx, ec, &range)?;
                comp.EndComposition(ec)?;
                Ok(())
            }
        })?;
        let mut state = self.inner_mut();
        state.composition = None;
        state.anchor = None;
        Ok(())
    }

    fn cancel(&self, context: &ITfContext) -> windows::core::Result<()> {
        let Some(comp) = self.inner().composition.clone() else {
            return Ok(());
        };
        let cid = self.inner().client_id;
        let ctx = context.clone();

        run_sync(cid, context, move |ec| {
            // SAFETY: ec valid; comp/ctx/range valid.
            unsafe {
                let range = comp.GetRange()?;
                range.SetText(ec, TF_ST_CORRECTION, &[])?;
                ctx.GetProperty(&GUID_PROP_ATTRIBUTE)?.Clear(ec, &range)?;
                comp.EndComposition(ec)?;
                Ok(())
            }
        })?;
        let mut state = self.inner_mut();
        state.composition = None;
        state.anchor = None;
        drop(state);
        self.hide_popup();
        Ok(())
    }

    fn show_popup(&self, popup: Option<Popup>) {
        let state = self.inner();
        let Some(win) = state.candidate_window.as_ref() else {
            return;
        };
        match popup {
            Some(p) => win.show(&p, state.anchor),
            None => win.hide(),
        }
    }

    pub(crate) fn hide_popup(&self) {
        self.show_popup(None);
    }
}

/// Screen position (left, bottom) of `range`, if the application can say.
///
/// # Safety
/// `ec` must be a valid edit cookie for `ctx` inside an edit session.
unsafe fn text_position(ctx: &ITfContext, ec: u32, range: &ITfRange) -> Option<(i32, i32)> {
    let view = ctx.GetActiveView().ok()?;
    let mut rect = RECT::default();
    let mut clipped = false.into();
    view.GetTextExt(ec, range, &mut rect, &mut clipped).ok()?;
    if rect.left == 0 && rect.bottom == 0 {
        return None;
    }
    Some((rect.left, rect.bottom))
}

impl ITfCompositionSink_Impl for TextService_Impl {
    fn OnCompositionTerminated(
        &self,
        _ecwrite: u32,
        _pcomposition: Ref<'_, ITfComposition>,
    ) -> windows::core::Result<()> {
        // The document ended the composition (focus change, app reset).
        {
            let mut state = self.inner_mut();
            state.composition = None;
            state.anchor = None;
            state.session.reset();
        }
        self.hide_popup();
        Ok(())
    }
}
