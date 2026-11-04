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

/* States of a key for seokdok_mode_key, and its modifier flags. */
enum { SK_KEY_UP = 0, SK_KEY_DOWN, SK_KEY_REPEAT };
enum { SK_MOD_CTRL = 1, SK_MOD_SHIFT = 2, SK_MOD_ALT = 4, SK_MOD_SUPER = 8 };

typedef struct Ime Ime;

Ime *seokdok_new(const char *dict_dir);
void seokdok_free(Ime *ime);
void seokdok_sync(Ime *ime);
bool seokdok_key(Ime *ime, int32_t kind, uint32_t ch, bool shift);
bool seokdok_mode_key(Ime *ime, const char *name, uint32_t mods, int32_t state);
void seokdok_flush(Ime *ime);
void seokdok_reset(Ime *ime);
const char *seokdok_take_commit(Ime *ime);
size_t seokdok_preedit_len(Ime *ime);
const char *seokdok_preedit_run(Ime *ime, size_t i, int32_t *attr);
size_t seokdok_popup_len(Ime *ime);
const char *seokdok_popup_item(Ime *ime, size_t i, const char **note);
void seokdok_popup_info(Ime *ime, size_t *selected, size_t *position, size_t *total, bool *horizontal);
size_t seokdok_list_len(Ime *ime);
const char *seokdok_list_item(Ime *ime, size_t i, const char **reading);
size_t seokdok_list_selected(Ime *ime);
void seokdok_list_can_resize(Ime *ime, bool *shorten, bool *lengthen);
void seokdok_list_pick(Ime *ime, size_t i);
int32_t seokdok_mode(Ime *ime);
void seokdok_set_mode(Ime *ime, int32_t mode);
void seokdok_mode_control(Ime *ime);
uint32_t seokdok_key_jamo(uint32_t key);
bool seokdok_ready(void);
bool seokdok_hangul_only(void);
int32_t seokdok_script(void);
bool seokdok_space_converts(void);
char *seokdok_config_json(void);
bool seokdok_config_save_json(const char *json);
char *seokdok_config_dir(void);
char *seokdok_config_defaults_json(void);
char *seokdok_config_error(void);
char *seokdok_config_update_json(const char *patch);
bool seokdok_chord_valid(const char *text);
char *seokdok_custom_json(void);
char *seokdok_custom_save_json(const char *json);
void seokdok_reload(Ime *ime);
void seokdok_string_free(char *s);

#ifdef __cplusplus
}
#endif
