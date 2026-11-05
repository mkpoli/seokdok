// Interface strings. Korean is the default; Japanese and English follow the
// browser, and the choice is remembered.
import { options } from './options.svelte.js'
import SINO from './sino-chars.json'
import pages from './pages.js'

const ko = {
  nav: { ime: '{入力器|입력기}', custom: '{自作|자작}', rules: '{表記|표기}', words: '{個言|낱말}', convert: '{變換|변환}', old: '{古|예}ㅅ{韓㐎|한글}', install: '{設置|설치}', eumdok: '{音讀|음독}', join: '{會|모임}' },
  theme: { light: '{明|밝}게', dark: '{暗|어둡}게', auto: '{自動|자동}' },
  hero: {
    kicker: '{釋讀會|풀어읽기모임} {御製|어제}',
    title1: '{韓㐎|한글}로 {打|치}면',
    title2: '{漢字|한자}로 {解|풀}어 {記|적}는다',
    lede: '{漢字語|한자어}는 {音|소리}로, {固有語|고유어}는 {意|뜻}으로. {韓國語|한국어}를 {其|그}대로 {打|치}고 Space를 {押|누르}면 {音讀|음독}・{訓讀|훈독}・{末音添記|말음첨기}가 한꺼번에 {出|나}옵니다. {入力器|입력기}가 {無|없}어도 {此|이} 페이지에서 {直|바로} {書|써} {見|보}ㄹ {手|수} {有|있}습니다.',
    try: '{直|바로} {書|써} {見|보}기',
    join: '텔레그램 {會|모임}',
  },
  ime: {
    eyebrow: '{入力器|입력기}',
    title: '{書|써} {見|보}기',
    lede: '{英文|영문} {字板|자판} {其|그}대로 두벌{式|식}으로 {打|치}ㅂ니다. {韓國語|한국어} {入力器|입력기}를 {用|쓰}고 {有|있}다면 {其|그}대로 {打|쳐}도 {成|되}ㅂ니다.',
    placeholder: '{此處|여기}를 {押|누르}고 {打|쳐} {見|보}세요. {例|예}: gksmfdmf qhkTek → 하늘을 봤다',
    loading: '{辭典|사전}을 {讀|읽}는 {中|중}',
    failed: '{辭典|사전}을 {讀|읽}지 못했습니다',
    hangul: '{韓|한} 두벌{式|식}',
    ascii: 'A {英字|영자}',
    hangulTip: '{英文|영문} {字板|자판}으로 {打|치}ㄴ {字|글자}를 두벌{式|식} {韓㐎|한글}로 {變|바꾸}ㅂ니다. Alt를 한 {番|번} {押|누르}거나 한/영 키로 {轉換|전환}.',
    asciiTip: '키를 {其|그}대로 {打|치}ㅂ니다. {自己|자기} {韓國語|한국어} {入力器|입력기}로 {打|치}ㄹ {時|때}도 {此|이} {方式|방식}. Alt를 한 {番|번} {押|누르}거나 한/영 키로 {轉換|전환}.',
    hints: [
      ['Space', '{變換|변환} · {次|다음} {候補|후보}'],
      ['Tab|PgDn', '{次|다음} {面|면}'],
      ['1–9', '{選擇|선택}·{確定|확정}'],
      ['←|→', '{節|마디} {移|옮기}기'],
      ['Shift+←|→', '{節|마디} {長|길}이'],
      ['Enter', '{確定|확정}'],
      ['Esc', '{取消|취소}'],
      ['Shift+Space', '띄어{書|쓰}기'],
      ['Alt', '{韓|한} ↔ A'],
      ['살찔비|비살찔', '{訓音|훈음}으로 {字|글자} {探|찾}기 (肥)'],
      ['{設置|설치}{版|판} Windows'],
      ['한/영|Right Alt|半角/全角', '{韓|한} ↔ A'],
      ['Shift+한/영|LCtrl+LAlt', '{釋讀|석독} → {國漢文|국한문} {混用|혼용} → {韓㐎|한글} {專用|전용}'],
      ['한자|Right Ctrl|変換|F7', '{變換|변환}'],
      ['無変換', 'A로'],
      ['{設置|설치}{版|판} macOS'],
      ['Option', '{韓|한} ↔ A'],
      ['LCtrl+LOption', '{釋讀|석독} → {國漢文|국한문} {混用|혼용} → {韓㐎|한글} {專用|전용}'],
      ['Option+Return|F7', '{變換|변환}'],
      ['英数|かな', 'A로 · {韓|한}으로'],
      ['{設置|설치}{版|판} Linux'],
      ['Hangul|Shift+Space|Right Alt', '{韓|한} ↔ A'],
      ['Shift+Hangul|LCtrl+LAlt', '{釋讀|석독} → {國漢文|국한문} {混用|혼용} → {韓㐎|한글} {專用|전용}'],
      ['Hanja|F9|変換|F7', '{變換|변환}'],
      ['{設置|설치}{版|판} Android'],
      ['한/영|Shift+Space', '{韓|한} ↔ A'],
      ['Shift+한/영|LCtrl+LAlt', '{釋讀|석독} → {國漢文|국한문} {混用|혼용} → {韓㐎|한글} {專用|전용}'],
      ['한자|F7', '{變換|변환}'],
    ],
    kind: { sino: '{音讀|음독}', kun: '{訓讀|훈독}' },
    copy: '{複寫|복사}',
    copied: '{複寫|복사}했습니다',
    clear: '{消|지우}기',
    ruby: '{讀|읽}기 {添|달}기',
    read: '{韓㐎|한글}로 {讀|읽}기',
    preview: '미리{見|보}기',
    keyboard: '{畵面|화면} {字板|자판}',
    settings: '{設定|설정}',
    keysHelp: '키 {案內|안내}',
    convert: '{變換|변환}',
    commit: '{確定|확정}',
    segments: '{變換|변환} {區間|구간}',
    segPrev: '{前|앞} {區間|구간}',
    segNext: '{次|다음} {區間|구간}',
    segList: '{候補|후보}',
  },
  options: {
    title: '{表記|표기} {設定|설정}',
    input: '{入力|입력}',
    spelling: '{表記|표기}',
    layout: '{候補|후보} {窓|창}',
    layoutTip: '{候補|후보} {窓|창}의 {方向|방향}. {橫|가로}이면 ←→로 {候補|후보}를 {移|옮기}ㅂ니다.',
    vertical: '{縱|세로}',
    horizontal: '{橫|가로}',
    spaceSwitch: 'Space {變換|변환}',
    spaceTip: '{押|누르}면 Space로 {變換|변환}합니다. {滅|끄}면 Space는 띄어{書|쓰}기, Tab·↓로 {變換|변환}.',
    oldSwitch: '{古|예}ㅅ{韓㐎|한글} 겹{字母|자모}',
    oldTip: 'ㅅ ㄱ → ᄭ, ㅂ ㅇ → ㅸ처럼 {現代|현대} {字母|자모}를 {連|이}어 {打|치}면 {古|예}ㅅ 겹{字母|자모}가 {成|되}ㅂ니다. ㆍ ㅿ ㆁ ㆆ는 {此|이} {設定|설정}과 {關係|관계}{無|없}이 Shift로 {打|치}ㅂ니다.',
    learnSwitch: '{選|고르}ㄴ {候補|후보} {學習|학습}',
    learnTip: '{同|같}은 {讀音|읽기}에 {同|같}은 {候補|후보}를 {二|두} {番|번} {選|고르}면 {其|그} {候補|후보}가 {先|먼저} {出|나오}ㅂ니다. {選|고르}ㄴ {個言|낱말}은 {此|이} 브라우저에만 {貯藏|저장}됩니다.',
    learnForget: '{選|고르}ㄴ {事|거}ㅅ {忘|잊}기',
    learnForgotten: '{消|지우}었습니다',
    script: '{文字|문자}',
    scriptTip: '{釋讀|석독}은 {音讀|음독}과 {訓讀|훈독} {皆|모두} {漢字|한자}로 {變換|변환}합니다. {國漢文|국한문} {混用|혼용}은 {音讀|음독}만 {漢字|한자}로 {變換|변환}합니다. {韓㐎|한글} {專用|전용}은 {變換|변환} {無|없}이 {韓㐎|한글}만 {打|치}고, Space는 띄어{書|쓰}기, Enter는 {確定|확정}입니다.',
    seokdokSide: '{釋讀|석독}',
    sinoSwitch: '{國漢文|국한문} {混用|혼용}',
    hangulSwitch: '{韓㐎|한글} {專用|전용}',
    sinoTip: '{音讀|음독} {個言|낱말}만 {漢字|한자}로 {書|쓰}고 {固有語|고유어}는 {韓㐎|한글}로 {置|두}ㅂ니다. 先生님께서 工夫하셨다',
    style: '{末音添記|말음첨기}',
    surface: '{表音式|표음식} 見ㅆ다',
    morpheme: '{形態式|형태식} 見았다',
    fragment: '{殘|남}는 {母音|모음}',
    absorb: '{漢字|한자}에 {吸收|흡수} 見라',
    letter: '{字母|자모}로 見ㅏ라',
    coda: '{末音添記|말음첨기} 늘 {書|쓰}기 (有ㅆ다)',
    resyllabify: '받침 {移|넘기}기 (在쓰면)',
    hada: '하다를 爲로 (安寧爲세요)',
    rubyAbove: '{讀|읽}는 {音|소리}: {上|위}',
    rubyRight: '{讀|읽}는 {音|소리}: {右|오른}쪽',
    itda: '있다의 {表記|표기}',
    itdaTip: '{選|고르}ㄴ {表記|표기}를 {先|먼저} {置|두}ㅂ니다. {異|다르}ㄴ {表記|표기}도 {候補|후보}에 {殘|남}습니다.',
    oldHa: '하 → ᄒᆞ {優先|우선}',
    oldHaTip: '하·한·할·함·합·해의 {古|예}ㅅ{韓㐎|한글} {表記|표기}(ᄒᆞ ᄒᆞᆫ ᄒᆞᆯ ᄒᆞᆷ ᄒᆞᆸ ᄒᆡ)를 {現代|현대} {表記|표기}보다 {先|먼저} {置|두}ㅂ니다. 하고 → ᄒᆞ고, 追加할까요 → 追加ᄒᆞᆯ까요. {滅|끄}면 {現代|현대} {表記|표기} {後|뒤}에 {置|두}ㅂ니다.',
  },
  convert: {
    eyebrow: '{變換|변환}',
    title: '{韓㐎|한글}을 {釋讀文|석독문}으로',
    lede: '{韓㐎|한글}이나 {釋讀文|석독문}을 {貼|붙여} {入|넣}으면 {個言|낱말}마다 {候補|후보}가 {出|나}옵니다. {個言|낱말}에 마우스를 {置|올리}거나 {押|누르}면 {異|다르}ㄴ {候補|후보}를 {選|고르}ㄹ {手|수} {有|있}습니다.',
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
    ruby: '{讀|읽}는 {音|소리}를 {添|달}아',
    plain: '{韓㐎|한글}로만',
  },
  rules: {
    eyebrow: '{表記|표기}',
    title: '세 가지 {材料|재료}',
    lede: '{文法|문법}은 {韓國語|한국어} {其|그}대로 {置|두}고, {單語|단어}를 {漢字|한자}로 {解|풀}어 {記|적}습니다. {定|정}해진 {規則|규칙}은 {無|없}고 {意|뜻}이 {通|통}하면 {成|되}ㅂ니다(作文 規則 無，意味傳達 第一). {入力器|입력기}는 {會|모임}에서 {實際|실제}로 {用|쓰}인 {表記|표기}를 {從|따르}고, {岐|갈리}는 데는 {設定|설정}으로 {殘|남}겨 {置|두}ㅂ니다.',
    sino: ['{音讀|음독}', '{漢字語|한자어}는 {音|소리}로', '규칙 → 規則, 환영 → 歡迎'],
    kun: ['{訓讀|훈독}', '{固有語|고유어}는 {意|뜻}으로', '하늘 → 天, 읽다 → 讀다'],
    okuri: ['{末音添記|말음첨기}', '{殘|나머지}는 {韓㐎|한글}로', '읽었다 → 讀었다, 다른 → 異ㄴ'],
    okuriNote: '{漢字|한자} {後|뒤}의 {韓㐎|한글}은 {二|두} {種|가지}입니다. {鄕札|향찰}의 夜音(밤), 道尸(길)처럼 {訓|훈}으로 {讀|읽}은 {字|글자} {後|뒤}에 {末音|끝소리}을 {添|덧붙}여 {書|적}는 {方式|방식}이 {末音添記|말음첨기}입니다. 超무, 少ㅁ이 {此|이} {例|예}입니다. 天을, 讀었다의 을·었다 {同|같}은 {文法|문법} {要素|요소}는 {吐|토}입니다. {日本語|일본어}의 送り仮名에 {該當|해당}합니다.',
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
    lede: '{對話|대화} {記錄|기록}에서 {頻|자주} {出|나}온 {表記|표기}입니다. {押|누르}면 {入力器|입력기}에 {入|넣}어 {見|보}ㅂ니다.',
    filter: '{韓㐎|한글}이나 {漢字|한자}로 {探|찾}기',
    count: '{回|회}',
  },
  custom: {
    eyebrow: '{自作|자작}',
    title: '{自己|자기}만의 {規則|규칙}',
    lede: 'RIME처럼 {規則|규칙}과 {個言|낱말}을 {直接|직접} {定|정}합니다. {此|이} 브라우저에 {貯藏|저장}되고, JSON으로 {共有|공유}할 {手|수} {有|있}습니다.',
    tabs: { rules: '{規則|규칙}', phrases: '{自作|자작} {個言|낱말}', words: '{訓讀|훈독} {個言|낱말}', share: '{共有|공유}' },
    rulesHelp: '{上|위}에서부터 {次例|차례}로 {適用|적용}합니다. {入力|입력} {規則|규칙}은 {變換|변환} {前|전}의 {韓㐎|한글}을, {出力|출력} {規則|규칙}은 {候補|후보}마다 {適用|적용}됩니다. 「{加|더하}기」는 {原來|원래} {候補|후보}를 {殘|남}기고 {新|새} {候補|후보}를 {後|뒤}에 {附|붙}입니다. regex에서는 $1 같은 {參照|참조}를 {用|쓰}ㄹ {手|수} {有|있}습니다.',
    presets: '{例|예}:',
    on: '{使用|사용}',
    stage: '{段階|단계}',
    stageInput: '{入力|입력} ({變換|변환} {前|전})',
    stageOutput: '{出力|출력} ({候補|후보})',
    mode: '{方式|방식}',
    xform: '{變|바꾸}기',
    derive: '{加|더하}기',
    pattern: '{探|찾}을 {字|글자}',
    replace: '{變|바꾸}ㄹ {字|글자}',
    remove: '{削除|삭제}',
    noRules: '{規則|규칙}이 {無|없}습니다. {例|예}를 {押|눌러} {始作|시작}해 {見|보}세요.',
    addRule: '{規則|규칙} {追加|추가}',
    test: '{試驗|시험}',
    phrasesHelp: '{讀|읽}는 {音|소리}에 {定|정}해진 {字|글자}를 {附|붙}입니다(RIME의 custom_phrase). {候補|후보}의 첫째로 {出|나}옵니다.',
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
    shareHelp: '{規則|규칙}과 {個言|낱말}을 JSON {文書|문서}로 {出|내}보내거나 {入|들}여옵니다. {入|들}여오면 {只今|지금}의 {設定|설정}을 {代身|대신}합니다.',
    download: 'JSON {下|내}려{受|받}기',
    copyJson: 'JSON {複寫|복사}',
    openFile: '{文書|문서} {開|열}기',
    import: '{入|들}여오기',
  },
  samples: {
    eyebrow: '{例文|예문}',
    title: '{名文|명문}을 {釋讀|석독}으로',
    lede: '{訓民正音|훈민정음}에서 {詩|시}와 {百科|백과}까지, {原文|원문}을 {其|그}대로 {入|넣}어 {變換|변환}했습니다. {入力器|입력기}에서 {開|열}면 {節|마디}마다 {異|다르}ㄴ {候補|후보}를 {見|보}ㄹ {手|수} {有|있}습니다.',
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
    friends: '{隣|이웃} {會|모임}',
    chat: '{國漢文混用|국한문혼용}으로 {雜談|잡담}하는 텔레그램 {房|방}',
    wiki: '{誰|누구}나 {書|쓰}고 {修正|수정}하는 {國漢文混用|국한문혼용} 위키',
  },
  old: {
    eyebrow: '{古|예}ㅅ{韓㐎|한글}',
    title: 'ㆍ와 {古|예}ㅅ {字母|자모} {打|치}기',
    lede: 'Microsoft {古|예}ㅅ{韓㐎|한글} {字板|자판}처럼 Shift {段|단}에 {古|예}ㅅ {字母|자모}가 {有|있}습니다. {此|이} {頁|쪽}의 {入力器|입력기}와 Windows·Linux {入力器|입력기}가 {皆|모두} {同|같}습니다.',
    mapCaption: '{各|각} {鍵|글쇠} {右|오른}쪽 {上|위}의 {小|작}은 {字|글자}는 Shift를 {押|누르}고 {打|치}는 {字母|자모}이고, {古|예}ㅅ {字母|자모}는 더 {大|크}게 {表示|표시}했습니다. {入力器|입력기}의 {畵面|화면} {字板|자판}에서는 ⇧를 {押|누르}ㄴ {後|뒤} {鍵|글쇠}를 {押|누르}ㅂ니다.',
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
    clustersNote: '{現代|현대} {字母|자모}를 {連|이}어 {打|쳐} {古|예}ㅅ 겹{字母|자모}를 {作|만드}는 {事|거}ㅅ은 「{古|예}ㅅ{韓㐎|한글} 겹{字母|자모}」를 {點|켜}야 {成|되}ㅂ니다. {入力器|입력기} {窓|창}의 {道具|도구} 줄, Windows {設定|설정} {窓|창}, {或|또}는 config.toml의 old_clusters = true.',
    conversion: '{古|예}ㅅ{韓㐎|한글} {音節|음절}은 {打|치}ㄴ {其|그}대로 {殘|남}고, {其|그} {前後|앞뒤}의 {現代|현대} {韓㐎|한글}만 {漢字|한자}로 {變換|변환}됩니다.',
    fonts: '{此|이} {頁|쪽}은 {古|예}ㅅ{韓㐎|한글} 글꼴을 {自|스스로} {呼|불러}옵니다. {異|다르}ㄴ 앱에서는 Noto Serif CJK KR, Noto Sans CJK KR, 함초롬바탕처럼 {古|예}ㅅ{韓㐎|한글}을 {支援|지원}하는 글꼴이 {必要|필요}합니다.',
  },
  install: {
    eyebrow: '{設置|설치}',
    title: '{自|저}ㅣ 컴퓨터에서 {書|쓰}기',
    lede: '{此|이} {頁|쪽}과 {同|같}은 엔진과 {辭典|사전}을 {用|쓰}는 {入力器|입력기}입니다. Windows, Linux(fcitx5), Android를 {支援|지원}합니다.',
    download: '{下|내}려{受|받}기',
    windows: [
      '{下|내}려{受|받}은 {設置|설치} 파일을 {實行|실행}합니다. {管理者|관리자} {權限|권한}을 {求|구}합니다.',
      'Win+Space로 {釋讀|석독}을 {選|고르}ㅂ니다. {韓國語|한국어}가 {言語|언어} {目錄|목록}에 {無|없}으면 {自動|자동}으로 {加|더}합니다.',
      '한/영이나 {右|오른}쪽 Alt로 {韓|한}과 A를 {轉換|전환}하고, 한자나 {右|오른}쪽 Ctrl로 {變換|변환}합니다.',
    ],
    linux: [
      '{壓縮|압축}을 {解|풀}고 install.sh를 {實行|실행}합니다. sudo {無|없}이 {此|이} {使用者|사용자}에게만 {設置|설치}합니다.',
      'fcitx5를 {再|다시} {始作|시작}하고, fcitx5-configtool에서 「Only Show Current Language」를 {解除|해제}한 {後|뒤} {釋讀|석독}을 {加|더}합니다.',
      'Hangul 키나 Shift+Space로 {韓|한}과 A를 {轉換|전환}하고, Hanja 키나 F9로 {變換|변환}합니다.',
    ],
    ios: [
      'Windows에 iTunes와 iCloud, Sideloadly를 {設置|설치}합니다. iTunes와 iCloud는 sideloadly.io에 {案內|안내}된 Apple {版|판}을 {用|쓰}고, Microsoft Store {版|판}은 {削除|삭제}합니다.',
      'iPhone을 USB로 {連結|연결}하고 {信賴|신뢰}를 {押|누르}ㅂ니다. {設定|설정} › {個人情報|개인정보} {保護|보호} 및 {保安|보안} › {開發者|개발자} 모드를 켭니다.',
      '{下|내}려{受|받}은 IPA를 Sideloadly에 {入|넣}고 Apple ID를 {入力|입력}하여 Start를 {押|누르}ㅂ니다. {無料|무료} Apple ID는 7{日|일}마다 {再|다시} {設置|설치}해야 합니다.',
      '{設定|설정} › 일반 › VPN 및 기기 관리에서 Apple ID를 {信賴|신뢰}합니다.',
      '{設定|설정} › 일반 › 키보드 › 키보드 › 새로운 키보드 추가에서 {釋讀|석독}을 {加|더}합니다. 지구본 키로 {釋讀|석독}을 {選|고르}ㅂ니다.',
    ],
    macos: [
      '{下|내}려{受|받}은 ZIP을 터미널에서 {下|아래} {命令|명령}으로 {設置|설치}합니다. {開發者|개발자} {署名|서명}이 {無|없}어 xattr이 {必要|필요}합니다. {實際|실제} Mac에서는 {尙|아직} {試驗|시험}하지 {不|않}았습니다.',
      '시스템 설정 › 키보드 › 입력 소스 › 편집…에서 +를 {押|눌러} 한국어의 {釋讀|석독}을 {加|더}합니다.',
      '메뉴 막대의 입력 메뉴에서 {釋讀|석독}을 {選|고르}ㅂ니다.',
    ],
    android: [
      '{下|내}려{受|받}은 APK를 엽니다. 「출처를 알 수 없는 앱」 {設置|설치}를 {許容|허용}하라는 {案內|안내}가 나오면, APK를 연 브라우저나 파일 앱에 {許容|허용}합니다.',
      '{設置|설치}한 뒤 {釋讀|석독} 앱을 엽니다. 「키보드 設定 열기」에서 {釋讀|석독}을 켭니다.',
      '글을 쓰는 곳에서 「入力 方法 고르기」나 키보드의 ⇆ 키로 {釋讀|석독}을 {選|고르}ㅂ니다. Space로 {變換|변환}하고, {候補|후보}를 {押|눌러} {確定|확정}합니다.',
    ],
    system: '{全|모든} {使用者|사용자}에게:',
    remove: '{削除|삭제}:',
    keys: 'Space {變換|변환} · Space {再|다시} {候補|후보} · 1–9 {選擇|선택} · Enter {確定|확정} · Esc {取消|취소} · 한/영 {或|또}는 Alt {短|짧}게: {釋讀|석독} ↔ A',
    custom: '{自作|자작} {規則|규칙}과 {個言|낱말}은 {上|위} 「{自作|자작}」의 {共有|공유}에서 JSON으로 {下|내}려{受|받}아 custom.json으로 {貯藏|저장}합니다.',
    where: '{置|두}는 {處|곳}',
    note: '{同|같}은 {處|곳}의 config.toml에서 {設定|설정}을 {變|바꾸}ㅂ니다. {入力器|입력기}를 {變|바꾸}었다가 {返|돌아}오면 {再|다시} {讀|읽}습니다.',
  },
  eumdok: {
    eyebrow: '{派生|파생} {入力器|입력기}',
    title: '{音讀|음독}: {國漢文混用|국한문혼용} {專用|전용}',
    lede: '{漢字語|한자어}는 {漢字|한자}로, {固有語|고유어}와 {其他|기타} {全部|전부}는 {韓㐎|한글}로 {書|쓰}ㅂ니다. {訓讀|훈독}은 {候補|후보}에도 {出|나오}지 {不|안}ㅎ습니다. {釋讀|석독}과 {同|같}은 엔진과 {辭典|사전}을 {用|쓰}는 {別途|별도}의 {入力器|입력기}입니다.',
    typed: '{打|치}ㄴ {文|글}',
    seokdok: '{釋讀|석독}',
    eumdok: '{音讀|음독}',
    switchTitle: '{釋讀|석독}의 {文字|문자} {轉換|전환}과 {何|무엇}이 {異|다르}ㄴ가',
    switchBody: '{釋讀|석독} {入力器|입력기}의 {文字|문자} {轉換|전환}은 {釋|석} → {混|혼} → 한을 {轉換|전환}합니다. {混|혼}에서 {變換|변환} {結果|결과}는 {音讀|음독} {入力器|입력기}와 {同|같}습니다. {音讀|음독}은 {此|이} {混|혼}만 {必要|필요}한 {人|분}을 {爲|위}한 {入力器|입력기}로, {設定|설정}에 {訓讀|훈독} {項目|항목}이 {無|없}고 {釋讀|석독}으로 {轉換|전환}되지 {不|안}ㅎ습니다.',
    rows: [
      ['{文字|문자} {轉換|전환} 키', '{釋|석} → {混|혼} → 한', '{混|혼} ↔ 한'],
      ['{訓讀|훈독} {候補|후보}', '{有|있}음', '{無|없}음'],
      ['{訓讀|훈독} {表記|표기} {設定|설정}', '{有|있}음', '{無|없}음'],
      ['{設定|설정}과 {學習|학습} 폴더', 'seokdok', 'eumdok'],
    ],
    downloadTitle: '{下|내}려{受|받}기',
    side: '{釋讀|석독}과 {一|한} {機器|기기}에 {共|함께} {設置|설치}할 {手|수} {有|있}습니다. {設定|설정}과 {學習|학습}은 {各各|각각} {別途|따로} {貯藏|저장}됩니다.',
    soon: '{音讀|음독} {設置|설치} 파일은 {尙|아직} {配布|배포}되지 {不|안}ㅎ았습니다.',
    windows: '{實行|실행}한 {後|뒤} Win+Space로 {音讀|음독}을 {選|고르}ㅂ니다.',
    linux: 'install.sh를 {實行|실행}하고 fcitx5를 {再|다시} {始作|시작}한 {後|뒤}, fcitx5-configtool에서 {音讀|음독}을 {加|더}합니다.',
    android: '{設置|설치}한 {後|뒤} {音讀|음독} 앱을 {開|열}고 키보드를 {點|켜}ㅂ니다.',
    apple: 'iOS와 macOS는 소스에서 빌드합니다. 무료 Apple ID로 사이드로딩하면 앱을 {同時|동시}에 {三|세} {個|개}까지 {用|쓰}ㄹ {手|수} {有|있}으므로, {音讀|음독}과 {釋讀|석독}을 {共|함께} {置|두}기 {難|어려}울 {手|수} {有|있}습니다.',
  },
  footer: {
    sources: '{辭典|사전}과 {出處|출처}',
    suggested: '{機械|기계} {提案|제안}: {言語|언어} 모델이 {意|뜻}으로 {附|붙}인 {漢字|한자}, {人|사람}이 {檢討|검토}하지 {不|안}ㅎ음',
    made: '{作|만드}ㄴ {人|이}',
    licence: '코드는 MIT. {辭典|사전}마다 {許可|허가}가 {異|다르}ㅂ니다.',
    privacy: '{個人情報|개인정보} {處理|처리}{方針|방침}',
    support: '{顧客|고객} {支援|지원}',
  },
}

