//! Applies session [`Effect`]s to the document: start, update, commit and
//! cancel the composition, style the segments, and drive the candidate
//! window. Also `ITfCompositionSink`.

use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

use windows::core::{IUnknownImpl, Interface, Ref};
use windows::Win32::Foundation::RECT;
use windows::Win32::System::Variant::{
    VARIANT, VARIANT_0, VARIANT_0_0, VARIANT_0_0_0, VT_I4,
};
use windows::Win32::UI::TextServices::{
    ITfComposition, ITfCompositionSink, ITfCompositionSink_Impl, ITfContext, ITfContextComposition,
    ITfInsertAtSelection, ITfRange, GUID_PROP_ATTRIBUTE, TF_AE_NONE, TF_ANCHOR_END,
    TF_ANCHOR_START, TF_DEFAULT_SELECTION, TF_IAS_QUERYONLY, TF_SELECTION, TF_SELECTIONSTYLE, TS_SD_READONLY, TS_SS_TRANSITORY,
};

use crate::config::Surface;
use crate::edit_session::run_sync;
use crate::engine_host::HOST;
use crate::session::{Attr, Effect, Run};
use crate::surface::{choose, classify_end, undelivered, End, Facts};
use crate::text_service::TextService_Impl;

/// This process refused a composition, so it gets the floating preedit.
static NEEDS_FLOATING: AtomicBool = AtomicBool::new(false);

/// What came of writing the preedit into the document.
enum Written {
    Shown(ITfComposition, Option<(i32, i32)>),
    /// The document would not start a composition; nothing reached it.
    NotStarted,
    /// The composition is open but its text is gone; it has been ended.
    Vanished,
    /// The first write failed midway and the composition was ended. `undone`:
    /// a fresh range over it was emptied and checked, inside the edit session
    /// that wrote it, so the document never saw the text.
    FirstWriteFailed { undone: bool },
}

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

/// The context's static flags.
fn static_flags(context: &ITfContext) -> u32 {
    // SAFETY: context is a valid ITfContext.
    unsafe { context.GetStatus() }.map_or(0, |s| s.dwStaticFlags)
}

/// `TS_SS_TRANSITORY` alone does not tell whether a composition will hold.
/// The system's IMM32 bridge sets it and ends each composition it is given,
/// handing over the text as finished; Chromium and the console set it on text
/// stores of their own.
fn is_transitory(context: &ITfContext) -> bool {
    static_flags(context) & TS_SS_TRANSITORY != 0
}

/// Nothing editable has the focus: Chromium says so while no field does.
fn is_read_only(context: &ITfContext) -> bool {
    // SAFETY: context is a valid ITfContext.
    unsafe { context.GetStatus() }.is_ok_and(|s| s.dwDynamicFlags & TS_SD_READONLY != 0)
}

/// The executable's file name, for the setting in config.toml.
fn process_name() -> &'static str {
    static NAME: OnceLock<String> = OnceLock::new();
    NAME.get_or_init(|| {
        std::env::current_exe().ok().and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned())).unwrap_or_default()
    })
}

fn forced_surface() -> Option<Surface> {
    let cfg = crate::config::current();
    cfg.composition.iter().find(|(app, _)| app.eq_ignore_ascii_case(process_name())).map(|(_, surface)| *surface)
}

fn utf16_len(s: &str) -> i32 {
    s.encode_utf16().count() as i32
}

