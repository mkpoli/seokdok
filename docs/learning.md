# Learning what a person picks

A candidate picked twice for a reading comes first for that reading. The code is
`crates/seokdok-session/src/learned.rs`; every front end goes through it.

## The rule

The table holds (reading, written form) entries, each counted separately.

- Committing a candidate other than the first counts one pick for that entry.
  The commit can come from a digit, Enter, the next key or a loss of focus.
  Cancelling counts nothing.
- A form with two picks comes first. Of several such forms, the one that reached
  its second pick last wins. So two recent picks beat any number of old ones, and
  one recent pick changes nothing.
- Committing the first candidate stores nothing for a reading with no entries.
  For a reading that has entries, it counts for that form if the form has an entry, and
  it drops rivals that have one pick. An old slip and a new slip therefore do not
  add up to two picks.
- The threshold is 2 (`THRESHOLD`). At 1 a slip would reorder the list. At 3 people
  would correct the same word three times before it sticks.

The rest of the candidate list keeps its order. A form that the list does not
offer at the moment (under another script, for example) has no effect.

## What it applies to

The reading is matched whole, at the level of a segment, so picks for 나 say
nothing about 나라. They say nothing about 나는 either: 나는 is its own reading,
with 我는 as its own form, and has its own count.

A piece of a longer run of Hangul that the engine left in Hangul is skipped: the 나
of 나봐요 or 나올겁니다 is the rest of a word the engine did not recognise, and
我 there would cut it. Alone in its word, or where the engine converts it, the pick
applies.

A span the person made by resizing is learned like a segment. At the next
conversion, adjacent segments whose readings join into a learned span are merged
into it, up to four at a time. Only whole segments merge, so a word inside a
segment is never cut.

An empty table leaves conversion exactly as it was, and so does a reading the
table does not hold.

## What is kept

`learned.tsv`, beside `config.toml`, in the folder `Config::user_dir` names:

```
# seokdok learned picks 1
나	我	2	1	2
```

The columns are reading, form, count, the previous pick and the last pick. The last
two are the table's own counter, with no clock times. A reading must be all Hangul,
with at most 12 syllables and no spaces, and a form has at most 24, so numbers and
long or spaced phrases are not kept. A short phrase typed without spaces looks like
a word and can be kept.

The table holds 4096 entries and evicts the oldest pick. Full, with four-syllable
words, it takes about 400 KiB of memory and 140 KiB on disk
(`crates/seokdok-session/tests/learned_memory.rs` measures this).

A file that is not exactly this format, or is over 2 MiB, is ignored whole and
replaced by the next save.

## Saving

Typing touches only the table in memory. A background thread writes about a second
after a pick: it writes a temporary file, flushes it to disk and renames it over
the old file, so a crash leaves one version or the other and can lose at most that
last second of picks. On Unix the file is readable only by its owner.

Each application hosts its own input method on Windows, so several processes can
share the file. A save reads the file, applies this process's picks on top, and
tries again later if the file changed in the meantime. If the file is gone, someone
chose to forget, so the waiting picks go too. Each conversion checks whether the
file changed and reloads it in the background. Two saves in the same instant can
still lose one's picks; the file stays whole.

A folder that cannot be written costs nothing visible: the table goes on working in
memory until the process ends, and each later pick tries the file again.

## Turning it off and forgetting

`input.learn = false` stops both counting and ranking, and frees the table. The
file stays until it is forgotten.

"Forget what I picked" deletes the file and any temporary file a crash left, and
empties this process's table. Other processes see the file gone at their next
conversion.

| Front end | Switch | Forget |
|---|---|---|
| Windows | settings window | settings window |
| fcitx5 | `config.toml` | delete `learned.tsv` |
| macOS | settings window | settings window |
| Android | settings screen | settings screen |
| iOS, Full Access on | app | app |
| iOS, Full Access off | hold the 釋 key | hold the 釋 key |
| Site | input settings | input settings |

On iOS without Full Access (or with a signature that grants no App Group), the
keyboard keeps the file in its own container, at `~/.config/seokdok/learned.tsv`,
because it cannot reach the shared one. The app cannot reach that file, so the
keyboard has the controls itself. When Full Access is turned on, the keyboard moves
the file to the shared container if none is there, and deletes it otherwise. So the
app's button clears everything that is in reach. If Full Access is later turned off,
the shared file stays until the app clears it. If the keyboard's own container cannot
be written, what it learned lasts until the keyboard's process ends.

The site keeps the same table as text under `seokdok:learned` in `localStorage`,
through the `Learned` class of `crates/seokdok-wasm`.

## Fields it stays out of

In these the table is neither read nor added to.

| Front end | How |
|---|---|
| Windows | input scopes for password, private, email, URL, number and telephone; the disabled and empty-context compartments; any field whose scopes cannot be read |
| fcitx5 | the capability flags Password, Sensitive, Email, Url, Digit, Number, Dialable |
| Android | password, email, URL, number, phone and date fields never reach the session; `IME_FLAG_NO_PERSONALIZED_LEARNING` also counts |
| iOS | only the standard keyboard type reaches the session; content types for passwords, codes, usernames, email, URL, telephone and card numbers also count |
| macOS | secure event input only: an input method is not told about other field types, so email, URL and number fields can be learned from |

All of this relies on the application naming the kind of field.

Front ends set this with `Session::set_private_field`, `seokdok_set_private_field`
in C, or `Native.setPrivateField` on Android.
