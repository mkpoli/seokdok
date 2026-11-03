// Interface strings. Korean is the default; Japanese and English follow the
// browser, and the choice is remembered.

const ko = {
  nav: { ime: '{入力器|입력기}', rules: '{表記|표기}', words: '낱{言|말}', join: '{會|모임}' },
  theme: { light: '{明|밝}게', dark: '{暗|어둡}게', auto: '{自動|자동}' },
  hero: {
    kicker: '풀어읽기모임 {釋讀會|석독회}의 {入力器|입력기}',
    title1: '{韓㐎|한글}로 {打|치}면',
    title2: '{漢字|한자}로 {解|풀}어 {記|적}는다',
    lede: '{漢字語|한자어}는 {音|소리}로, {固有語|고유어}는 {意|뜻}으로. {韓國語|한국어}를 {其|그}대로 {打|치}고 Space를 {押|누르}면 {音讀|음독}・{訓讀|훈독}・{送韓字|송한자}가 한꺼번에 {出|나}옵니다. {入力器|입력기}가 {無|없}어도 {此|이} 페이지에서 바로 {書|써} {見|보}ㄹ 수 {有|있}습니다.',
    try: '바로 {書|써} {見|보}기',
    join: '텔레그램 {會|모임}',
  },
  ime: {
    eyebrow: '{入力器|입력기}',
    title: '{書|써} {見|보}기',
    lede: '{英文|영문} {字板|자판} {其|그}대로 두벌식으로 {打|치}ㅂ니다. {韓國語|한국어} {入力器|입력기}를 {用|쓰}고 {有|있}다면 {其|그}대로 {打|쳐}도 {成|되}ㅂ니다.',
    placeholder: '{此處|여기}를 {押|누르}고 {打|쳐} {見|보}세요. {例|예}: gksmfdmf qhkTek → 하늘을 봤다',
    loading: '{辭典|사전}을 {讀|읽}는 {中|중}',
    failed: '{辭典|사전}을 {讀|읽}지 못했습니다',
    sim: '{英字|영자}→{韓㐎|한글}',
    os: '{英字|영자} {其|그}대로',
    simTip: '{英文|영문} {字板|자판}으로 {打|친} {字|글자}를 두벌식 {韓㐎|한글}로 {變|바꿉}니다. {韓國語|한국어} {入力器|입력기}가 {無|없}을 때 {用|씁}니다.',
    osTip: '{英字|영자}를 {其|그}대로 {打|칩}니다. {自己|자기} {韓國語|한국어} {入力器|입력기}로 {打|칠} 때 {用|씁}니다.',
    keys: {
      space: 'Space {變換|변환} · {次|다음} {候補|후보}',
      tab: 'Tab · PgDn {次|다음} {面|면}',
      digits: '1–9 {選擇|선택}·{確定|확정}',
      arrows: '←→ {節|마디} {移|옮기}기',
      resize: 'Shift+←→ {節|마디} {長|길}이',
      enter: 'Enter {確定|확정}',
      esc: 'Esc {取消|취소}',
      shiftSpace: 'Shift+Space 띄어{書|쓰}기',
    },
    src: {
      gloss: '{字釋|자석}(하늘 천 {類|류})에서',
      corpus: '{會|모임} {用例|용례}',
      community: '{標準國語釋讀辭典|표준국어석독사전}',
      editorial: '{編輯者|편집자}',
      user: '{使用者|사용자} {辭典|사전}',
      sino: '{音讀|음독}',
      kun: '{訓讀|훈독}',
    },
    copy: '{複寫|복사}',
    copied: '{複寫|복사}했습니다',
    clear: '{消|지우}기',
    ruby: '{讀|읽}기 {添|달}기',
    read: '{韓㐎|한글}로 {讀|읽}기',
    preview: '미리{見|보}기',
    keyboard: '{畵面|화면} {字板|자판}',
    convert: '{變換|변환}',
    commit: '{確定|확정}',
  },
  options: {
    title: '{表記|표기} {設定|설정}',
    spaceKey: 'Space {鍵|키}',
    spaceConvert: '{變換|변환} (Tab: {變換|변환})',
    spaceSpace: '띄어{書|쓰}기 (Tab·↓: {變換|변환})',
    style: '{送韓字|송한자}',
    surface: '{表音式|표음식} 見ㅆ다',
    morpheme: '{形態式|형태식} 見았다',
    fragment: '{殘|남}는 {母音|모음}',
    absorb: '{漢字|한자}에 {吸收|흡수} 見라',
    letter: '{字母|자모}로 見ㅏ라',
    coda: '{末音添記|말음첨기} 늘 {書|쓰}기 (有ㅆ다)',
    resyllabify: '받침 {移|넘기}기 (在쓰면)',
    hada: '하다를 爲로 (安寧爲세요)',
  },
  reader: {
    eyebrow: '{讀|읽}기',
    title: '{釋讀文|석독문}을 {韓㐎|한글}로',
    lede: '{釋讀文|석독문}을 {貼|붙여} {入|넣}으면 {讀|읽}는 {音|소리}를 {上|위}에 {添|달}아 {見|보}여 {授|주}ㅂ니다.',
    sample: '會의 規則을 一番씩 讀어 授시면 感謝하겠습니다',
    reading: '{讀|읽}기',
  },
  rules: {
    eyebrow: '{表記|표기}',
    title: '세 가지 {材料|재료}',
    lede: '{文法|문법}은 {韓國語|한국어} {其|그}대로 {置|두}고, {單語|단어}를 {漢字|한자}로 {解|풀}어 {記|적}습니다. {定|정}해진 {規則|규칙}은 {無|없}고 {意|뜻}이 {通|통}하면 {成|되}ㅂ니다(作文 規則 無，意味傳達 第一). {入力器|입력기}는 {會|모임}에서 {實際|실제}로 {用|쓰}인 {表記|표기}를 {從|따르}고, {岐|갈리}는 데는 {設定|설정}으로 {殘|남}겨 {置|두}ㅂ니다.',
    sino: ['音讀', '{漢字語|한자어}는 {音|소리}로', '규칙 → 規則, 환영 → 歡迎'],
    kun: ['訓讀', '{固有語|고유어}는 {意|뜻}으로', '하늘 → 天, 읽다 → 讀다'],
    okuri: ['送韓字', '{殘|나머지}는 {韓㐎|한글}로', '읽었다 → 讀었다, 다른 → 異ㄴ'],
    variants: '{同|같}은 {文章|문장}, {異|다르}ㄴ {表記|표기}',
    variantsLede: '{會|모임} {內|안}에서도 {表記|표기}가 {岐|갈리}ㅂ니다. {設定|설정} {一|하나}로 {變|바뀌}는 {模樣|모양}을 {見|보}세요.',
    variantNames: {
      surface: '{表音式|표음식}',
      morpheme: '{形態式|형태식}',
      letter: '{母音|모음} {字母|자모}',
      coda: '{末音添記|말음첨기}',
      resyllabify: '받침 {移|넘기}기',
    },
  },
  words: {
    eyebrow: '낱{言|말}',
    title: '{會|모임}에서 {多|많}이 {用|쓰}는 {訓讀|훈독}',
    lede: '{對話|대화} {記錄|기록}에서 {自主|자주} {出|나}온 {表記|표기}입니다. {押|누르}면 {入力器|입력기}에 {入|넣}어 {見|보}ㅂ니다.',
    filter: '{韓㐎|한글}이나 {漢字|한자}로 {探|찾}기',
    count: '{回|회}',
  },
  samples: {
    eyebrow: '{例文|예문}',
    title: '{名文|명문}을 {釋讀|석독}으로',
    lede: '{訓民正音|훈민정음}에서 {詩|시}와 {百科|백과}까지, {原文|원문}을 그대로 {入|넣}어 {變換|변환}했습니다. {入力器|입력기}에서 {開|열}면 {節|마디}마다 {他|다른} {候補|후보}를 {見|보}ㄹ 수 {有|있}습니다.',
    kinds: { all: '{全部|전부}', classic: '{古典|고전}', poem: '{詩|시}', wiki: '{百科|백과}', law: '{法|법}' },
    open: '{入力器|입력기}에서 {開|열}기',
    original: '{原文|원문}',
    source: '{出處|출처}',
  },
  join: {
    title: '풀어읽기모임',
    lede: '{漢字|한자}로 {韓國語|한국어}를 {解|풀}어 {記|적}는 {遊|놀}이를 {共|함께}하는 텔레그램 {會|모임}입니다.',
    disclaimer: '※진지한 석독 사용 주장 단체가 아닌, 인공어 감각의 동호회입니다.',
    disclaimerSource: '{會|모임} 포스터에서',
    president: '{會長|회장} 車魄',
    button: 't.me/pimsdh {入|들어}가기',
  },
  windows: {
    title: 'Windows {入力器|입력기}',
    body: 'Windows{用|용} {入力器|입력기}(TSF)를 {準備|준비}하고 {有|있}습니다. {同|같}은 엔진과 {辭典|사전}을 {用|쓰}ㅂ니다.',
  },
  footer: {
    sources: '{辭典|사전}과 {出處|출처}',
    made: '{作|만드}ㄴ {人|이}',
    licence: '코드는 MIT. {辭典|사전}마다 {許可|허가}가 {異|다르}ㅂ니다.',
  },
}

