// The /dict page's text, in Korean (with {漢字|한자} marks) and English. Kept out of
// i18n.svelte.js so the other pages do not download it. %n is a number. `files`
// says in plain words what each dict/*.tsv is; dict/README.md and each file's header
// have the full account.

const ko = {
  title: '{辭典|사전} {探|찾}기',
  lede: '{入力器|입력기}가 {用|쓰}는 {辭典|사전}의 {全部|전부}입니다. {韓㐎|한글} {讀音|읽기}나 {漢字|한자}로 {探|찾}고, {出處|출처}{別|별}로 {分|나누}어 {見|보}세요.',
  search: '{韓㐎|한글} {讀音|읽기} {或|또}는 {漢字|한자}',
  placeholder: '{例|예}: 하늘, 天, 讀다',
  filter: '{出處|출처}',
  all: '{全體|전체}',
  row: '%n{行|행}',
  rows: '%n{行|행}',
  empty: '{此|이} {條件|조건}에 {合|맞}는 {行|행}이 {無|없}습니다.',
  loading: '{辭典|사전}을 {讀|읽}는 {中|중}',
  failed: '{辭典|사전}을 {讀|읽}지 못했습니다. GitHub에서 {見|보}ㄹ {手|수} {有|있}습니다:',
  columns: { reading: '{讀音|읽기}', spelling: '{表記|표기}', pos: '{品詞|품사}', count: '{回數|횟수}', source: '{出處|출처}', note: '{備考|비고}' },
  pager: { label: '{面|면} {移動|이동}', first: '{初|처}음', prev: '{以前|이전}', next: '{次|다음}', last: '{終|끝}', page: '{面|면}' },
  sourcesTitle: '{各|각} {出處|출처}',
  licence: '라이선스',
  show: '{此|이} {行|행}만 {見|보}기',
  notice: '라이선스 {全文|전문}과 {來歷|내력}:',
  permission: '{編輯者|편집자}의 {許可|허가}를 {受|받}아 {使用|사용}',
  scopes: { namu: '나무위키 {引用|인용}', chat: '{手|손}으로 {讀|읽}은 {對話|대화} {個言|낱말}' },
  pos: {
    noun: '{名詞|명사}',
    pron: '{代名詞|대명사}',
    num: '{數詞|수사}',
    bound: '{依存名詞|의존명사}',
    adv: '{副詞|부사}',
    det: '{冠形詞|관형사}',
    interj: '{感歎詞|감탄사}',
    verb: '{動詞|동사}',
    adj: '{形容詞|형용사}',
  },
  tags: {
    hangul: '{對話|대화}에서는 {此|이} {讀音|읽기}를 {韓㐎|한글}로 {書|쓰}고 {此|이} {漢字|한자}로는 {書|쓰}지 {不|안}ㅎ습니다.',
    corpus: '{對話|대화}에 5{番|번} {以上|이상} {出|나}온 {漢字|한자} {個言|낱말}입니다. {讀音|읽기}는 {字|글자}마다 {多|많}이 {用|쓰}는 {事|거}ㅅ을 {取|취}했기에 {錯|틀}릴 {手|수} {有|있}습니다.',
    nativised: '{漢字語|한자어}가 {固有語|고유어}처럼 {變|바뀌}ㄴ {個言|낱말}입니다.',
    coined: '{韓國|한국}에서 {漢字|한자} {形態素|형태소}로 {作|만드}ㄴ {個言|낱말}입니다.',
  },
  files: {
    corpus: {
      name: '{對話|대화} {表記|표기}',
      about: '{釋讀會|풀어읽기모임} {對話房|대화방}에서 {實際|실제}로 {用|쓰}는 {表記|표기}입니다. {讀音|읽기}는 {編輯者|편집자}가 {文脈|문맥}을 {見|보}고 {定|정}했고, {回數|횟수}는 {對話|대화}에 {出|나}온 {番|번} {數|수}입니다.',
    },
    editorial: {
      name: '{編輯者|편집자} {項目|항목}',
      about: '{編輯者|편집자}가 {直接|직접} {追加|추가}한 {訓讀|훈독} {個言|낱말}입니다. {備考|비고}에 {理由|이유}를 {記|적}었습니다.',
    },
    community: {
      name: '標準國語釋讀辭典',
      about: '{釋讀會|풀어읽기모임} {會員|회원}들이 2022{年|년} 6{月|월}부터 {編纂|편찬}한 {辭典|사전}입니다.',
    },
    eonhae: {
      name: '{古|예}ㅅ {冊|책}',
      about: '訓蒙字會(1527) {等|등} {字|글자} {敎本|교본}과 {諺解|언해}에 {記|적}힌 {訓讀|훈독}입니다. {備考|비고}에 {確認|확인}할 {處|곳}을 {記|적}었습니다. {本文|본문}은 {大部分|대부분} {校正|교정}되지 {不|안}ㅎ은 위키文獻의 {轉寫|전사}와, {誰|누구}나 {編輯|편집}할 {手|수} {有|있}는 나무위키입니다. {標本|표본}을 {手|손}으로 {確認|확인}하니 2~10%가 {錯|틀}렸습니다. {其|그} {行|행}은 {除去|제거}했으나 {他|다른} {行|행}에도 {誤謬|오류}가 {有|있}을 {手|수} {有|있}습니다.',
    },
    'kun-attested': {
      name: '{印刷|인쇄}된 {訓讀|훈독}',
      about: '1882~1945{年|년} {冊|책}에서 {漢字|한자} {傍|곁}에 {固有語|고유어} {讀音|읽기}를 {印刷|인쇄}한 {用例|용례}입니다(勞動夜學讀本, 1908 {等|등}). {備考|비고}에 {冊|책}, {年度|연도}, {頁|쪽}, {引用|인용}을 {記|적}었습니다. {本文|본문}은 위키文獻의 {轉寫|전사}이며 {大部分|대부분} {校正|교정}되지 {不|안}ㅎ았습니다.',
    },
    suggested: {
      name: '{機械|기계} {提案|제안}',
      about: '{手|손}으로 {作|만드}ㄴ {項目|항목}이 {無|없}는 {固有語|고유어}에 {言語|언어} {模型|모형}이 {提案|제안}한 {漢字|한자}이며, {人|사람}이 {檢討|검토}하지 {不|안}ㅎ았습니다. {對話|대화}에 {頻|자주} {出|나}오는 {一部|일부} {個言|낱말}만은 {手|손}으로 {讀|읽}어 {確認|확인}했고 {備考|비고}에 {根據|근거}를 {記|적}었습니다.',
    },
    'place-names': {
      name: '{地名|지명}',
      about: '{固有語|고유어} {地名|지명}과 {其|그} {漢字|한자}입니다. {出處|출처}는 {日本語|일본어} 위키百科의 朝鮮地名音訓一覧으로, {參考文獻|참고문헌}이 {一|하나}뿐인 {二次|이차} {資料|자료}입니다. {此|이} {表|표}에는 로마字 {表記|표기}만 {有|있}어 {韓㐎|한글}은 {其|그} 로마字를 {轉寫|전사}한 {事|거}ㅅ입니다.',
    },
    'wikt-native': {
      name: '{常用|상용} {固有語|고유어}',
      about: '{第一|첫} {候補|후보}가 {韓㐎|한글} {其|그}대로이던 {常用|상용} {固有語|고유어}입니다. {部分|부분}의 {漢字|한자}를 {連|이}어 {作|만드}ㄴ {事|거}ㅅ(눈앞 目前)이거나 {訓音|훈음}・JMdict {連結|연결}을 {從|따르}ㄴ {事|거}ㅅ이며, {合成語|합성어} {全體|전체}의 {表記|표기}를 {記|적}은 {出處|출처}는 {無|없}습니다. {部分|부분}과 {意味|의미}는 English Wiktionary에서 {來|와}ㅆ고, {候補|후보}는 {皆|모두} {手|손}으로 {讀|읽}어 {確認|확인}했습니다.',
    },
    gloss: {
      name: '{訓音|훈음} {推定|추정}',
      about: 'libhangul {漢字|한자} {目錄|목록}의 {訓音|훈음}에서 {機械|기계}로 {復元|복원}한 {候補|후보}입니다(하늘 천 → 하늘 = 天). {錯|틀}린 {個言|낱말}도 {有|있}습니다.',
    },
    'kun-ja': {
      name: '{日本語|일본어} {經由|경유}',
      about: '{韓國語|한국어} {固有語|고유어}와 JMdict의 {日本語|일본어} {個言|낱말}이 {同|같}은 {英語|영어} {解|풀이}를 {共有|공유}하면 {其|그} {漢字|한자}를 {候補|후보}로 {取|취}합니다(태어나다 ↔ 産まれる → 產다). {檢討|검토}되지 {不|안}ㅎ았고, {同音異義語|동음이의어} 때문에 {錯|틀}린 {連結|연결}도 {有|있}습니다.',
    },
    'kun-ja-weak': {
      name: '{日本語|일본어} {經由|경유}, {弱|약}한 {連結|연결}',
      about: '{同|같}은 {方法|방법}에서 {連結|연결}이 {弱|약}한 {事|거}ㅅ입니다. {候補|후보}를 {失|잃}지 {不|안}ㅎ으려고 {置|두}었습니다.',
    },
    'editorial-sino': {
      name: '{編輯者|편집자} {音讀|음독}',
      about: '{韓㐎|한글}로 {始作|시작}하고 {漢字|한자} {部分|부분}이 {有|있}는 {個言|낱말}(두벌式)과, {同音語|동음어} {中|중} {第一|첫} {候補|후보}를 {編輯者|편집자}가 {定|정}한 {事|거}ㅅ입니다.',
    },
    fusion: {
      name: 'FusionIME',
      about: 'Lee0701의 FusionIME(十八字)에 {入|들}어 {有|있}는 {音讀|음독} {個言|낱말}과 {頻度|빈도}입니다. {其|그} {資料|자료}는 표준국어대사전과 libhangul에서 {來|와}ㅆ습니다. {回數|횟수} {欄|칸}은 {頻度|빈도}의 {立方根|세제곱근}을 {半|반}올림한 {値|값}입니다.',
    },
    sino: {
      name: 'libhangul',
      about: 'libhangul {漢字|한자} {目錄|목록}의 {音讀|음독} {個言|낱말}과 {字|글자}에 {對話|대화}에 {出|나}온 {個言|낱말}을 {加|더}한 {事|거}ㅅ입니다. {回數|횟수}는 텔레그램 {對話房|대화방} {二|두} {處|곳}({釋讀會|풀어읽기모임}, 漢字混用)에 {出|나}온 {番|번} {數|수}입니다.',
    },
    etym: {
      name: 'Wiktionary {漢字|한자}',
      about: '{他|다른} 파일에 {無|없}는 {漢字|한자} {表記|표기}를 English Wiktionary(kaikki.org)에서 {取|가져}온 {事|거}ㅅ입니다. {語源|어원}으로만 {漢字語|한자어}인 {個言|낱말}도 {入|들}어 {有|있}습니다(재미 滋味, 김치 沉菜). Wiktionary의 {語源|어원}은 {共同|공동} {編輯|편집}이라 {異論|이론}이 {有|있}는 {事|거}ㅅ도 {有|있}습니다.',
    },
    loanwords: {
      name: '{外來語|외래어}',
      about: '유럽 {言語|언어}, 튀르키예語, 아랍語, 힌디語, 페르시아語에서 {借用|차용}해 {韓㐎|한글}로 {置|두}는 {個言|낱말}입니다(페이지, 피자). English Wiktionary에서 {來|와}ㅆ고, {備考|비고}는 {原語|원어}의 Wiktionary {符號|부호}입니다.',
    },
    people: {
      name: '{人名|인명}',
      about: '{有名|유명}한 {人|사람}의 {姓名|이름}과 {漢字|한자}입니다(김정은 金正恩, 이순신 李舜臣). 위키데이터에서 {人物|인물}을 {選|고르}고 {韓國語|한국어} 위키百科 {文書|문서}에서 {漢字|한자}를 {取|가져}왔으며, {字|글자}의 {音|소리}가 {姓名|이름}과 {合|맞}는 {事|거}ㅅ만 {入|넣}었습니다. {備考|비고}는 위키데이터 {項目|항목}과 {文書|문서} {版|판}입니다.',
    },
    hun: {
      name: '{訓音|훈음} 키',
      about: '{訓音|훈음}으로 {字|글자}를 {探|찾}는 키입니다. 살찔비를 {打|치}면 肥가 {出|나}옵니다. {此|이} 파일의 {讀音|읽기} {欄|칸}에는 키가 {入|들}어 {有|있}습니다. libhangul에서 {來|와}ㅆ습니다.',
    },
  },
}

