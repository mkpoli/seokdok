//! Applies session [`Effect`]s to the document: start, update, commit and
//! cancel the composition, style the segments, and drive the candidate
//! window. Also `ITfCompositionSink`.

use std::mem::ManuallyDrop;

use windows::core::{IUnknownImpl, Interface, Ref};
use windows::Win32::Foundation::RECT;
use windows::Win32::System::Variant::{
    VARIANT, VARIANT_0, VARIANT_0_0, VARIANT_0_0_0, VT_I4,
};
use windows::Win32::UI::TextServices::{
    ITfComposition, ITfCompositionSink, ITfCompositionSink_Impl, ITfContext, ITfContextComposition,
    ITfInsertAtSelection, ITfRange, GUID_PROP_ATTRIBUTE, TF_AE_NONE, TF_ANCHOR_END,
    TF_ANCHOR_START, TF_DEFAULT_SELECTION, TF_IAS_QUERYONLY, TF_SELECTION, TF_SELECTIONSTYLE, TS_SS_TRANSITORY,
};

use crate::edit_session::run_sync;
use crate::session::{Attr, Effect, Run};
use crate::text_service::TextService_Impl;

/// See [`TextService_Impl::composer`].
struct Composer {
    current: Option<ITfComposition>,
    cc: ITfContextComposition,
    insert: ITfInsertAtSelection,
    sink: ITfCompositionSink,
}

impl Composer {
    /// The composition, started at the selection if there is none.
    ///
    /// # Safety
    /// `ec` must be a valid read-write edit cookie.
    unsafe fn get(&self, ec: u32) -> windows::core::Result<ITfComposition> {
        if let Some(c) = &self.current {
            return Ok(c.clone());
        }
        let range = self.insert.InsertTextAtSelection(ec, TF_IAS_QUERYONLY, &[])?;
        let comp = self.cc.StartComposition(ec, &range, &self.sink);
        crate::debug_log::log(|| format!("StartComposition: {:?}", comp.as_ref().map(|c| c.as_raw())));
        comp
    }
}

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

/// Whether the document is an IMM32 application seen through the system's
/// bridge (CUAS). Under a Korean keyboard layout that bridge accepts only a
/// one-character "interim" composition and ends any other at once, so these
/// documents get no composition while typing: the preedit is drawn in the
/// popup and the text arrives when it is confirmed.
fn is_transitory(context: &ITfContext) -> bool {
    // SAFETY: context is a valid ITfContext.
    unsafe { context.GetStatus() }.is_ok_and(|s| s.dwStaticFlags & TS_SS_TRANSITORY != 0)
}

fn utf16_len(s: &str) -> i32 {
    s.encode_utf16().count() as i32
}

impl TextService_Impl {
    /// Apply effects in order.
    pub(crate) fn apply(&self, context: &ITfContext, effects: Vec<Effect>) -> windows::core::Result<()> {
        let floating = is_transitory(context);
        let mut result = Ok(());
        for effect in effects {
            crate::debug_log::log(|| format!("apply {effect:?} floating={floating}"));
            let r = match effect {
                Effect::Preedit(runs) if floating => {
                    self.float_preedit(context, runs);
                    Ok(())
                }
                Effect::Preedit(runs) => self.set_preedit(context, &runs),
                Effect::Commit(text) => {
                    self.inner_mut().floating = None;
                    let r = self.commit(context, &text);
                    self.redraw();
                    r
                }
                Effect::Cancel if floating => {
                    self.inner_mut().floating = None;
                    self.redraw();
                    Ok(())
                }
                Effect::Cancel => self.cancel(context),
                Effect::Popup(p) => {
                    self.inner_mut().popup = p;
                    self.redraw();
                    Ok(())
                }
            };
            if let Err(e) = &r {
                crate::debug_log::log(|| format!("  -> {e:?}"));
                result = r;
            }
        }
        result
    }

    /// What an edit session needs to write into the composition: the current
    /// one, or the means to start one at the selection inside that same
    /// session. Starting it in a session of its own leaves it empty when the
    /// session ends, and the IMM32 bridge that Qt and other IMM32 applications
    /// go through terminates such a composition at once, so every keystroke
    /// would arrive as finished text.
    fn composer(&self, context: &ITfContext) -> windows::core::Result<Composer> {
        Ok(Composer {
            current: self.inner().composition.clone(),
            cc: context.cast()?,
            insert: context.cast()?,
            sink: self.to_interface(),
        })
    }

    /// Replace the composition text by `runs`, each with its display attribute.
    fn set_preedit(&self, context: &ITfContext, runs: &[Run]) -> windows::core::Result<()> {
        let composer = self.composer(context)?;
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

        let (comp, anchor) = run_sync(cid, context, move |ec| {
            // SAFETY: ec valid; comp/ctx/ranges are valid TSF objects.
            unsafe {
                let comp = composer.get(ec)?;
                let range = comp.GetRange()?;
                range.SetText(ec, 0, &text)?;

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
                Ok((comp, anchor))
            }
        })?;
        let mut state = self.inner_mut();
        state.composition = Some(comp);
        state.anchor = anchor;
        Ok(())
    }

    /// Replace the composition by final text and end it. With no composition
    /// the text is inserted at the selection.
    fn commit(&self, context: &ITfContext, text: &str) -> windows::core::Result<()> {
        if self.inner().composition.is_none() && text.is_empty() {
            return Ok(());
        }
        let composer = self.composer(context)?;
        let wide: Vec<u16> = text.encode_utf16().collect();
        let cid = self.inner().client_id;
        let ctx = context.clone();

        run_sync(cid, context, move |ec| {
            // SAFETY: ec valid; comp/ctx/range valid.
            unsafe {
                let comp = composer.get(ec)?;
                let range = comp.GetRange()?;
                range.SetText(ec, 0, &wide)?;
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
                range.SetText(ec, 0, &[])?;
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

    /// Draw the preedit in the popup at the caret (see [`is_transitory`]).
    fn float_preedit(&self, context: &ITfContext, runs: Vec<Run>) {
        let cid = self.inner().client_id;
        let ctx = context.clone();
        let anchor = run_sync(cid, context, move |ec| {
            // SAFETY: ec valid inside the session; ctx valid.
            unsafe {
                let mut sel = [TF_SELECTION::default()];
                let mut fetched = 0;
                ctx.GetSelection(ec, TF_DEFAULT_SELECTION, &mut sel, &mut fetched)?;
                let range = ManuallyDrop::into_inner(std::mem::take(&mut sel[0].range));
                Ok(range.and_then(|r| text_position(&ctx, ec, &r)))
            }
        })
        .ok()
        .flatten();
        let mut state = self.inner_mut();
        if anchor.is_some() || state.floating.is_none() {
            state.anchor = anchor;
        }
        state.floating = (!runs.is_empty()).then_some(runs);
        drop(state);
        self.redraw();
    }

    /// Show the floating preedit and the candidates, or hide the popup.
    fn redraw(&self) {
        let state = self.inner();
        let Some(win) = state.candidate_window.as_ref() else {
            return;
        };
        if state.floating.is_none() && state.popup.is_none() {
            win.hide();
        } else {
            win.show(state.floating.as_deref(), state.popup.as_ref(), state.anchor);
        }
    }

    pub(crate) fn hide_popup(&self) {
        {
            let mut state = self.inner_mut();
            state.popup = None;
            state.floating = None;
        }
        self.redraw();
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
        crate::debug_log::log(|| "OnCompositionTerminated".to_string());
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