const en = {
  nav: { ime: 'Try', rules: 'Spelling', words: 'Words', join: 'Group' },
  theme: { light: 'Light', dark: 'Dark', auto: 'Auto' },
  hero: {
    kicker: 'The input method of 풀어읽기모임 釋讀會',
    title1: 'Type Hangul,',
    title2: 'write it out in Hanja',
    lede: 'Sino-Korean words by sound, native words by meaning. Type Korean as usual, press Space, and get 音讀, 訓讀 and 送韓字 at once. No input method to install: try it on this page.',
    try: 'Try it',
    join: 'Telegram group',
  },
  ime: {
    eyebrow: 'Input method',
    title: 'Try it',
    lede: 'Type 두벌식 Korean on a Latin keyboard. If you already use a Korean IME, type as usual.',
    placeholder: 'Click here and type, e.g. gksmfdmf qhkTek → 하늘을 봤다',
    loading: 'Loading the dictionaries',
    failed: 'The dictionaries could not be loaded',
    sim: 'QWERTY → Hangul',
    os: 'Keys as typed',
    simTip: 'Turns Latin keys into 두벌식 Hangul. For readers without a Korean IME.',
    osTip: 'Leaves keys as typed. For typing with your own Korean IME.',
    keys: {
      space: 'Space convert · next',
      tab: 'Tab · PgDn next page',
      digits: '1–9 pick and commit',
      arrows: '←→ move between words',
      resize: 'Shift+←→ resize word',
      enter: 'Enter commit',
      esc: 'Esc cancel',
      shiftSpace: 'Shift+Space space',
    },
    src: {
      gloss: 'from a single-character gloss (하늘 천)',
      corpus: 'used in the group chat',
      community: '標準國語釋讀辭典',
      editorial: 'project editors',
      user: 'your dictionary',
      sino: 'by sound',
      kun: 'by meaning',
    },
    copy: 'Copy',
    copied: 'Copied',
    clear: 'Clear',
    ruby: 'Readings',
    read: 'Read as Hangul',
    preview: 'Preview',
    keyboard: 'On-screen keys',
    convert: 'Convert',
    commit: 'Commit',
  },
  options: {
    title: 'Spelling options',
    spaceKey: 'Space key',
    spaceConvert: 'Converts (Japanese style)',
    spaceSpace: 'Types a space (Tab or ↓ converts)',
    style: '送韓字',
    surface: 'Surface 見ㅆ다',
    morpheme: 'Morpheme 見았다',
    fragment: 'Leftover vowel',
    absorb: 'In the Hanja 見라',
    letter: 'As a letter 見ㅏ라',
    coda: 'Always write the final consonant (有ㅆ다)',
    resyllabify: 'Carry it to the next syllable (在쓰면)',
    hada: '爲 for 하다 (安寧爲세요)',
  },
  reader: {
    eyebrow: 'Reading',
    title: '釋讀 back to Hangul',
    lede: 'Paste 釋讀 text to see its reading above each word.',
    sample: '會의 規則을 一番씩 讀어 授시면 感謝하겠습니다',
    reading: 'Reading',
  },
  rules: {
    eyebrow: 'Spelling',
    title: 'Three kinds of material',
    lede: 'The grammar stays Korean; the words are written out in Hanja. The group has no fixed rules, only that the meaning gets across (作文 規則 無，意味傳達 第一). The input method follows how members actually write, and keeps their differences as options.',
    sino: ['音讀', 'Sino-Korean words by sound', '규칙 → 規則, 환영 → 歡迎'],
    kun: ['訓讀', 'Native words by meaning', '하늘 → 天, 읽다 → 讀다'],
    okuri: ['送韓字', 'The rest in Hangul', '읽었다 → 讀었다, 다른 → 異ㄴ'],
    variants: 'One sentence, several spellings',
    variantsLede: 'Members spell differently. Each option below changes one thing.',
    variantNames: {
      surface: 'Surface',
      morpheme: 'Morpheme',
      letter: 'Vowel letter',
      coda: '末音添記',
      resyllabify: 'Resyllabified',
    },
  },
  words: {
    eyebrow: 'Words',
    title: 'The group’s common 訓讀',
    lede: 'The spellings that come up most in the chat. Click one to try it in the input method.',
    filter: 'Search in Hangul or Hanja',
    count: '×',
  },
  samples: {
    eyebrow: 'Samples',
    title: 'Famous texts in 釋讀',
    lede: 'From 訓民正音 to poems and encyclopedia entries, each converted from its original wording. Open one in the input method to see the other candidates for each word.',
    kinds: { all: 'All', classic: 'Classic', poem: 'Poems', wiki: 'Encyclopedia', law: 'Law' },
    open: 'Open in the input method',
    original: 'Original',
    source: 'Source',
  },
  join: {
    title: '풀어읽기모임',
    lede: 'A Telegram group for the game of writing Korean out in Hanja.',
    disclaimer: '※진지한 석독 사용 주장 단체가 아닌, 인공어 감각의 동호회입니다.',
    disclaimerSource: 'from the group’s poster: a hobby club in the spirit of a constructed language',
    president: '車魄, head of the group',
    button: 'Join t.me/pimsdh',
  },
  windows: {
    title: 'Windows IME',
    body: 'A Windows input method (TSF) is in preparation, with the same engine and dictionaries.',
  },
  footer: {
    sources: 'Dictionaries and sources',
    made: 'Made by',
    licence: 'Code under MIT; each dictionary keeps its own licence.',
  },
}

