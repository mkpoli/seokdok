// Interface strings. Korean is the default; Japanese and English follow the
// browser, and the choice is remembered.
import { options } from './options.svelte.js'
import SINO from './sino-chars.json'

const ko = {
  nav: { ime: '{入力器|입력기}', custom: '{自作|자작}', rules: '{表記|표기}', words: '{個言|낱말}', convert: '{變換|변환}', old: '{古|예}ㅅ{韓㐎|한글}', install: '{設置|설치}', join: '{會|모임}' },
  theme: { light: '{明|밝}게', dark: '{暗|어둡}게', auto: '{自動|자동}' },
  hero: {
    kicker: '{釋讀會|풀어읽기모임} {御製|어제}',
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
    hangul: '韓 두벌식',
    ascii: 'A {英字|영자}',
    hangulTip: '{英文|영문} {字板|자판}으로 {打|친} {字|글자}를 두벌식 {韓㐎|한글}로 {變|바꿉}니다. Alt를 한 {番|번} {押|누르}거나 한/영 {鍵|키}로 {轉換|전환}.',
    asciiTip: '{鍵|키}를 {其|그}대로 {打|칩}니다. {自己|자기} {韓國語|한국어} {入力器|입력기}로 {打|칠} {時|때}도 {此|이} {方式|방식}. Alt를 한 {番|번} {押|누르}거나 한/영 {鍵|키}로 {轉換|전환}.',
    hints: [
      ['Space', '{變換|변환} · {次|다음} {候補|후보}'],
      ['Tab|PgDn', '{次|다음} {面|면}'],
      ['1–9', '{選擇|선택}·{確定|확정}'],
      ['←|→', '{節|마디} {移|옮기}기'],
      ['Shift+←|→', '{節|마디} {長|길}이'],
      ['Enter', '{確定|확정}'],
      ['Esc', '{取消|취소}'],
      ['Shift+Space', '띄어{書|쓰}기'],
      ['Alt', '韓 ↔ A'],
    ],
    kind: { sino: '{音讀|음독}', kun: '{訓讀|훈독}' },
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
    layout: '{候補|후보} {窓|창}',
    layoutTip: '{候補|후보} {窓|창}의 {方向|방향}. {橫|가로}이면 ←→로 {候補|후보}를 {移|옮깁}니다.',
    vertical: '{縱|세로}',
    horizontal: '{橫|가로}',
    spaceSwitch: 'Space {變換|변환}',
    spaceTip: '{押|누르}면 Space로 {變換|변환}합니다. {解|끄}면 Space는 띄어{書|쓰}기, Tab·↓로 {變換|변환}.',
    oldSwitch: '{古|예}ㅅ{韓㐎|한글} 겹{字母|자모}',
    oldTip: 'ㅅ ㄱ → ᄭ, ㅂ ㅇ → ㅸ처럼 {現代|현대} {字母|자모}를 {連|이}어 {打|치}면 {古|예}ㅅ 겹{字母|자모}가 {成|됩}니다. ㆍ ㅿ ㆁ ㆆ는 {此|이} {設定|설정}과 {關係|관계}{無|없}이 Shift로 {打|칩}니다.',
    seokdokSide: '{釋讀|석독}',
    sinoSwitch: '{國漢文|국한문} {混用|혼용}',
    sinoTip: '{音讀|음독} {個言|낱말}만 {漢字|한자}로 {書|쓰}고 {固有語|고유어}는 {韓㐎|한글}로 {置|둡}니다. 先生님께서 工夫하셨다',
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
  convert: {
    eyebrow: '{變換|변환}',
    title: '{韓㐎|한글}을 {釋讀文|석독문}으로',
    lede: '{韓㐎|한글} {文|글}을 {貼|붙여} {入|넣}으면 {個言|낱말}마다 {變換|변환}합니다. {個言|낱말}에 마우스를 {置|올리}거나 {押|누르}면 {他|다른} {候補|후보}를 {選|고를} 수 {有|있}습니다.',
    sample: '하늘을 보며 이것저것 생각하는 게 좋아요\n대충 사 주세요',
    input: '{韓㐎|한글} {入力|입력}',
    result: '{變換|변환} {結果|결과}',
  },
  reader: {
    eyebrow: '{讀|읽}기',
    title: '{釋讀文|석독문}을 {韓㐎|한글}로',
    lede: '{釋讀文|석독문}을 {貼|붙여} {入|넣}으면 {讀|읽}는 {音|소리}를 {上|위}에 {添|달}아 {見|보}여 {授|주}ㅂ니다.',
    sample: '會의 規則을 一番씩 讀어 授시면 感謝하겠습니다',
    reading: '{讀|읽}기',
    ruby: '{讀|읽}는 {音|소리}를 {上|위}에',
    plain: '{韓㐎|한글}로만',
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
    eyebrow: '{個言|낱말}',
    title: '{會|모임}에서 {多|많}이 {用|쓰}는 {訓讀|훈독}',
    lede: '{對話|대화} {記錄|기록}에서 {自主|자주} {出|나}온 {表記|표기}입니다. {押|누르}면 {入力器|입력기}에 {入|넣}어 {見|보}ㅂ니다.',
    filter: '{韓㐎|한글}이나 {漢字|한자}로 {探|찾}기',
    count: '{回|회}',
  },
  custom: {
    eyebrow: '{自作|자작}',
    title: '{自己|자기}만의 {規則|규칙}',
    lede: 'RIME처럼 {規則|규칙}과 {個言|낱말}을 {直接|직접} {定|정}합니다. {此|이} {瀏覽器|브라우저}에 {保存|저장}되고, JSON으로 {共有|공유}할 수 {有|있}습니다.',
    tabs: { rules: '{規則|규칙}', phrases: '{自作|자작} {個言|낱말}', words: '{訓讀|훈독} {個言|낱말}', share: '{共有|공유}' },
    rulesHelp: '{上|위}에서부터 {次例|차례}로 {適用|적용}합니다. {入力|입력} {規則|규칙}은 {變換|변환} {前|전}의 {韓㐎|한글}을, {出力|출력} {規則|규칙}은 {候補|후보}마다 {適用|적용}됩니다. 「{加|더하}기」는 {原來|원래} {候補|후보}를 {殘|남}기고 {新|새} {候補|후보}를 {後|뒤}에 {添|붙}입니다. regex에서는 $1 같은 {參照|참조}를 {用|쓸} 수 {有|있}습니다.',
    presets: '{例|예}:',
    on: '{使用|사용}',
    stage: '{段階|단계}',
    stageInput: '入力 (變換 前)',
    stageOutput: '出力 (候補)',
    mode: '{方式|방식}',
    xform: '바꾸기',
    derive: '{加|더}하기',
    pattern: '{探|찾}을 {字|글자}',
    replace: '바꿀 {字|글자}',
    remove: '{削除|삭제}',
    noRules: '{規則|규칙}이 {無|없}습니다. {例|예}를 {押|눌러} {始作|시작}해 {見|보}세요.',
    addRule: '{規則|규칙} {追加|추가}',
    test: '{試驗|시험}',
    phrasesHelp: '{讀|읽}는 {音|소리}에 {定|정}해진 {字|글자}를 {添|붙}입니다(RIME의 custom_phrase). {候補|후보}의 {第一|첫째}로 {出|나}옵니다.',
    reading: '{讀音|독음} ({韓㐎|한글})',
    text: '{表記|표기}',
    noPhrases: '{自作|자작} {個言|낱말}이 {無|없}습니다.',
    addPhrase: '{個言|낱말} {追加|추가}',
    wordsHelp: '{訓讀|훈독} {個言|낱말}은 {活用|활용}합니다. {例|예}: 짓다 → 作다 {動詞|동사}이면 지었다는 作었다로 {變換|변환}됩니다. {表記|표기}의 {規則|규칙}은 {釋讀辭典|석독사전}과 {同|같}습니다(初음, 在ㅆ다).',
    lemma: '{基本形|기본형}',
    written: '{釋讀|석독} {表記|표기}',
    pos: '{品詞|품사}',
    posNames: { noun: '{名詞|명사}', pron: '{代名詞|대명사}', adv: '{副詞|부사}', det: '{冠形詞|관형사}', verb: '{動詞|동사}', adj: '{形容詞|형용사}' },
    noWords: '{訓讀|훈독} {個言|낱말}이 {無|없}습니다.',
    addWord: '{個言|낱말} {追加|추가}',
    shareHelp: '{規則|규칙}과 {個言|낱말}을 JSON {文書|문서}로 {內|내}보내거나 {入|들}여옵니다. {入|들}여오면 {今|지금}의 {設定|설정}을 {代身|대신}합니다.',
    download: 'JSON {內|내}려받기',
    copyJson: 'JSON {複寫|복사}',
    openFile: '{文書|문서} {開|열}기',
    import: '{入|들}여오기',
  },
  samples: {
    eyebrow: '{例文|예문}',
    title: '{名文|명문}을 {釋讀|석독}으로',
    lede: '{訓民正音|훈민정음}에서 {詩|시}와 {百科|백과}까지, {原文|원문}을 {其|그}대로 {入|넣}어 {變換|변환}했습니다. {入力器|입력기}에서 {開|열}면 {節|마디}마다 {他|다른} {候補|후보}를 {見|보}ㄹ 수 {有|있}습니다.',
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
    president: '{會長|회장} {車魄|차백}',
    button: 't.me/pimsdh {入|들어}가기',
  },
  old: {
    eyebrow: '{古|예}ㅅ{韓㐎|한글}',
    title: 'ㆍ와 {古|예}ㅅ {字母|자모} {打|치}기',
    lede: 'Microsoft {古|예}ㅅ{韓㐎|한글} {字板|자판}처럼 Shift {段|단}에 {古|예}ㅅ {字母|자모}가 {有|있}습니다. {此|이} {頁|쪽}의 {入力器|입력기}와 Windows·Linux {入力器|입력기}가 {皆|모두} {同|같}습니다.',
    mapCaption: '{各|각} {鍵|글쇠} {右|오른}쪽 {上|위}의 {小|작}은 {字|글자}는 Shift를 {押|누르}고 {打|치}는 {字母|자모}이고, {古|예}ㅅ {字母|자모}는 {色|색}으로 {表示|표시}했습니다. {入力器|입력기}의 {畵面|화면} {字板|자판}에서는 ⇧를 {押|누른} {後|뒤} {鍵|글쇠}를 {押|누릅}니다.',
    shift: 'Shift로 {打|치}는 {字母|자모}',
    names: {
      araea: '{下|아래}아',
      pansios: '{半齒音|반치음}',
      yesieung: '옛이응',
      yeorinhieuh: '{弱|여린}히읗',
      chitueumS: '{齒頭音|치두음} ㅅ',
      ceongchieumS: '{正齒音|정치음} ㅅ',
      chitueumJ: '{齒頭音|치두음} ㅈ',
      ceongchieumJ: '{正齒音|정치음} ㅈ',
      chitueumCh: '{齒頭音|치두음} ㅊ',
      ceongchieumCh: '{正齒音|정치음} ㅊ',
    },
    examples: '{例|예}',
    clusters: '겹{字母|자모}',
    clustersNote: '{現代|현대} {字母|자모}를 {連|이}어 {打|쳐} {古|예}ㅅ 겹{字母|자모}를 {作|만드}는 {事|것}은 「{古|예}ㅅ{韓㐎|한글} 겹{字母|자모}」를 {押|켜}야 {成|됩}니다. {入力器|입력기} {窓|창}의 {道具|도구} {欄|줄}, Windows {設定|설정} {窓|창}, {或|또}는 config.toml의 old_clusters = true.',
    conversion: '{古|예}ㅅ{韓㐎|한글} {音節|음절}은 {打|친} {其|그}대로 {殘|남}고, {其|그} {前後|앞뒤}의 {現代|현대} {韓㐎|한글}만 {漢字|한자}로 {變換|변환}됩니다.',
    fonts: '{此|이} {頁|쪽}은 {古|예}ㅅ{韓㐎|한글} 글꼴을 {自|스스로} {讀|불러}옵니다. {他|다른} 앱에서는 Noto Serif CJK KR, Noto Sans CJK KR, 함초롬바탕처럼 {古|예}ㅅ{韓㐎|한글}을 {支援|지원}하는 글꼴이 {要|필요}합니다.',
  },
  install: {
    eyebrow: '{設置|설치}',
    title: '{自己|제} 컴퓨터에서 {書|쓰}기',
    lede: '{此|이} {頁|쪽}과 {同|같}은 엔진과 {辭典|사전}을 {用|쓰}는 {入力器|입력기}입니다. Windows와 Linux(fcitx5)를 {支援|지원}합니다.',
    download: '{內|내}려받기',
    windows: [
      '{內|내}려받은 {設置|설치} 파일을 {實行|실행}합니다. {管理者|관리자} {權限|권한}을 {求|구}합니다.',
      'Win+Space로 釋讀을 {選|고르}ㅂ니다. {韓國語|한국어}가 {言語|언어} {目錄|목록}에 {無|없}으면 {自動|자동}으로 {加|더}합니다.',
    ],
    linux: [
      '{壓縮|압축}을 {解|풀}고 install.sh를 {實行|실행}합니다. sudo {無|없}이 {此|이} {使用者|사용자}에게만 {設置|설치}합니다.',
      'fcitx5를 {再|다시} {始作|시작}하고, fcitx5-configtool에서 「Only Show Current Language」를 {解除|해제}한 {後|뒤} 釋讀을 {加|더}합니다.',
    ],
    system: '{全|모든} {使用者|사용자}에게:',
    remove: '{削除|삭제}:',
    keys: 'Space {變換|변환} · Space {再|다시} {候補|후보} · 1–9 {選擇|선택} · Enter {確定|확정} · Esc {取消|취소} · 한/영 {或|또}는 Alt {短|짧}게: 釋讀 ↔ A',
    custom: '{自作|자작} {規則|규칙}과 {個言|낱말}은 {上|위} 「{自作|자작}」의 {共有|공유}에서 JSON으로 {內|내}려받아 custom.json으로 {保存|저장}합니다.',
    where: '{置|두}는 {處|곳}',
    note: '{同|같}은 {處|곳}의 config.toml에서 {設定|설정}을 {變|바꾸}ㅂ니다. {入力器|입력기}를 {變|바꾸}었다가 {返|돌아}오면 {再|다시} {讀|읽}습니다.',
  },
  footer: {
    sources: '{辭典|사전}과 {出處|출처}',
    suggested: '{機械|기계} {提案|제안}: {言語|언어} 모델이 {意|뜻}으로 {付|붙}인 {漢字|한자}, {人|사람}이 {檢討|검토}하지 {不|않}음',
    made: '{作|만드}ㄴ {人|이}',
    licence: '코드는 MIT. {辭典|사전}마다 {許可|허가}가 {異|다르}ㅂ니다.',
  },
}

const en = {
  nav: { ime: 'Try', custom: 'Customize', rules: 'Spelling', words: 'Words', convert: 'Convert', old: 'Old Hangul', install: 'Install', join: 'Group' },
  theme: { light: 'Light', dark: 'Dark', auto: 'Auto' },
  hero: {
    kicker: '釋讀會 御製',
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
    hangul: '韓 Hangul',
    ascii: 'A ASCII',
    hangulTip: 'Latin keys type 두벌식 Hangul. Tap Alt or press 한/영 to switch.',
    asciiTip: 'Keys as typed: Latin text, or your own Korean IME. Tap Alt or press 한/영 to switch.',
    hints: [
      ['Space', 'convert · next'],
      ['Tab|PgDn', 'next page'],
      ['1–9', 'pick and commit'],
      ['←|→', 'move between words'],
      ['Shift+←|→', 'resize word'],
      ['Enter', 'commit'],
      ['Esc', 'cancel'],
      ['Shift+Space', 'space'],
      ['Alt', '韓 ↔ A'],
    ],
    kind: { sino: 'by sound', kun: 'by meaning' },
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
    layout: 'Candidate list',
    layoutTip: 'Direction of the candidate list. When horizontal, ← → move through candidates.',
    vertical: 'Vertical',
    horizontal: 'Horizontal',
    spaceSwitch: 'Space converts',
    spaceTip: 'On: Space converts. Off: Space types a space and Tab or ↓ converts.',
    oldSwitch: 'Old clusters',
    oldTip: 'Modern letters typed in a row join into old clusters: ㅅ ㄱ → ᄭ, ㅂ ㅇ → ㅸ. ㆍ ㅿ ㆁ ㆆ are on Shift either way.',
    seokdokSide: '釋讀',
    sinoSwitch: '國漢文混用',
    sinoTip: 'Hanja for Sino-Korean words only; native words stay in Hangul: 先生님께서 工夫하셨다.',
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
  convert: {
    eyebrow: 'Convert',
    title: 'Hangul text to 釋讀',
    lede: 'Paste Hangul text and every word is converted. Hover over a word, or tap it, to pick another candidate.',
    sample: '하늘을 보며 이것저것 생각하는 게 좋아요\n대충 사 주세요',
    input: 'Hangul input',
    result: 'Result',
  },
  reader: {
    eyebrow: 'Reading',
    title: '釋讀 back to Hangul',
    lede: 'Paste 釋讀 text to see its reading above each word.',
    sample: '會의 規則을 一番씩 讀어 授시면 感謝하겠습니다',
    reading: 'Reading',
    ruby: 'With readings above',
    plain: 'Hangul only',
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
  custom: {
    eyebrow: 'Customize',
    title: 'Your own rules',
    lede: 'Define rules and words yourself, much like RIME. They are saved in this browser and can be shared as JSON.',
    tabs: { rules: 'Rules', phrases: 'Phrases', words: '訓讀 words', share: 'Share' },
    rulesHelp: 'Rules run top to bottom. Input rules rewrite the Hangul before conversion; output rules rewrite every candidate. “Add” keeps the original candidate and puts the rewritten one after it. Regex replacements can use $1 and the like.',
    presets: 'Examples:',
    on: 'On',
    stage: 'Stage',
    stageInput: 'Input (before converting)',
    stageOutput: 'Output (candidates)',
    mode: 'Mode',
    xform: 'Replace',
    derive: 'Add',
    pattern: 'Find',
    replace: 'Replace with',
    remove: 'Remove',
    noRules: 'No rules yet. Start from an example.',
    addRule: 'Add rule',
    test: 'Try',
    phrasesHelp: 'Fixed text for a reading (RIME’s custom_phrase). It comes first among the candidates.',
    reading: 'Reading (Hangul)',
    text: 'Written',
    noPhrases: 'No phrases yet.',
    addPhrase: 'Add phrase',
    wordsHelp: '訓讀 words inflect. With 짓다 → 作다 as a verb, 지었다 converts to 作었다. The written form follows the dictionary format (初음, 在ㅆ다).',
    lemma: 'Lemma',
    written: '釋讀 form',
    pos: 'Part of speech',
    posNames: { noun: 'noun', pron: 'pronoun', adv: 'adverb', det: 'determiner', verb: 'verb', adj: 'adjective' },
    noWords: 'No 訓讀 words yet.',
    addWord: 'Add word',
    shareHelp: 'Export your rules and words as a JSON file, or import one. Importing replaces the current settings.',
    download: 'Download JSON',
    copyJson: 'Copy JSON',
    openFile: 'Open file',
    import: 'Import',
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
  old: {
    eyebrow: 'Old Hangul',
    title: 'Typing ㆍ and the old letters',
    lede: 'As on Microsoft Old Hangul, the old letters sit on the Shift layer. The same keys work on this page and in the Windows and Linux input methods.',
    mapCaption: 'The small letter in each key’s corner is typed with Shift; old letters are coloured. On the on-screen keyboard above, tap ⇧ and then the key.',
    shift: 'Letters on Shift',
    names: {
      araea: 'arae-a',
      pansios: 'pansios (bansiot)',
      yesieung: 'old ieung',
      yeorinhieuh: 'yeorin hieuh',
      chitueumS: 'dental ㅅ (chidueum)',
      ceongchieumS: 'retroflex ㅅ (jeongchieum)',
      chitueumJ: 'dental ㅈ (chidueum)',
      ceongchieumJ: 'retroflex ㅈ (jeongchieum)',
      chitueumCh: 'dental ㅊ (chidueum)',
      ceongchieumCh: 'retroflex ㅊ (jeongchieum)',
    },
    examples: 'Examples',
    clusters: 'Clusters',
    clustersNote: 'Joining modern letters into old clusters needs “Old clusters” switched on: in the input method’s toolbar, in the Windows settings window, or old_clusters = true in config.toml.',
    conversion: 'Old syllables stay as typed; only the modern Hangul around them is converted to Hanja.',
    fonts: 'This page loads its own old-Hangul font. Other apps need a font that supports old Hangul, such as Noto Serif CJK KR, Noto Sans CJK KR or HCR Batang.',
  },
  install: {
    eyebrow: 'Install',
    title: 'Type it on your own computer',
    lede: 'A system input method with the same engine and dictionaries as this page, for Windows and for Linux (fcitx5).',
    download: 'Download',
    windows: [
      'Run the downloaded setup. It asks for administrator rights.',
      'Pick 釋讀 with Win+Space. Korean is added to your language list if it is missing.',
    ],
    linux: [
      'Unpack and run install.sh. It installs for your user only, without sudo.',
      'Restart fcitx5, then add 釋讀 in fcitx5-configtool (untick “Only Show Current Language” to find it).',
    ],
    system: 'For all users:',
    remove: 'Uninstall:',
    keys: 'Space converts · Space again lists candidates · 1–9 pick · Enter confirms · Esc cancels · Han/Eng or a tap of Alt: 釋讀 ↔ A',
    custom: 'For your own rules and words, download the JSON from Share under Customize above and save it as custom.json.',
    where: 'Location',
    note: 'Settings live in config.toml in the same folder. Switch input method away and back to reload.',
  },
  footer: {
    suggested: 'Machine suggestions: Hanja assigned by meaning by a language model, not reviewed by a person',
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

const MARK = /\{([^|}]+)\|([^}]+)\}([ㄱ-ㅎ])?/g
const esc = (s) => s.replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c])

/** A mark read by its characters' own Sino-Korean readings (音讀). */
export function isSino(hanja, reading) {
  const h = [...hanja]
  const k = [...reading]
  return h.length === k.length && h.every((c, i) => SINO[c]?.includes(k[i]))
}

// Final consonants as they follow a syllable without one (되 + ㅂ → 됩).
const FINALS = ' ㄱㄲㄳㄴㄵㄶㄷㄹㄺㄻㄼㄽㄾㄿㅀㅁㅂㅄㅅㅆㅇㅈㅊㅋㅌㅍㅎ'
function attach(word, jamo) {
  if (!jamo) return word
  const last = word.codePointAt(word.length - 1) - 0xac00
  const f = FINALS.indexOf(jamo)
  if (last < 0 || last > 11171 || last % 28 !== 0 || f < 1) return word + jamo
  return word.slice(0, -1) + String.fromCodePoint(0xac00 + last + f)
}

/** In 國漢文混用 mode a 訓讀 mark shows its Hangul reading. */
const hangulOnly = (h, k) => options.sinoOnly && !isSino(h, k)

function render(s, ruby, html) {
  const text = html ? esc(s) : s
  return text.replace(MARK, (_, h, k, j = '') => {
    if (hangulOnly(h, k)) return attach(k, j)
    if (!html) return (ruby ? k : h) + j
    return (ruby ? `<ruby>${h}<rt>${k}</rt></ruby>` : h) + j
  })
}

/** Rich text: ruby over each marked word in 한, the bare Hanja in 韓. */
export function r(s) {
  return render(s, i18n.lang === 'ko', true)
}

/** Rich text with ruby in every language (the hero slogan). */
export function rk(s) {
  return render(s, i18n.lang !== 'kore', true)
}

/** Plain text for attributes: Hangul readings in 한, Hanja in 韓. */
export function plain(s) {
  return render(s, i18n.lang === 'ko', false)
}
