//! `ITfThreadMgrEventSink`: a focus change ends what is being composed.

use windows::core::{Interface, Ref};
use windows::Win32::UI::TextServices::{ITfContext, ITfDocumentMgr, ITfThreadMgrEventSink_Impl};

use crate::text_service::TextService_Impl;

impl ITfThreadMgrEventSink_Impl for TextService_Impl {
    fn OnInitDocumentMgr(&self, _pdim: Ref<'_, ITfDocumentMgr>) -> windows::core::Result<()> {
        Ok(())
    }

    fn OnUninitDocumentMgr(&self, _pdim: Ref<'_, ITfDocumentMgr>) -> windows::core::Result<()> {
        Ok(())
    }

    fn OnSetFocus(
        &self,
        pdimfocus: Ref<'_, ITfDocumentMgr>,
        pdimprevfocus: Ref<'_, ITfDocumentMgr>,
    ) -> windows::core::Result<()> {
        if pdimfocus.as_ref().map(Interface::as_raw) != pdimprevfocus.as_ref().map(Interface::as_raw) {
            self.end_at_boundary("the focus moved");
        }
        Ok(())
    }

    fn OnPushContext(&self, _pic: Ref<'_, ITfContext>) -> windows::core::Result<()> {
        Ok(())
    }

    fn OnPopContext(&self, _pic: Ref<'_, ITfContext>) -> windows::core::Result<()> {
        Ok(())
    }
}
