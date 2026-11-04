/* fcitx5 front end of the 釋讀 input method.
 *
 * Every input context gets its own session from the Rust core; this file maps
 * keysyms to session keys and draws the session's view with fcitx5's preedit
 * and candidate list. */
#include "seokdok.h"

#include <fcitx/addonfactory.h>
#include <fcitx/addonmanager.h>
#include <fcitx/candidatelist.h>
#include <fcitx/inputcontext.h>
#include <fcitx/inputcontextproperty.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/inputpanel.h>
#include <fcitx/instance.h>
#include <fcitx/userinterfacemanager.h>
#include <fcitx-utils/keysym.h>

#include <memory>
#include <string>

namespace {

class SeokdokEngine;

struct State : public fcitx::InputContextProperty {
    Ime *ime = seokdok_new(nullptr);
    // The key that just acted through the mode lists: its repeats do nothing.
    fcitx::KeySym modeKeyHeld = FcitxKey_None;
    ~State() override { seokdok_free(ime); }
};

// The name of a key in the [input.keys] lists, or null when they cannot name it.
// AltGr (ISO_Level3_Shift) has no name, so it never counts as Right Alt.
const char *keyName(fcitx::KeySym sym) {
    switch (sym) {
    case FcitxKey_Hangul: return "Hangul";
    case FcitxKey_Hangul_Hanja: return "Hanja";
    case FcitxKey_Zenkaku_Hankaku: return "Zenkaku";
    case FcitxKey_Henkan_Mode: return "Henkan";
    case FcitxKey_Muhenkan: return "Muhenkan";
    case FcitxKey_Hiragana_Katakana: return "Kana";
    case FcitxKey_Eisu_toggle: return "Eisu";
    case FcitxKey_space: return "Space";
    case FcitxKey_Return: case FcitxKey_KP_Enter: return "Enter";
    case FcitxKey_Tab: case FcitxKey_ISO_Left_Tab: return "Tab";
    case FcitxKey_Escape: return "Esc";
    case FcitxKey_Control_L: return "LCtrl";
    case FcitxKey_Control_R: return "RCtrl";
    case FcitxKey_Shift_L: return "LShift";
    case FcitxKey_Shift_R: return "RShift";
    case FcitxKey_Alt_L: case FcitxKey_Meta_L: return "LAlt";
    case FcitxKey_Alt_R: case FcitxKey_Meta_R: return "RAlt";
    case FcitxKey_Super_L: return "LSuper";
    case FcitxKey_Super_R: return "RSuper";
    default: break;
    }
    if (sym >= FcitxKey_F1 && sym <= FcitxKey_F24) {
        static const char *const names[] = {"F1",  "F2",  "F3",  "F4",  "F5",  "F6",  "F7",  "F8",
                                            "F9",  "F10", "F11", "F12", "F13", "F14", "F15", "F16",
                                            "F17", "F18", "F19", "F20", "F21", "F22", "F23", "F24"};
        return names[sym - FcitxKey_F1];
    }
    if (sym >= FcitxKey_A && sym <= FcitxKey_Z) {
        static const char *const letters[] = {"A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M",
                                              "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z"};
        return letters[sym - FcitxKey_A];
    }
    if (sym >= FcitxKey_a && sym <= FcitxKey_z) {
        return keyName(static_cast<fcitx::KeySym>(sym - FcitxKey_a + FcitxKey_A));
    }
    if (sym >= FcitxKey_0 && sym <= FcitxKey_9) {
        static const char *const digits[] = {"0", "1", "2", "3", "4", "5", "6", "7", "8", "9"};
        return digits[sym - FcitxKey_0];
    }
    return nullptr;
}

uint32_t modFlags(fcitx::KeyStates states) {
    uint32_t mods = 0;
    if (states.test(fcitx::KeyState::Ctrl)) mods |= SK_MOD_CTRL;
    if (states.test(fcitx::KeyState::Shift)) mods |= SK_MOD_SHIFT;
    if (states.test(fcitx::KeyState::Alt)) mods |= SK_MOD_ALT;
    if (states.test(fcitx::KeyState::Super)) mods |= SK_MOD_SUPER;
    return mods;
}

class Candidate : public fcitx::CandidateWord {
public:
    Candidate(SeokdokEngine *engine, int index, std::string text);
    void select(fcitx::InputContext *ic) const override;

private:
    SeokdokEngine *engine_;
    int index_;
};

class SeokdokEngine : public fcitx::InputMethodEngineV2 {
public:
    explicit SeokdokEngine(fcitx::Instance *instance)
        : instance_(instance),
          factory_([](fcitx::InputContext &) { return new State; }) {
        instance_->inputContextManager().registerProperty("seokdokState", &factory_);
    }