const en = {
  nav: { ime: 'Try', custom: 'Customize', rules: 'Spelling', words: 'Words', convert: 'Convert', old: 'Old Hangul', install: 'Install', eumdok: '音讀', join: 'Group' },
  theme: { light: 'Light', dark: 'Dark', auto: 'Auto' },
  hero: {
    kicker: '釋讀會 御製',
    title1: 'Type Hangul,',
    title2: 'write it out in Hanja',
    lede: 'Sino-Korean words by sound, native words by meaning. Type Korean as usual, press Space, and get 音讀, 訓讀 and 末音添記 at once. No input method to install: try it on this page.',
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
      ['살찔비|비살찔', 'find a character by its gloss (肥)'],
      ['Installed on Windows'],
      ['Han/Eng|Right Alt|半角/全角', 'Korean ↔ Latin'],
      ['Shift+Han/Eng|LCtrl+LAlt', '釋讀 → 國漢文混用 → 韓㐎 專用'],
      ['Hanja|Right Ctrl|変換|F7', 'convert'],
      ['無変換', 'to Latin'],
      ['Installed on macOS'],
      ['Option', 'Korean ↔ Latin'],
      ['LCtrl+LOption', '釋讀 → 國漢文混用 → 韓㐎 專用'],
      ['Option+Return|F7', 'convert'],
      ['英数|かな', 'to Latin · to Korean'],
      ['Installed on Linux'],
      ['Hangul|Shift+Space|Right Alt', 'Korean ↔ Latin'],
      ['Shift+Hangul|LCtrl+LAlt', '釋讀 → 國漢文混用 → 韓㐎 專用'],
      ['Hanja|F9|変換|F7', 'convert'],
      ['Installed on Android'],
      ['Han/Eng|Shift+Space', 'Korean ↔ Latin'],
      ['Shift+Han/Eng|LCtrl+LAlt', '釋讀 → 國漢文混用 → 韓㐎 專用'],
      ['Hanja|F7', 'convert'],
    ],
    kind: { sino: 'by sound', kun: 'by meaning' },
    copy: 'Copy',
    copied: 'Copied',
    clear: 'Clear',
    ruby: 'Readings',
    read: 'Read as Hangul',
    preview: 'Preview',
    keyboard: 'On-screen keys',
    settings: 'Settings',
    keysHelp: 'Keys',
    convert: 'Convert',
    commit: 'Commit',
    segments: 'Segments',
    segPrev: 'Previous segment',
    segNext: 'Next segment',
    segList: 'Choices',
  },
  options: {
    title: 'Spelling options',
    input: 'Input',
    spelling: 'Spelling',
    layout: 'Candidate list',
    layoutTip: 'Direction of the candidate list. When horizontal, ← → move through candidates.',
    vertical: 'Vertical',
    horizontal: 'Horizontal',
    spaceSwitch: 'Space converts',
    spaceTip: 'On: Space converts. Off: Space types a space and Tab or ↓ converts.',
    oldSwitch: 'Old clusters',
    oldTip: 'Modern letters typed in a row join into old clusters: ㅅ ㄱ → ᄭ, ㅂ ㅇ → ㅸ. ㆍ ㅿ ㆁ ㆆ are on Shift either way.',
    learnSwitch: 'Learn picks',
    learnTip: 'Pick the same candidate twice for a reading and it comes first. The words you picked stay in this browser.',
    learnForget: 'Forget what I picked',
    learnForgotten: 'Forgotten',
    script: 'Script',
    scriptTip: 'Seokdok converts Sino-Korean and native words to Hanja. 國漢文混用 converts Sino-Korean words only. Hangul only never converts: Space types a space and Enter commits.',
    seokdokSide: '釋讀',
    sinoSwitch: '國漢文混用',
    hangulSwitch: 'Hangul only',
    sinoTip: 'Hanja for Sino-Korean words only; native words stay in Hangul: 先生님께서 工夫하셨다.',
    style: '末音添記',
    surface: 'Surface 見ㅆ다',
    morpheme: 'Morpheme 見았다',
    fragment: 'Leftover vowel',
    absorb: 'In the Hanja 見라',
    letter: 'As a letter 見ㅏ라',
    coda: 'Always write the final consonant (有ㅆ다)',
    resyllabify: 'Carry it to the next syllable (在쓰면)',
    hada: '爲 for 하다 (安寧爲세요)',
    rubyAbove: 'Readings: above',
    rubyRight: 'Readings: to the right',
    itda: 'Spelling of 있다',
    itdaTip: 'Lists the chosen spelling first. The others stay among the candidates.',
    oldHa: '하 → ᄒᆞ first',
    oldHaTip: 'Puts the old spelling of 하 한 할 함 합 해 (ᄒᆞ ᄒᆞᆫ ᄒᆞᆯ ᄒᆞᆷ ᄒᆞᆸ ᄒᆡ) before the modern one: 하고 → ᄒᆞ고, 追加할까요 → 追加ᄒᆞᆯ까요. Off, the old spelling is offered after the modern one.',
  },
  convert: {
    eyebrow: 'Convert',
    title: 'Hangul text to 釋讀',
    lede: 'Paste Hangul or 釋讀 text and every word gets its candidates. Hover over a word, or tap it, to pick another candidate.',
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
    ruby: 'With readings',
    plain: 'Hangul only',
  },
  rules: {
    eyebrow: 'Spelling',
    title: 'Three kinds of material',
    lede: 'The grammar stays Korean; the words are written out in Hanja. The group has no fixed rules, only that the meaning gets across (作文 規則 無，意味傳達 第一). The input method follows how members actually write, and keeps their differences as options.',
    sino: ['音讀', 'Sino-Korean words by sound', '규칙 → 規則, 환영 → 歡迎'],
    kun: ['訓讀', 'Native words by meaning', '하늘 → 天, 읽다 → 讀다'],
    okuri: ['末音添記', 'The rest in Hangul', '읽었다 → 讀었다, 다른 → 異ㄴ'],
    okuriNote: 'The Hangul after a Hanja comes in two kinds. 末音添記 (maleum cheomgi) adds the last sound of a word read by meaning, as in hyangchal 夜音 for 밤 and 道尸 for 길; 超무 and 少ㅁ work the same way. Grammatical endings such as 을 and 었다 in 天을 and 讀었다 are 吐. Together they correspond to Japanese okurigana.',
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
    friends: 'Neighbours',
    chat: 'A Telegram chat written in mixed Hanja and Hangul',
    wiki: 'A wiki in mixed Hanja and Hangul that anyone can edit',
  },
  old: {
    eyebrow: 'Old Hangul',
    title: 'Typing ㆍ and the old letters',
    lede: 'As on Microsoft Old Hangul, the old letters sit on the Shift layer. The same keys work on this page and in the Windows and Linux input methods.',
    mapCaption: 'The small letter in each key’s corner is typed with Shift; old letters are drawn larger there. On the on-screen keyboard above, tap ⇧ and then the key.',
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
    lede: 'A system input method with the same engine and dictionaries as this page, for Windows, Linux (fcitx5) and Android.',
    download: 'Download',
    windows: [
      'Run the downloaded setup. It asks for administrator rights.',
      'Pick 釋讀 with Win+Space. Korean is added to your language list if it is missing.',
      'Han/Eng or right Alt switches between Korean and Latin. Hanja or right Ctrl converts.',
    ],
    linux: [
      'Unpack and run install.sh. It installs for your user only, without sudo.',
      'Restart fcitx5, then add 釋讀 in fcitx5-configtool (untick “Only Show Current Language” to find it).',
      'The Hangul key or Shift+Space switches between Korean and Latin. The Hanja key or F9 converts.',
    ],
    ios: [
      'On Windows, install iTunes and iCloud (the Apple versions linked at sideloadly.io, not the Microsoft Store ones) and Sideloadly.',
      'Connect the iPhone by USB and tap Trust. Turn on Settings › Privacy & Security › Developer Mode.',
      'Drag the downloaded IPA onto Sideloadly, enter your Apple ID and press Start. With a free Apple ID, install it again every 7 days.',
      'In Settings › General › VPN & Device Management, trust your Apple ID.',
      'Add 釋讀 under Settings › General › Keyboard › Keyboards › Add New Keyboard, and pick it with the globe key.',
    ],
    macos: [
      'Install the downloaded ZIP with the commands below in Terminal. It has no developer signature, so xattr is needed. It has not been tried on a real Mac yet.',
      'In System Settings › Keyboard › Input Sources › Edit…, press + and add 釋讀 under Korean.',
      'Pick 釋讀 in the input menu of the menu bar.',
    ],
    android: [
      'Open the downloaded APK. If Android asks to allow installing apps from unknown sources, allow it for the browser or file manager you opened the APK from.',
      'After installing, open the 釋讀 app and use “Keyboard settings” to turn 釋讀 on.',
      'Where you type, pick 釋讀 with “Choose input method” or the ⇆ key on the keyboard. Space converts; tap a candidate to confirm.',
    ],
    system: 'For all users:',
    remove: 'Uninstall:',
    keys: 'Space converts · Space again lists candidates · 1–9 pick · Enter confirms · Esc cancels · Han/Eng or a tap of Alt: 釋讀 ↔ A',
    custom: 'For your own rules and words, download the JSON from Share under Customize above and save it as custom.json.',
    where: 'Location',
    note: 'Settings live in config.toml in the same folder. Switch input method away and back to reload.',
  },
  eumdok: {
    eyebrow: 'Sister input method',
    title: '音讀 (Eumdok): 國漢文混用 only',
    lede: 'Sino-Korean words in Hanja, native words and everything else in Hangul. 訓讀 never appears, not even among the candidates. It is a separate input method on the same engine and dictionaries as 釋讀.',
    typed: 'Typed',
    seokdok: '釋讀',
    eumdok: '音讀',
    switchTitle: 'How it differs from the switch in 釋讀',
    switchBody: 'The script switch in the 釋讀 input method goes 釋 → 混 → 한. In 混 it converts exactly as 音讀 does. 音讀 is for people who only want 混: its settings have no 訓讀 options, and it cannot switch to 釋讀.',
    rows: [
      ['Script key', '釋 → 混 → 한', '混 ↔ 한'],
      ['訓讀 candidates', 'Yes', 'None'],
      ['訓讀 spelling settings', 'Yes', 'None'],
      ['Settings and learning folder', 'seokdok', 'eumdok'],
    ],
    downloadTitle: 'Download',
    side: 'It installs beside 釋讀 on one machine. Each keeps its own settings and learned picks.',
    soon: 'The 音讀 installers have not been released yet.',
    windows: 'Run the setup, then pick 音讀 with Win+Space.',
    linux: 'Run install.sh, restart fcitx5, then add 音讀 in fcitx5-configtool.',
    android: 'Install it, open the 音讀 app and turn the keyboard on.',
    apple: 'iOS and macOS build from source. Sideloading with a free Apple ID allows three apps at a time, so keeping both 音讀 and 釋讀 may not fit.',
  },
  footer: {
    suggested: 'Machine suggestions: Hanja assigned by meaning by a language model, not reviewed by a person',
    sources: 'Dictionaries and sources',
    made: 'Made by',
    licence: 'Code under MIT; each dictionary keeps its own licence.',
    privacy: 'Privacy policy',
    support: 'Support',
  },
}

