// Smoke test for the built bundle: the dictionaries are in
// Contents/Resources/dict and the Rust engine, linked as libseokdok_ffi.a,
// converts 안녕 (typed as dkssud) to 安寧 through the C interface.
//
//   smoke <Seokdok.app>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include "seokdok.h"

int main(int argc, char **argv) {
    if (argc != 2) {
        fprintf(stderr, "usage: smoke Seokdok.app\n");
        return 2;
    }
    char dict[4096];
    snprintf(dict, sizeof dict, "%s/Contents/Resources/dict", argv[1]);
    char probe[4200];
    snprintf(probe, sizeof probe, "%s/sino.tsv", dict);
    if (access(probe, R_OK) != 0) {
        fprintf(stderr, "missing %s\n", probe);
        return 1;
    }
    Ime *ime = seokdok_new(dict);
    for (int i = 0; i < 400 && !seokdok_ready(); i++) usleep(50000);
    if (!seokdok_ready()) {
        fprintf(stderr, "dictionaries did not load\n");
        return 1;
    }
    for (const char *k = "dkssud"; *k; k++) seokdok_key(ime, SK_LETTER, (uint32_t)*k, false);
    seokdok_key(ime, SK_SPACE, 0, false);
    seokdok_key(ime, SK_ENTER, 0, false);
    const char *out = seokdok_take_commit(ime);
    printf("commit: %s\n", out ? out : "(none)");
    int ok = out && strcmp(out, "安寧") == 0;
    seokdok_free(ime);
    return ok ? 0 : 1;
}