impl TextService_Impl {
    /// Apply effects in order.
    pub(crate) fn apply(&self, context: &ITfContext, effects: Vec<Effect>) -> windows::core::Result<()> {
        let mut result = Ok(());
        for effect in effects {
            crate::debug_log::log(|| format!("apply {effect:?} floating={}", self.inner().floating.is_some()));
            let r = match effect {
                Effect::Preedit(runs) => match self.surface(context) {
                    Surface::Floating => {
                        self.float_preedit(context, runs);
                        Ok(())
                    }
                    Surface::Inline => self.set_preedit(context, runs),
                },
                Effect::Commit(text) => {
                    self.inner_mut().floating = None;
                    let r = self.commit(context, &text);
                    self.redraw();
                    r
                }
                Effect::Cancel => {
                    self.inner_mut().floating = None;
                    let r = self.cancel(context);
                    self.redraw();
                    r
                }
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

    /// The surface of the composition in progress, or the one to start with.
    fn surface(&self, context: &ITfContext) -> Surface {
        let through_imm32 = {
            let state = self.inner();
            if state.composition.is_some() {
                return Surface::Inline;
            }
            if state.floating.is_some() {
                return Surface::Floating;
            }
            state.imm32_context == Some(context.as_raw() as usize)
        };
        let flags = static_flags(context);
        let (surface, why) = choose(Facts {
            transitory: flags & TS_SS_TRANSITORY != 0,
            through_imm32,
            remembered: NEEDS_FLOATING.load(Ordering::Relaxed),
            forced: forced_surface(),
        });
        crate::debug_log::log(|| format!("surface {surface:?}: {why} (static flags {flags:#x})"));
        surface
    }

    /// Float this process from now on.
    fn remember_floating(&self, why: &str) {
        if !NEEDS_FLOATING.swap(true, Ordering::Relaxed) {
            crate::debug_log::log(|| format!("fallback: this process gets the floating preedit: {why}"));
        }
    }

    /// What an edit session needs to write into the composition: the current
    /// one, or the means to start one at the selection inside that same
    /// session. A composition started empty in a session of its own may be
    /// ended before the text arrives.
    fn composer(&self, context: &ITfContext) -> windows::core::Result<Composer> {
        let current = self.inner().composition.clone();
        Ok(Composer {
            current,
            cc: context.cast()?,
            insert: context.cast()?,
            sink: self.to_interface(),
        })
    }

    /// Replace the composition text by `runs`, each with its display attribute.
    /// If the document will not hold it, the text moves to the floating preedit.
    fn set_preedit(&self, context: &ITfContext, runs: Vec<Run>) -> windows::core::Result<()> {
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
        let transitory = is_transitory(context);
        let started = composer.current.is_none();
        let holds_text = !started && self.inner().drawn.iter().any(|r| !r.text.is_empty());
        let ended_before = {
            let mut state = self.inner_mut();
            state.drawn = runs.clone();
            state.composition_transitory = transitory;
            state.writing = true;
            state.compositions_ended
        };

        let written = run_sync(cid, context, move |ec| {
            // SAFETY: ec valid; comp/ctx/ranges are valid TSF objects.
            unsafe {
                let comp = match composer.get(ec) {
                    Ok(comp) => comp,
                    Err(_) if transitory && started => return Ok(Written::NotStarted),
                    Err(e) => return Err(e),
                };
                if transitory && holds_text && comp.GetRange()?.IsEmpty(ec)?.as_bool() {
                    comp.EndComposition(ec)?;
                    return Ok(Written::Vanished);
                }
                let write = || -> windows::core::Result<Option<(i32, i32)>> {
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
                    Ok(anchor)
                };
                match write() {
                    Ok(anchor) => Ok(Written::Shown(comp, anchor)),
                    Err(e) if started => {
                        crate::debug_log::log(|| format!("first write failed: {e:?}"));
                        // `write` may have collapsed its range; ask again.
                        let undone = comp.GetRange().and_then(|range| {
                            range.SetText(ec, 0, &[])?;
                            range.IsEmpty(ec)
                        });
                        let _ = comp.EndComposition(ec);
                        Ok(Written::FirstWriteFailed { undone: undone.is_ok_and(|e| e.as_bool()) })
                    }
                    Err(e) => Err(e),
                }
            }
        });
        let ended = {
            let mut state = self.inner_mut();
            state.writing = false;
            state.compositions_ended != ended_before
        };

        // A store that cannot be written to right now (locked, nothing editable
        // focused) says nothing about compositions; the popup keeps the typing
        // visible.
        let written = match written {
            Err(e) if transitory && started => {
                crate::debug_log::log(|| format!("preedit not written, floating this composition: {e:?}"));
                self.float_preedit(context, runs);
                return Ok(());
            }
            other => other?,
        };
        match written {
            Written::Shown(comp, anchor) => {
                let mut state = self.inner_mut();
                state.anchor = anchor;
                if ended {
                    // The sink dealt with it; `comp` is dead.
                    drop(state);
                    self.redraw();
                } else {
                    state.composition = Some(comp);
                    state.first_write_at = started.then(Instant::now);
                    if !started {
                        state.refusals.accepted();
                    }
                }
            }
            Written::NotStarted => {
                crate::debug_log::log(|| "the document would not start a composition".to_string());
                if !is_read_only(context) && self.inner_mut().refusals.refused(false) {
                    self.remember_floating("the document would not start a composition");
                }
                self.float_preedit(context, runs);
            }
            Written::Vanished => {
                self.remember_floating("the composition's text vanished");
                crate::debug_log::log(|| "  the text so far counts as delivered; replaying this key".to_string());
                let effects = {
                    let mut state = self.inner_mut();
                    state.composition = None;
                    state.drawn.clear();
                    state.session.reset();
                    let key = state.current_key.take();
                    key.map(|k| state.session.handle(k, &HOST).effects).unwrap_or_default()
                };
                return self.apply(context, effects);
            }
            Written::FirstWriteFailed { undone: true } => {
                crate::debug_log::log(|| "  undone before the document saw it, floating this composition".to_string());
                self.float_preedit(context, runs);
            }
            Written::FirstWriteFailed { undone: false } => {
                crate::debug_log::log(|| "  could not be undone: the document keeps it as finished, session reset".to_string());
                let mut state = self.inner_mut();
                state.drawn.clear();
                state.session.reset();
                drop(state);
                self.hide_popup();
            }
        }
        Ok(())
    }

    /// Replace the composition by final text and end it. With no composition
    /// the text is inserted at the selection: through a composition that starts
    /// and ends here on the IMM32 path, whose bridge delivers text that way;
    /// plainly in a transitory text store, because a composition started while
    /// handling a key makes Chromium fire that key's keydown ahead of the text
    /// and outside any composition, and a page taking it for a real Enter then
    /// breaks the line before the text arrives.
    fn commit(&self, context: &ITfContext, text: &str) -> windows::core::Result<()> {
        let open = self.inner().composition.is_some();
        if !open && text.is_empty() {
            return Ok(());
        }
        let transitory = is_transitory(context);
        let plain = !open && transitory && self.inner().imm32_context != Some(context.as_raw() as usize);
        let delivered: String = self.inner().drawn.iter().map(|r| r.text.as_str()).collect();
        let owed: Vec<u16> = undelivered(&delivered, text).encode_utf16().collect();
        crate::debug_log::log(|| format!("commit: {}", if open { "into the composition" } else if plain { "plain insert" } else { "one-shot composition" }));
        let composer = self.composer(context)?;
        let wide: Vec<u16> = text.encode_utf16().collect();
        let cid = self.inner().client_id;
        let ctx = context.clone();

        self.inner_mut().ending = true;
        let result = run_sync(cid, context, move |ec| {
            // SAFETY: ec valid; comp/ctx/range valid.
            unsafe {
                if plain {
                    let range = composer.insert.InsertTextAtSelection(ec, Default::default(), &wide)?;
                    range.Collapse(ec, TF_ANCHOR_END)?;
                    return set_single_selection(&ctx, ec, &range);
                }
                let comp = composer.get(ec)?;
                let range = comp.GetRange()?;
                if open && transitory && !delivered.is_empty() && range.IsEmpty(ec)?.as_bool() {
                    crate::debug_log::log(|| format!("commit: the composition's text vanished; counts as delivered, {} still owed", owed.len()));
                    range.SetText(ec, 0, &owed)?;
                    range.Collapse(ec, TF_ANCHOR_END)?;
                    set_single_selection(&ctx, ec, &range)?;
                    return comp.EndComposition(ec);
                }
                range.SetText(ec, 0, &wide)?;
                ctx.GetProperty(&GUID_PROP_ATTRIBUTE)?.Clear(ec, &range)?;
                range.Collapse(ec, TF_ANCHOR_END)?;
                set_single_selection(&ctx, ec, &range)?;
                comp.EndComposition(ec)?;
                Ok(())
            }
        });
        self.inner_mut().ending = false;
        result?;
        let mut state = self.inner_mut();
        state.composition = None;
        state.anchor = None;
        state.drawn.clear();
        Ok(())
    }

    fn cancel(&self, context: &ITfContext) -> windows::core::Result<()> {
        let Some(comp) = self.inner().composition.clone() else {
            return Ok(());
        };
        let cid = self.inner().client_id;
        let ctx = context.clone();

        self.inner_mut().ending = true;
        let result = run_sync(cid, context, move |ec| {
            // SAFETY: ec valid; comp/ctx/range valid.
            unsafe {
                let range = comp.GetRange()?;
                range.SetText(ec, 0, &[])?;
                ctx.GetProperty(&GUID_PROP_ATTRIBUTE)?.Clear(ec, &range)?;
                comp.EndComposition(ec)?;
                Ok(())
            }
        });
        self.inner_mut().ending = false;
        result?;
        let mut state = self.inner_mut();
        state.composition = None;
        state.anchor = None;
        state.drawn.clear();
        drop(state);
        self.hide_popup();
        Ok(())
    }

    /// Draw the preedit in the popup at the caret.
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
        let end = {
            let mut state = self.inner_mut();
            state.compositions_ended += 1;
            if state.ending {
                // Our own commit or cancel: the session has moved on already.
                state.composition = None;
                return Ok(());
            }
            let since_first_write = state.first_write_at.map(|t| t.elapsed());
            classify_end(state.composition_transitory, state.writing, since_first_write)
        };
        crate::debug_log::log(|| format!("OnCompositionTerminated: {end:?}"));
        if let End::Refused { in_edit } = end {
            if self.inner_mut().refusals.refused(in_edit) {
                self.remember_floating("the document ended a composition just written");
            }
        }
        // The document ended the composition (a refusal, focus change, app
        // reset) and has its text. Erasing it here could not show that the
        // application gave it back, so it counts as finished and the session
        // holds nothing of it.
        crate::debug_log::log(|| "  the document keeps the text as finished, session reset".to_string());
        {
            let mut state = self.inner_mut();
            state.composition = None;
            state.anchor = None;
            state.drawn.clear();
            state.session.reset();
        }
        self.hide_popup();
        Ok(())
    }
}