// 'ko': Korean in 釋讀 with Hangul ruby; 'kore': the same, Hanja only; 'en'.
const all = { ko: { ...ko, ...pages.ko }, kore: { ...ko, ...pages.ko }, en: { ...en, ...pages.en } }


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

const MARK = /\{([^|}]+)\|([^}]+)\}([ㄱ-ㅎㅣ])?/g
const esc = (s) => s.replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c])

/** A mark read by its characters' own Sino-Korean readings (音讀). */
export function isSino(hanja, reading) {
  // 手 for the bound noun 수 goes by meaning, though 手 also reads 수 by sound.
  if (hanja === '手' && reading === '수') return false
  const h = [...hanja]
  const k = [...reading]
  return h.length === k.length && h.every((c, i) => SINO[c]?.includes(k[i]))
}

// Final consonants as they follow a syllable without one (되 + ㅂ → 됩).
const FINALS = ' ㄱㄲㄳㄴㄵㄶㄷㄹㄺㄻㄼㄽㄾㄿㅀㅁㅂㅄㅅㅆㅇㅈㅊㅋㅌㅍㅎ'
// A final plus a consonant that makes a cluster final (안 + ㅎ → 않).
const CLUSTERS = { ㄱㅅ: 'ㄳ', ㄴㅈ: 'ㄵ', ㄴㅎ: 'ㄶ', ㄹㄱ: 'ㄺ', ㄹㅁ: 'ㄻ', ㄹㅂ: 'ㄼ', ㄹㅅ: 'ㄽ', ㄹㅌ: 'ㄾ', ㄹㅍ: 'ㄿ', ㄹㅎ: 'ㅀ', ㅂㅅ: 'ㅄ' }
// A vowel plus ㅣ (저 + ㅣ → 제).
const WITH_I = { 0: 1, 2: 3, 4: 5, 6: 7, 8: 11, 13: 16, 18: 19 }
function attach(word, jamo) {
  if (!jamo) return word
  const last = word.codePointAt(word.length - 1) - 0xac00
  if (last < 0 || last > 11171) return word + jamo
  const final = last % 28
  const head = word.slice(0, -1)
  if (jamo === 'ㅣ') {
    const vowel = Math.floor(last / 28) % 21
    return final === 0 && vowel in WITH_I ? head + String.fromCodePoint(0xac00 + last + (WITH_I[vowel] - vowel) * 28) : word + jamo
  }
  if (final === 0) {
    const f = FINALS.indexOf(jamo)
    return f < 1 ? word + jamo : head + String.fromCodePoint(0xac00 + last + f)
  }
  const cluster = FINALS.indexOf(CLUSTERS[FINALS[final] + jamo])
  return cluster > 0 ? head + String.fromCodePoint(0xac00 + last - final + cluster) : word + jamo
}

/** The Hangul reading of a marked string: every mark replaced by its reading. */
export function reading(s) {
  return s.replace(MARK, (_, h, k, j = '') => attach(k, j))
}

/** In 國漢文混用 mode a 訓讀 mark shows its Hangul reading. */
const hangulOnly = (h, k) => options.sinoOnly && !isSino(h, k)

function render(s, ruby, html) {
  const text = html ? esc(s) : s
  const out = text.replace(MARK, (_, h, k, j = '') => {
    if (hangulOnly(h, k)) return attach(k, j)
    if (!html) return ruby ? attach(k, j) : h + j
    return (ruby ? `<ruby>${h}<rt>${k}</rt></ruby>` : h) + j
  })
  // One span, so a flex parent (button, label) sees one item and not a piece per ruby.
  return html ? `<span>${out}</span>` : out
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