    State *state(fcitx::InputContext *ic) { return ic->propertyFor(&factory_); }

    void activate(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) override {
        seokdok_sync(state(event.inputContext())->ime);
    }

    void deactivate(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override {
        auto *ic = event.inputContext();
        seokdok_flush(state(ic)->ime);
        apply(ic);
        InputMethodEngineV2::deactivate(entry, event);
    }

    void reset(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) override {
        auto *ic = event.inputContext();
        // Focus moved or the client cleared the text: keep what was typed.
        seokdok_flush(state(ic)->ime);
        apply(ic);
    }

    std::string subModeLabelImpl(const fcitx::InputMethodEntry &, fcitx::InputContext &ic) override {
        switch (seokdok_mode(state(&ic)->ime)) {
        case 0: return "釋";
        case 1: return "한";
        default: return "A";
        }
    }

    void keyEvent(const fcitx::InputMethodEntry &, fcitx::KeyEvent &event) override {
        auto *ic = event.inputContext();
        auto *st = state(ic);
        const fcitx::Key key = event.rawKey();
        const fcitx::KeySym sym = key.sym();
        const fcitx::KeyStates states = key.states();
        const fcitx::KeyStates shortcut{fcitx::KeyState::Ctrl, fcitx::KeyState::Alt, fcitx::KeyState::Super};
        const char *name = keyName(sym);

        if (event.isRelease()) {
            if (sym == st->modeKeyHeld) {
                st->modeKeyHeld = FcitxKey_None;
            }
            if (name && seokdok_mode_key(st->ime, name, 0, SK_KEY_UP)) {
                apply(ic);
                event.filterAndAccept();
            }
            return;
        }
        const int state = sym == st->modeKeyHeld ? SK_KEY_REPEAT : SK_KEY_DOWN;
        if (seokdok_mode_key(st->ime, name, modFlags(states), state)) {
            st->modeKeyHeld = sym;
            apply(ic);
            event.filterAndAccept();
            return;
        }
        // A modifier alone never reaches the session: it must not end a composition.
        if (key.isModifier()) {
            return;
        }

        int kind = 0;
        uint32_t ch = 0;
        const bool shift = states.test(fcitx::KeyState::Shift);
        if (states & shortcut) {
            // Shortcuts belong to the application; finish the composition first.
            seokdok_flush(st->ime);
            apply(ic);
            return;
        } else {
            switch (sym) {
            case FcitxKey_space: kind = SK_SPACE; break;
            case FcitxKey_Return: case FcitxKey_KP_Enter: kind = SK_ENTER; break;
            case FcitxKey_Escape: kind = SK_ESC; break;
            case FcitxKey_BackSpace: kind = SK_BACKSPACE; break;
            case FcitxKey_Tab: kind = SK_TAB; break;
            case FcitxKey_ISO_Left_Tab: kind = SK_TAB; break;
            case FcitxKey_Left: kind = SK_LEFT; break;
            case FcitxKey_Right: kind = SK_RIGHT; break;
            case FcitxKey_Up: kind = SK_UP; break;
            case FcitxKey_Down: kind = SK_DOWN; break;
            case FcitxKey_Page_Up: kind = SK_PAGEUP; break;
            case FcitxKey_Page_Down: kind = SK_PAGEDOWN; break;
            default: {
                const uint32_t u = fcitx::Key::keySymToUnicode(sym);
                if ((u >= 'a' && u <= 'z') || (u >= 'A' && u <= 'Z')) {
                    kind = SK_LETTER;
                    ch = u;
                } else if (u >= 0x20 && u != 0x7f) {
                    kind = SK_PRINTABLE;
                    ch = u;
                } else {
                    kind = SK_OTHER;
                }
            }
            }
        }
        const bool tabShift = sym == FcitxKey_ISO_Left_Tab || shift;
        if (seokdok_key(st->ime, kind, ch, kind == SK_TAB ? tabShift : shift)) {
            event.filterAndAccept();
        }
        apply(ic);
    }

    // Choose candidate `index` on the page, as if its digit was typed.
    void pick(fcitx::InputContext *ic, int index) {
        seokdok_key(state(ic)->ime, SK_PRINTABLE, '1' + index, false);
        apply(ic);
    }

    void apply(fcitx::InputContext *ic) {
        Ime *ime = state(ic)->ime;
        if (const char *c = seokdok_take_commit(ime)) {
            ic->commitString(c);
        }
        auto &panel = ic->inputPanel();
        panel.reset();

        fcitx::Text preedit;
        int caret = -1, len = 0;
        for (size_t i = 0, n = seokdok_preedit_len(ime); i < n; i++) {
            int32_t attr = 0;
            const char *t = seokdok_preedit_run(ime, i, &attr);
            if (!t) continue;
            fcitx::TextFormatFlags f = fcitx::TextFormatFlag::Underline;
            if (attr == 2) {
                f = fcitx::TextFormatFlags{fcitx::TextFormatFlag::HighLight, fcitx::TextFormatFlag::Underline};
                if (caret < 0) caret = len;
            }
            preedit.append(t, f);
            len += std::string(t).size();
        }
        preedit.setCursor(caret < 0 ? len : caret);
        if (ic->capabilityFlags().test(fcitx::CapabilityFlag::Preedit)) {
            panel.setClientPreedit(preedit);
        } else {
            panel.setPreedit(preedit);
        }

        const size_t n = seokdok_popup_len(ime);
        if (n > 0) {
            size_t selected = 0, position = 0, total = 0;
            bool horizontal = false;
            seokdok_popup_info(ime, &selected, &position, &total, &horizontal);
            auto list = std::make_unique<fcitx::CommonCandidateList>();
            list->setPageSize(n);
            list->setLayoutHint(horizontal ? fcitx::CandidateLayoutHint::Horizontal
                                           : fcitx::CandidateLayoutHint::Vertical);
            list->setSelectionKey(fcitx::Key::keyListFromString("1 2 3 4 5 6 7 8 9"));
            std::string note;
            for (size_t i = 0; i < n; i++) {
                const char *nt = nullptr;
                const char *t = seokdok_popup_item(ime, i, &nt);
                if (!t) continue;
                if (i == selected && nt) note = nt;
                list->append<Candidate>(this, int(i), t);
            }
            list->setGlobalCursorIndex(int(selected));
            list->setCursorPositionAfterPaging(fcitx::CursorPositionAfterPaging::DonotChange);
            panel.setCandidateList(std::move(list));
            std::string aux = std::to_string(position) + "/" + std::to_string(total);
            if (!note.empty()) aux = note + "  " + aux;
            panel.setAuxDown(fcitx::Text(aux));
        }
        ic->updatePreedit();
        ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
        ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
    }

private:
    fcitx::Instance *instance_;
    fcitx::FactoryFor<State> factory_;
};

// fcitx5 5.1 has no per-candidate comment; the selected one's note goes to
// the aux line.
Candidate::Candidate(SeokdokEngine *engine, int index, std::string text) : engine_(engine), index_(index) {
    setText(fcitx::Text(std::move(text)));
}

void Candidate::select(fcitx::InputContext *ic) const { engine_->pick(ic, index_); }

class SeokdokFactory : public fcitx::AddonFactory {
    fcitx::AddonInstance *create(fcitx::AddonManager *manager) override {
        return new SeokdokEngine(manager->instance());
    }
};

} // namespace

FCITX_ADDON_FACTORY(SeokdokFactory);