// 'ko': Korean in 釋讀 with Hangul ruby; 'kore': the same, Hanja only; 'en'.
const all = { ko, kore: ko, en }

function initial() {
  try {
    const saved = localStorage.getItem('seokdok:lang')
    if (saved && saved in all) return saved
  } catch {}
  return (navigator.language || 'ko').startsWith('ko') || !(navigator.language || '').startsWith('en') ? 'ko' : 'en'
}

export const i18n = $state({ lang: initial() })

export function setLang(l) {
  i18n.lang = l
  try {
    localStorage.setItem('seokdok:lang', l)
  } catch {}
  document.documentElement.lang = l === 'en' ? 'en' : 'ko'
}

/** The strings for the current language (Korean strings carry {漢字|한자} marks). */
export function t() {
  return all[i18n.lang]
}

const MARK = /\{([^|}]+)\|([^}]+)\}/g
const esc = (s) => s.replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c])

/** Rich text: ruby over each marked word in 한, the bare Hanja in 韓. */
export function r(s) {
  if (i18n.lang === 'ko') return esc(s).replace(MARK, (_, h, k) => `<ruby>${h}<rt>${k}</rt></ruby>`)
  return esc(s).replace(MARK, (_, h) => h)
}

/** Plain text for attributes: Hangul readings in 한, Hanja in 韓. */
export function plain(s) {
  return s.replace(MARK, (_, h, k) => (i18n.lang === 'ko' ? k : h))
}