const en = {
  title: 'Browse the dictionary',
  lede: 'Everything the input method looks words up in. Search by Hangul reading or by Hanja, and narrow to one source.',
  search: 'Hangul reading or Hanja',
  placeholder: 'e.g. 하늘, 天, 讀다',
  filter: 'Source',
  all: 'All',
  row: '%n row',
  rows: '%n rows',
  empty: 'No rows match.',
  loading: 'Loading the dictionary',
  failed: 'The dictionary could not be loaded. It can also be read on GitHub:',
  columns: { reading: 'Reading', spelling: 'Spelling', pos: 'Part of speech', count: 'Count', source: 'Source', note: 'Note' },
  pager: { label: 'Pages', first: 'First', prev: 'Previous', next: 'Next', last: 'Last', page: 'Page' },
  sourcesTitle: 'The sources',
  licence: 'Licence',
  show: 'Show these rows',
  notice: 'Full licence texts and provenance:',
  permission: "Used with the editors' permission",
  scopes: { namu: '나무위키 citations', chat: 'chat words read by hand' },
  pos: {
    noun: 'noun',
    pron: 'pronoun',
    num: 'numeral',
    bound: 'bound noun',
    adv: 'adverb',
    det: 'determiner',
    interj: 'interjection',
    verb: 'verb',
    adj: 'adjective',
  },
  tags: {
    hangul: 'The chat writes this reading in Hangul and never this Hanja.',
    corpus: "Used at least five times in the chat. The reading is each character's most common one, so a rare reading may be wrong.",
    nativised: 'Nativised from a Sino-Korean word.',
    coined: 'Coined in Korea from Chinese morphemes.',
  },
  files: {
    corpus: {
      name: 'Chat spellings',
      about: "Spellings the 釋讀會 chat uses. The project's editors assigned the readings from context; the count is how many times the chat writes each one.",
    },
    editorial: {
      name: "Editors' entries",
      about: "訓讀 words the project's editors added by hand. The note gives the reason.",
    },
    community: {
      name: '標準國語釋讀辭典',
      about: 'The dictionary 釋讀會 members have compiled since June 2022.',
    },
    eonhae: {
      name: 'Old books',
      about: "Readings from character primers such as 訓蒙字會 (1527) and from 諺解, which print the Chinese next to its Korean. The note says where to check each row. The text comes from Korean Wikisource's transcription, mostly unproofread, and 나무위키, which anyone can edit. Hand checks of samples found 2–10% of rows wrong; those are gone, and the rest may still hold errors.",
    },
    'kun-attested': {
      name: 'Printed readings',
      about: "Books of 1882–1945 that print a native reading beside the Hanja, such as 勞動夜學讀本 (1908). The note gives book, year, page and quote. The text is Korean Wikisource's transcription, mostly unproofread.",
    },
    suggested: {
      name: 'Machine suggestions',
      about: 'Hanja a language model proposed for native words with no hand-made entry, unreviewed by any person. A group of the words the chat types most was read by hand, and their notes name what supports the spelling.',
    },
    'place-names': {
      name: 'Place names',
      about: "Native Korean place names with their Hanja, from Japanese Wikipedia's 朝鮮地名音訓一覧, a secondary source with one reference. It gives romanized names only, so the Hangul is transliterated from them.",
    },
    'wikt-native': {
      name: 'Common native words',
      about: "Common native words whose first candidate stayed in Hangul. A row joins the Hanja of the word's parts (눈앞 目前) or follows a 訓音 or JMdict link; no source states a compound's whole spelling. Parts and senses come from English Wiktionary, and every candidate was read by hand.",
    },
    gloss: {
      name: '訓音 guesses',
      about: "Candidates rebuilt by machine from the 訓音 glosses in libhangul's Hanja list (하늘 천 gives 하늘 = 天). Some of the words are wrong.",
    },
    'kun-ja': {
      name: 'Through Japanese',
      about: 'A Korean native word and a Japanese word in JMdict that share an English gloss lend the kanji as a candidate (태어나다 ↔ 産まれる → 產다). Unreviewed, and homonyms make some links wrong.',
    },
    'kun-ja-weak': {
      name: 'Through Japanese, weak',
      about: 'The same method with weaker links, kept so no candidate is lost.',
    },
    'editorial-sino': {
      name: "Editors' 音讀 choices",
      about: 'Words that start in Hangul with a Hanja part (두벌式), and the first candidate among homophones, set by the editors.',
    },
    fusion: {
      name: 'FusionIME',
      about: "音讀 words with usage frequencies from Lee0701's FusionIME (十八字), whose data comes from 표준국어대사전 and libhangul. The count is the cube root of the frequency, rounded.",
    },
    sino: {
      name: 'libhangul',
      about: "音讀 words and characters from libhangul's Hanja list, plus words the chat uses. The count is occurrences in two Telegram chats, 釋讀會 and 漢字混用.",
    },
    etym: {
      name: 'Wiktionary Hanja',
      about: "Hanja spellings the other files lack, from English Wiktionary (kaikki.org), including words Sino-Korean only by origin (재미 滋味, 김치 沉菜). Wiktionary's etymologies are crowd-edited and some are disputed.",
    },
    loanwords: {
      name: 'Loanwords',
      about: "Words borrowed from European languages, Turkish, Arabic, Hindi or Persian, which stay in Hangul (페이지, 피자). From English Wiktionary; the note is the source language's Wiktionary code.",
    },
    people: {
      name: "People's names",
      about: "Well-known people's names and their Hanja (김정은 金正恩, 이순신 李舜臣). People come from Wikidata and Hanja from their Korean Wikipedia articles, kept only when each character's sound matches the name. The note gives the Wikidata item and the article revision.",
    },
    hun: {
      name: '訓音 keys',
      about: 'Keys for finding a character by its 訓音: typing 살찔비 gives 肥. In this file the reading column holds the key. From libhangul.',
    },
  },
}

export default { ko, en }
