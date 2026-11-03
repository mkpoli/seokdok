/* C interface of the seokdok session (crates/seokdok-ffi/src/lib.rs). */
#pragma once
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

enum {
    SK_LETTER = 1, SK_PRINTABLE, SK_SPACE, SK_ENTER, SK_ESC, SK_BACKSPACE, SK_TAB,
    SK_LEFT, SK_RIGHT, SK_UP, SK_DOWN, SK_PAGEUP, SK_PAGEDOWN, SK_CONVERT, SK_TOGGLE, SK_OTHER
};

typedef struct Ime Ime;

Ime *seokdok_new(const char *dict_dir);
void seokdok_free(Ime *ime);
void seokdok_sync(Ime *ime);
bool seokdok_key(Ime *ime, int32_t kind, uint32_t ch, bool shift);
void seokdok_flush(Ime *ime);
void seokdok_reset(Ime *ime);
const char *seokdok_take_commit(Ime *ime);
size_t seokdok_preedit_len(Ime *ime);
const char *seokdok_preedit_run(Ime *ime, size_t i, int32_t *attr);
size_t seokdok_popup_len(Ime *ime);
const char *seokdok_popup_item(Ime *ime, size_t i, const char **note);
void seokdok_popup_info(Ime *ime, size_t *selected, size_t *position, size_t *total, bool *horizontal);
int32_t seokdok_mode(Ime *ime);
uint32_t seokdok_key_jamo(uint32_t key);
bool seokdok_ready(void);
bool seokdok_alt_toggles(void);

#ifdef __cplusplus
}
#endif
