# Changelog

Changes that people typing or reading with 釋讀 can notice, by the date they reached `main`. The site follows `main`. Installers follow when they are next rebuilt, so a download can be behind the newest entry. Every commit is in `git log`.

## 2026-11-06

### Settings
- Modifier keys are named 左側 and 右側: 左側 Ctrl, 右側 Alt.

### Windows
- On a Japanese keyboard layout, ASCII no longer turns into half-width katakana when the kana lock is on.

### Conversion
- 사라지다 keeps its inner Hangul: 消라져去고, 消라졌다.
- 후 comes first as 後: 千年後에는, 後에도.
- Well-known people's names convert whole: 金正恩同志, 文在寅, 李世乭. Hanja come from their Korean Wikipedia articles; `dict/people.tsv` lists them.
- A loanword inside a longer word stays Hangul: 深化버전, 웹페이지. Its Hanja are offered only when it is typed alone.
- 하다 and 되다 after a word stay Hangul by default: 表示하는, 發音되는. 爲 and 成 stay on offer, and a setting puts them first: 表示爲는, 發音成는.
- A Sino-Korean word beats a weak verb row: 選擇하도록, 改善하였어요, not 選도록, 善하였어요.

### Dictionary
- 아우롭다 is written 悌롭다, a modern form coined from the old 아ᅀᆞᄅᆞ외다.
- Common loanwords offer Hanja behind the Hangul: 페이지 頁 丁 張, 컴퓨터 電腦, 커피 珈琲, 러시아 露西亞.

### iOS
- The keyboard follows Apple's more closely: held Delete at 10/s and then by words, Space trackpad timing, 123 switching on landing, landscape height, margin taps, double-space period, smart quotes, Return labels and field types.
- Every key with a corner letter offers it on a long press, so ㅿ, ㆁ, ㆆ and ㆍ need no Shift.
- The memory leak across keyboard launches is fixed.

### 音讀
- A second input method, 音讀 (Eumdok), for 國漢文混用 only: Sino-Korean words in Hanja, native words in Hangul, with no 訓讀 in the candidates or settings. It installs beside 釋讀 with its own settings on Windows, Linux, macOS, Android and iOS.
- 音讀 has its own site, eumdok.mkpo.li, with readings at the bottom right of each Hanja, a 韓 mode without them, a box to type in, and 國漢文混用 mode writes Sino-Korean verbs in Hanja: 推薦해, 可能합니다, 複雜한.

### Conversion
- Numerals typed as their own word rank high: 삼 and 세 give 三, 이 and 사 offer 二 and 四 second, and 세 사람 gives 三 人, 오 분 gives 五 分.

### Site
- seokdok.mkpo.li/dict lists every dictionary row: search by Hangul reading or Hanja, filter by source file, and read what each source is and its licence.
- The site honours the hand-set homophone choices it had been dropping: 구사 驅使, 의자 椅子, 지도 地圖.

### Windows
- The candidate window can look like Windows 11 or the classic square list of Windows 10 and earlier, and can follow the system or stay light or dark. Both are in the settings app and `config.toml`, and apply at the next key.

### iOS and macOS
- The site offers the macOS input method as a zip and the iOS app as an IPA, so neither needs Xcode.

## 2026-11-05

### Conversion
- Helper verbs typed without a space keep their verb: 보고싶어요 → 見고欲어요, 하고있어요 → 하고有어요, 먹지않아 → 食지不ㅎ아.
- The next verb decides between homophones when both words are converted together: 눈이 내린다 → 雪이, 눈이 아프다 → 目이, 배가 고프다 → 腹.
- Endings that run on from a stem convert: 안타깝게도 → 惜깝게도, 없긴한데 → 無긴한데, 된듯요 → 成ㄴ듯요, 써야하는지 → 書야하는지, 모르다라는 → 不知다라는.
- The copula stays Hangul after any noun: 親舊인줄, 開發者인데도.
- 그렇- forms write ㅎ where the word keeps it: 然ㅎ군요, 然ㅎ죠.
- Compound words offer 其, 此 or 彼 plus the word: 其話를, 其火를. 하나 typed alone gives 一.
- The IME learns the candidates a person picks often. The setting is on by default and the picks can be cleared.
- The mode chord rotates 釋讀 → 國漢文混用 → 韓㐎 專用.

### Dictionary
- The common word comes first across homophones: 음성 音聲, 신상 身上, 기물 器物, 진사 進士, 지도 地圖, 책 冊, 고수 高手, 빈도 頻度.
- Spellings the chat writes most lead: 感謝합니다, 主로, 故로, 乃終에, 例를, 房에, 點이, 式으로, 外에, 妙한. 거예요 stays Hangul first.
- 눈썹 gives 眉.
- A whole loanword is not cut into pieces: 버그를 no longer offers 버其를.
- 나 gives 我 first.
- Native place names from a 1900s gazetteer.

### Site
- A 字 button in the top bar puts readings above or, as 文書문서, to the right at half size, on every page.
- The converter shows one space between words.
- The top bar's section links scroll inside the bar, so narrow windows no longer scroll sideways.
- Tapping moves between conversion segments.
- Dictation no longer doubles its text.

### Windows
- A WinUI 3 settings app replaces the old dialog.
- Plain Hangul composes inline in Telegram and other IMM32 apps. The floating box sits at the caret in Qt apps.
- The taskbar menu shows each mode's keys.
- Tapping the mode chord keeps working after a modifier's release is lost.

### iOS
- The app is rebuilt as native settings.
- One bar of system height holds segments and candidates, with readings beside each candidate and a drawer for the rest.
- Keys have the system press highlight, pop-up preview, click and tap. Holding Space moves the caret.
- The first jamo survives when a document answers slowly.

### Android
- The unused FFI library is left out of the APK.

## 2026-11-04

### Conversion
- Words already written in Hanja convert, and old Hangul is accepted as input (텬하, 갓다, 거ᅀᅴ, ᄆᆞᅀᆞᆷ).
- A character can be found by its gloss: 살찔비 → 肥.
- Compound nouns offer every combination. Common Sino-Korean suffixes attach to whole words: 표시용 → 表示用.
- Plural 들 offers 等, 們 and 達, and takes more particles. Past 았/었 offers ㅆ. 이 offers the ㅣ spelling.
- Every 末音添記 choice is listed for single-Hanja words, and the old spellings ᄒᆞ, ᄂᆞᆫ and ᄅᆞᆯ sit behind the modern ones.
- Loanwords stay Hangul. A particle typed alone stays Hangul.
- Peak memory fell to 44.5 MB.

### Dictionary
- 訓讀 readings from 諺解 and primers (新增類合, 訓蒙字會), 勞動夜學讀本 and Wiktionary, each with its source.
- The bound noun 데, the demonstrative family, interrogatives, 저희 as 吾等 自等 我等 僕等, 頗 for 꽤.
- A setting chooses how 있다 is written first: 有, 在 or 在ㅆ.

### Settings
- A script setting chooses 釋讀, 國漢文混用 or 韓㐎 專用 on every platform.
- Each platform gets mode keys from its own Korean tools.
- Inline or floating composition can be chosen per application.

### Site
- The privacy and support pages, written in Hanja.
- Fonts come from the site itself, and the pages load nothing from other servers.
- A switch in the top bar moves between 釋讀 and 國漢文混用.
- Android and iOS install guides, and the signed APK.
- The textarea composes with every keyboard. Pasted Hanja converts in the input method.

### Windows
- Candidates are drawn with Direct2D in the Windows 11 style.
- A taskbar menu sets the script mode and quick settings.
- Updating replaces the DLL without closing apps or a reboot.
- Text no longer goes to the wrong field or appears twice when a composition is cut short.

### macOS, iOS and Android
- First versions of the macOS input method (InputMethodKit), the iOS keyboard and its app, and the Android keyboard with its settings.
- iOS has number, decimal, phone, email and URL layouts and bundles a font for old letters.
- One 釋 icon on every platform.

## 2026-11-03

First public version.

- The Rust engine: 音讀 and 訓讀 lookup, 末音添記, particles and endings, and ranked candidates.
- seokdok.mkpo.li: an input method in the browser, a converter, samples, the old-Hangul guide, and readers' own words and rewrite rules.
- Windows (TSF) and Linux (fcitx5) input methods with installers.
- Old Hangul composition in the 두벌식 composer.
- Dictionaries from the 釋讀會 chat's counts, the community dictionary, libhangul glosses, Japanese 訓読み links and Wiktionary.
