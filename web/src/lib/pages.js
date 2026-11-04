// The privacy policy and the support page, in Korean (with {漢字|한자} marks)
// and English. A section's body is a list: a string is a paragraph, an array
// is a bulleted list, and { h, ol | ul | p } is a titled step list, list or
// paragraph (`ol: 'windows'` takes the steps of the install section). `code`
// is monospace.

const MAIL = 'seokdok@mkpo.li'
const ISSUES = 'https://github.com/mkpoli/seokdok/issues'
const TELEGRAM = 'https://t.me/pimsdh'

const ko = {
  privacy: {
    eyebrow: '{個人情報|개인정보}',
    title: '{個人情報|개인정보} {處理|처리}{方針|방침}',
    updated: '2026-11-05',
    note: '{釋讀|석독} {入力器|입력기} 프로젝트가 {直接|직접} {書|쓰}ㄴ {文書|문서}이며 {法律|법률} {檢討|검토}는 {受|받}지 {不|안}ㅎ았습니다. {各|각} {項目|항목}은 2026-11-05에 소스 코드를 {確認|확인}한 {內容|내용}입니다.',
    lede: '{釋讀|석독} {入力器|입력기}와 {此|이} 웹사이트는 {利用者|이용자}의 {個人情報|개인정보}를 {收集|수집}하지 {不|안}ㅎ습니다. {打|치}ㄴ {文|글}은 {機器|기기} {內|안}에서만 {變換|변환}되며 {何處|어디}로도 {傳送|전송}되지 {不|안}ㅎ습니다.',
    sections: [
      {
        id: 'device',
        h: '{機器|기기} {內|안}에 {置|두}는 {事|거}ㅅ',
        body: [
          '{入力器|입력기}(Windows, Linux, iOS, macOS, Android)는 {機器|기기}에 {設置|설치}된 {辭典|사전}만으로 {變換|변환}합니다. 네트워크에 {接續|접속}하는 코드, {分析|분석} {道具|도구}, {誤謬|오류} {報告|보고} {道具|도구}가 {無|없}습니다. Android 앱은 인터넷 {權限|권한}을 {要請|요청}하지 {不|안}ㅎ습니다.',
          [
            '`config.toml`: {設定|설정} {値|값}을 {貯藏|저장}합니다. {位置|위치}는 Windows `%APPDATA%\\seokdok`, Linux `~/.config/seokdok`, macOS `~/Library/Application Support/seokdok`, Android 앱 {專用|전용} {貯藏|저장} {空間|공간}, iOS 앱과 키보드가 {共|함께} {用|쓰}는 App Group 컨테이너입니다.',
            '`custom.json`, `user.tsv`: {利用者|이용자}가 {直接|직접} {置|둔} {境遇|경우}에만 {有|있}습니다. {自作|자작} {規則|규칙}과 {個言|낱말}이 {入|들}어 {有|있}고 {打|치}ㄴ {文|글}은 {入|들}어 {有|있}지 {不|안}ㅎ습니다.',
            '`learned.tsv`: {候補|후보} {窓|창}에서 {選|고르}ㄴ {個言|낱말}과 {其|그} {讀音|읽기}, {回數|횟수}를 {貯藏|저장}합니다. {二|두} {番|번} {選|고르}ㄴ {候補|후보}를 {先|먼저} {見|보}이려는 {事|거}ㅅ입니다. {第一|첫} {候補|후보}를 {其|그}대로 {確定|확정}한 {個言|낱말}, {文章|문장}, {時刻|시각}은 {入|들}어{去|가}지 {不|안}ㅎ습니다. {位置|위치}는 `config.toml`과 {同|같}으며 {何處|어디}로도 {傳送|전송}되지 {不|안}ㅎ습니다.',
          ],
          '{選|고르}ㄴ {個言|낱말}을 {消|지우}려면 {設定|설정}의 「{選|고르}ㄴ {事|거}ㅅ {忘|잊}기」를 {押|누르}거나 `learned.tsv`를 {消|지우}ㅂ니다. 「{選|고르}ㄴ {候補|후보} {學習|학습}」(`input.learn`)을 {滅|끄}면 {學習|학습}도 {使用|사용}도 {止|멈추}고, {消|지우}기 {前|전}까지 파일은 {殘|남}습니다.',
          '{暗號|암호}, {電子郵便|전자우편}, 웹 {住所|주소}, {數字|숫자} 칸과 {學習|학습}을 {禁|막}는 칸(시크릿 탭 {等|등})에서는 {學習|학습}하지 {不|안}ㅎ습니다. {其|그} {判斷|판단}은 앱이 {告|알리}는 칸의 {種類|종류}를 {從|따르}므로, {種類|종류}를 {告|알리}지 {不|안}ㅎ는 앱에서는 {學習|학습}할 {手|수} {有|있}습니다. macOS는 {入力器|입력기}에 {保安|보안} {入力|입력} {與否|여부}만 {告|알리}므로 {暗號|암호} 칸만 {區別|구별}합니다.',
          '{入力器|입력기}는 {此|이} 파일을 {送|보내}지 {不|안}ㅎ지만, {利用者|이용자}나 {組織|조직}이 {用|쓰}는 백업(iCloud와 컴퓨터 백업, Time Machine, Windows {移動|이동} 프로필)은 {異|다르}ㄴ 파일처럼 {此|이} 폴더도 {複寫|복사}할 {手|수} {有|있}습니다.',
          'Windows {入力器|입력기}는 `%LOCALAPPDATA%\\seokdok` 폴더에 `debug`라는 {名|이름}의 파일이 {有|있}을 {時|때}만 {同|같}은 폴더의 `tsf.log`에 {記錄|기록}을 {殘|남}깁니다. {記錄|기록}에는 {時刻|시각}, 프로그램 {名|이름}, 키 {番號|번호}, {變換|변환} {結果|결과}가 {入|들}어 {有|있}어 {打|치}ㄴ {文|글}이 {露|드러나}ㄹ {手|수} {有|있}습니다. `debug` 파일이 {無|없}으면 아무{事|거}ㅅ도 {書|쓰}지 {不|안}ㅎ습니다. {此|이} {記錄|기록}은 {機器|기기} {內|안}에만 {有|있}으며 {傳送|전송}되지 {不|안}ㅎ습니다. `debug`와 `tsf.log`를 {消|지우}면 {無|없}어집니다.',
          'Android 앱은 Android의 {自動|자동} 백업을 {滅|꺼} {置|두}었으므로 앱의 파일이 Google {計定|계정}으로 {複寫|복사}되지 {不|안}ㅎ습니다.',
        ],
      },
      {
        id: 'ios',
        h: 'iOS의 {全體|전체} {接近|접근} {許容|허용}(Full Access)',
        body: [
          'iOS 키보드는 {全體|전체} {接近|접근}을 {許容|허용}해야 앱이 {貯藏|저장}한 {設定|설정}을 {讀|읽}을 {手|수} {有|있}습니다. {此|이} 키보드가 {全體|전체} {接近|접근}을 {用|쓰}는 {處|곳}은 {其|그}{事|거}ㅅ과, 앱이 {消|지우}ㄹ {手|수} {有|있}는 {共用|공용} 폴더에 `learned.tsv`를 {置|두}는 {事|거}ㅅ뿐입니다.',
          [
            '{許容|허용}하지 {不|안}ㅎ아도 키보드는 {基本|기본} {設定|설정}으로 {全|모든} {機能|기능}이 {動作|동작}합니다. `learned.tsv`는 키보드 {自體|자체} 폴더에 {置|두}며, 앱이 {其|그}{處|곳}에 {接近|접근}하지 {不|못}하므로 키보드의 釋 키를 길게 {押|눌러} {消|지우}거나 {學習|학습}을 {滅|끄}ㅂ니다. {其|그} 폴더에 {書|쓰}지 {不|못}하면 키보드가 {動作|동작}하는 {間|동안} 메모리에만 {有|있}습니다.',
            'Apple {文書|문서}에 {從|따르}면 {全體|전체} {接近|접근}을 {受|받}은 키보드는 키 {入力|입력}을 네트워크로 {送|보내}ㄹ {手|수} {有|있}습니다. {此|이} 키보드에는 네트워크를 {用|쓰}는 코드가 {無|없}으므로 {打|치}ㄴ {文|글}은 {機器|기기}를 {離|떠}나지 {不|안}ㅎ습니다.',
            '키보드는 {何|어}느 앱에서나 {字|글자}를 {變換|변환}하려고 메모리 {內|안}에서 {操|다루}며, {貯藏|저장}하는 {事|거}ㅅ은 {候補|후보} {窓|창}에서 {選|고르}ㄴ {個言|낱말}(`learned.tsv`)뿐입니다.',
          ],
        ],
      },
      {
        id: 'web',
        h: '웹사이트(seokdok.mkpo.li)',
        body: [
          [
            '{打|치}ㄴ {文|글}: {此|이} 페이지의 {入力|입력} 칸과 {變換|변환} 칸에 {置|둔} {文|글}은 브라우저 {內|안}(WebAssembly)에서 {處理|처리}됩니다. 서버로 {去|가}는 {要請|요청}은 {辭典|사전} 파일(`/dict/native.bin`, `/dict/sino.bin`)뿐입니다. {複寫|복사} 버튼을 {押|누르}면 {其|그} {文|글}이 클립보드에 {入|들}어{去|가}ㅂ니다.',
            '브라우저 {貯藏|저장}: `localStorage`에 `seokdok:theme`, `seokdok:lang`, `seokdok:options`, `seokdok:custom`, `seokdok:learned`이 {貯藏|저장}됩니다. {明|밝}기, {言語|언어}, {表記|표기} {設定|설정}, 「{自作|자작}」에 {入|넣}은 {規則|규칙}과 {個言|낱말}, {候補|후보} {窓|창}에서 {選|고르}ㄴ {個言|낱말}입니다. {選|고르}ㄴ {個言|낱말}은 {入力|입력} {設定|설정}의 「{選|고르}ㄴ {事|거}ㅅ {忘|잊}기」로 {消|지우}ㅂ니다. 브라우저 {內|안}에만 {有|있}고 쿠키는 {用|쓰}지 {不|안}ㅎ으며, 사이트 데이터를 {消|지우}면 {無|없}어집니다.',
            '글꼴: {此|이} 사이트의 글꼴(Noto Sans KR, Noto Serif KR, Gowun Batang)은 사이트 {自體|자체}에서 {提供|제공}되며 {異|다르}ㄴ 서버에 {要請|요청}하지 {不|안}ㅎ습니다.',
            '호스팅: Cloudflare가 사이트를 {提供|제공}합니다. 페이지를 {開|열}면 Cloudflare는 {何|어}느 웹 호스트나 하듯 IP {住所|주소}, {要請|요청}한 {住所|주소}, 브라우저 {情報|정보}를 {運營|운영}에 {必要|필요}한 {範圍|범위}에서 {處理|처리}하며, {此|이}는 Cloudflare의 {方針|방침}을 {從|따르}ㅂ니다.',
            '프로젝트의 {設定|설정}(`wrangler.jsonc`)은 {靜的|정적} 파일만 {提供|제공}하고 서버 코드, {分析|분석} {道具|도구}, 로그 {收集|수집}을 {點|켜}지 {不|안}ㅎ았으며 페이지는 `Cache-Control: no-transform` 머리글로 {提供|제공}되어 Cloudflare가 {分析|분석} 비컨을 {入|넣}지 {不|못}하며, 페이지는 {異|다르}ㄴ 서버로 {要請|요청}을 {送|보내}지 {不|안}ㅎ습니다. Cloudflare {管理|관리} {畵面|화면}에서 프로젝트가 {見|보}ㄹ {手|수} {有|있}는 {集計|집계}는 {此|이} {貯藏所|저장소}로 {確認|확인}할 {手|수} {無|없}습니다.',
            'GitHub, Telegram, 위키로 {去|가}는 링크는 {各|각} 서비스의 {方針|방침}을 {從|따르}ㅂ니다.',
          ],
        ],
      },
      {
        id: 'dict',
        h: '{辭典|사전}의 {出處|출처}',
        body: [
          '{辭典|사전}은 libhangul, FusionIME, English Wiktionary(kaikki.org), JMdict, 위키文獻의 옛 {冊|책}, 標準國語釋讀辭典에서 {來|와}ㅆ습니다. {出處|출처}와 {許可|허가}는 `dict/README.md`와 페이지 {下|아래}의 「{辭典|사전}과 {出處|출처}」에 {有|있}습니다.',
          '{用例|용례} {回數|횟수}는 {釋讀會|풀어읽기모임} Telegram {對話房|대화방}(t.me/pimsdh)의 {對話|대화} {記錄|기록}(2026-10-29까지)에서 {算|세}었습니다. {入力器|입력기}와 {別|따로} {會|모}은 {事|거}ㅅ이며 {對話|대화} {原文|원문}은 {此|이} {貯藏所|저장소}에 {入|들}어 {有|있}지 {不|안}ㅎ습니다. {入力器|입력기}나 {此|이} 페이지에서 {打|치}ㄴ {文|글}은 {此|이} {回數|횟수}에 {入|들}어{去|가}지 {不|안}ㅎ습니다.',
          '{機械|기계} {提案|제안}({言語|언어} 모델이 {漢字|한자}를 {附|붙}인 {辭典|사전}의 {一部|일부})은 English Wiktionary의 {意|뜻}{解|풀이}와 libhangul의 {訓音|훈음} {解|풀이}에서 {作|만들}었습니다. {何|어}떤 {個言|낱말}을 {選|고르}ㄹ지는 Telegram {對話|대화}에서 {其|그} {個言|낱말}이 {用|쓰}인 {回數|횟수}로 {定|정}했습니다. {入力器|입력기}나 {此|이} 페이지에서 {打|치}ㄴ {文|글}은 {用|쓰}인 적이 {無|없}습니다.',
        ],
      },
      {
        id: 'contact',
        h: '{問議|문의}와 {變更|변경}',
        body: [
          [`{電子郵便|전자우편}: ${MAIL}`, `GitHub: mkpoli (${ISSUES})`, `Telegram: ${TELEGRAM}`],
          '{此|이} {方針|방침}은 {貯藏所|저장소}의 `web/src/lib/pages.js`에 {有|있}으며 {變更|변경} {履歷|이력}은 GitHub에서 {見|보}ㄹ {手|수} {有|있}습니다. {內容|내용}을 {變|바꾸}면 {上|위}의 {日|날짜}도 {改|고치}ㅂ니다.',
        ],
      },
    ],
  },
  support: {
    eyebrow: '{支援|지원}',
    title: '{使用|사용}법과 {問議|문의}',
    lede: '{釋讀|석독} {入力器|입력기}를 {點|켜}는 {方法|방법}, {頻|자주} {經|겪}는 {問題|문제}, {錯|틀}린 {變換|변환}을 {告|알}리는 {方法|방법}입니다.',
    sections: [
      {
        id: 'enable',
        h: '{點|켜}는 {方法|방법}',
        body: [
          { h: 'Windows', ol: 'windows' },
          { h: 'Linux (fcitx5)', ol: 'linux' },
          {
            h: 'iOS',
            ol: [
              '앱을 {設置|설치}합니다.',
              '{設定|설정} › 일반 › 키보드 › 키보드 › 새로운 키보드 추가에서 {釋讀|석독}을 {加|더}합니다.',
              '지구본 키를 {長|길}게 {押|눌러} {釋讀|석독}을 {選|고르}ㅂ니다. Space로 {變換|변환}하고 {候補|후보}를 {押|눌러} {確定|확정}합니다.',
              '앱의 {設定|설정}을 키보드에 {反映|반영}하려면 {此|이} 키보드의 「{全體|전체} {接近|접근} {許容|허용}」을 {點|켜}ㅂ니다. {滅|꺼} {置|두}어도 {基本|기본} {設定|설정}으로 {動作|동작}합니다.',
            ],
          },
          {
            h: 'macOS',
            ol: [
              '앱을 `~/Library/Input Methods`에 {入|넣}고 {再|다시} 로그인합니다.',
              '시스템 설정 › 키보드 › 입력 소스 › 편집…에서 +를 {押|눌러} {韓國語|한국어}의 {釋讀|석독}을 {加|더}합니다.',
              '메뉴 막대의 {入力|입력} 메뉴에서 {釋讀|석독}을 {選|고르}ㅂ니다. {設定|설정}은 {同|같}은 메뉴의 「{設定|설정}…」에 {有|있}습니다.',
            ],
          },
          {
            h: 'Android',
            ol: [
              'APK를 {設置|설치}하고 {釋讀|석독} 앱을 {開|여}ㅂ니다.',
              '「키보드 {設定|설정} 열기」에서 {釋讀|석독}을 {點|켜}ㅂ니다.',
              '「{入力|입력} 방법 고르기」나 Space 키를 {長|길}게 {押|눌러} {釋讀|석독}을 {選|고르}ㅂ니다.',
            ],
          },
        ],
      },
      {
        id: 'keys',
        h: '{模式|모드}를 {變|바꾸}는 {鍵|키}',
        body: [
          '{釋讀|석독}, {韓㐎|한글}, A(라틴) {三|세} {模式|모드}가 {有|있}습니다. {韓|한} ↔ A는 {直前|직전}에 {用|쓰}던 {釋讀|석독}이나 {韓㐎|한글}로 {返|돌아}갑니다. {文字|문자}를 {變|바꾸}는 {鍵|키}는 {釋讀|석독}, {國漢文|국한문} {混用|혼용}, {韓㐎|한글} {專用|전용}을 {次例|차례}로 {設定|설정}에 {書|쓰}고 {釋讀|석독} {模式|모드}로 {入|들어}가며, {開|열}린 {應用|응용} {皆|모두}에 {適用|적용}됩니다. {韓㐎|한글} {模式|모드}는 Windows 메뉴, macOS {入力|입력} 소스, `hangul` {目錄|목록}으로 {入|들어}갑니다. {修飾|수식} {鍵|키}만 {書|쓰}인 {項目|항목}은 {其|그} {鍵|키}만 {押|눌렀}다 {離|떼}는 {事|것}입니다. {目錄|목록}은 `config.toml`의 `[input.keys]`에 {有|있}습니다.',
          {
            h: 'Windows',
            ul: [
              '`한/영`, {右|오른}쪽 `Alt`, `半角/全角`: {韓|한} ↔ A. {右|오른}쪽 Alt가 AltGr인 {配列|배열}에서는 AltGr {其|그}대로입니다.',
              '`Shift+한/영`, {左|왼}쪽 `Ctrl`과 {左|왼}쪽 `Alt`를 {共|함께}: {釋讀|석독} → {國漢文|국한문} {混用|혼용} → {韓㐎|한글} {專用|전용}',
              '`한자`, {右|오른}쪽 `Ctrl`, `変換`, `F7`: {變換|변환}',
              '`無変換`: A로. `変換`: A에서 {韓|한}으로',
            ],
          },
          {
            h: 'macOS',
            ul: [
              '`Option`: {韓|한} ↔ A',
              '{左|왼}쪽 `Control`과 {左|왼}쪽 `Option`을 {共|함께}: {釋讀|석독} → {國漢文|국한문} {混用|혼용} → {韓㐎|한글} {專用|전용}',
              '`Option+Return`, `F7`: {變換|변환}',
              '`英数`: A로. `かな`: A에서 {韓|한}으로',
              '`Control+Space`, Globe, Caps Lock은 macOS가 {入力|입력} 소스를 {變|바꾸}는 데 {用|쓰}며, {釋讀|석독}, {釋讀|석독} {韓㐎|한글}, {釋讀|석독} A는 {各各|각각} {入力|입력} 소스입니다.',
            ],
          },
          {
            h: 'Linux (fcitx5)',
            ul: [
              '`Hangul`, `Shift+Space`, {右|오른}쪽 `Alt`, `半角/全角`: {韓|한} ↔ A',
              '`Shift+Hangul`, {左|왼}쪽 `Ctrl`과 {左|왼}쪽 `Alt`를 {共|함께}: {釋讀|석독} → {國漢文|국한문} {混用|혼용} → {韓㐎|한글} {專用|전용}',
              '`Hanja`, `F9`, `変換`, `F7`: {變換|변환}',
              '`無変換`: A로. `変換`: A에서 {韓|한}으로',
              '{釋讀|석독}을 {打|치}는 {中|중}에는 `Shift+Space`가 띄어{書|쓰}기입니다.',
              'fcitx5 {群|그룹}에 {入力器|입력기}가 {二|둘} {以上|이상}이면 `Ctrl+Space`, `Hangul`, `半角/全角`, {左|왼}쪽 `Shift`는 fcitx5가 {先|먼저} {用|쓰}ㅂ니다.',
            ],
          },
          {
            h: 'Android',
            ul: [
              '{物理|물리} 키보드의 `한/영`, `半角/全角`, `Shift+Space`: {韓|한} ↔ A. {釋讀|석독}을 {打|치}는 {中|중}에는 `Shift+Space`가 띄어{書|쓰}기입니다.',
              '`Shift+한/영`, {左|왼}쪽 `Ctrl`과 {左|왼}쪽 `Alt`를 {共|함께}: {釋讀|석독} → {國漢文|국한문} {混用|혼용} → {韓㐎|한글} {專用|전용}',
              '`한자`, `変換`, `F7`: {變換|변환}',
              '`Ctrl+Space`와 {言語|언어} {鍵|키}는 Android가 {用|쓰}ㅂ니다. Samsung {機器|기기}는 `Shift+Space`도 {用|쓰}며, {物理|물리} 키보드 {設定|설정}의 {言語|언어} {變更|변경} {短縮|단축}키에서 {解|끄}ㄹ {手|수} {有|있}습니다.',
              '{畵面|화면} 키보드의 {模式|모드} {鍵|키}는 釋 → 混 → 한 → A {順|차례}로 {變|바꾸}ㅂ니다.',
            ],
          },
          {
            h: 'iOS',
            p: '{畵面|화면}의 {模式|모드} {鍵|키}가 釋 → 混 → 한 → A로 {順|차례}대로 {變|바꾸}ㅂ니다. {物理|물리} 키보드의 {鍵|키}는 {此|이} 키보드에 {來|오}지 {不|않}습니다.',
          },
        ],
      },
      {
        id: 'problems',
        h: '{頻|자주} {經|겪}는 {問題|문제}',
        body: [
          { h: '{古|예}ㅅ{韓㐎|한글}이 {四角|네모}로 {見|보}입니다', p: '{入力器|입력기}는 {字|글자}를 {正|바르}게 {送|보내}지만, 앱이 {自己|자기} 글꼴을 {用|쓰}고 {其|그} 글꼴에 {古|예}ㅅ{韓㐎|한글}이 {無|없}으면 {四角|네모}로 {出|나}옵니다. 앱에서 Noto Serif CJK KR, Noto Sans CJK KR, 함초롬바탕처럼 {古|예}ㅅ{韓㐎|한글}을 {支援|지원}하는 글꼴을 {選|고르}ㅂ니다. {此|이} 사이트는 {自體|자체} 글꼴을 {呼|불러}오므로 {此處|여기}서는 {正|제대로} {見|보}입니다.' },
          { h: 'iPhone에서 {基本|기본} 키보드로 {返|돌아}갑니다', p: 'iOS는 메모리를 {多|많}이 {用|쓰}는 키보드를 {終|끝}냅니다. {此|이} 키보드는 {辭典|사전}을 {讀|읽}은 {後|뒤} {約|약} 66 MB를 {用|쓰}며(시뮬레이터에서 {測|재}ㄴ {値|값}, 2026-11-04), {知|알}려진 {限度|한도} {推定値|추정치}보다 {大|크}ㅂ니다. 지구본 키로 {釋讀|석독}을 {再|다시} {選|고르}ㅂ니다.' },
          { h: 'Windows에서 {更新|갱신}한 {後|뒤}에도 {古|예}ㅅ {動作|동작}이 {殘|남}습니다', p: '{更新|갱신}할 {時|때} {開|열}려 {有|있}던 앱은 {古|예}ㅅ 버전을 {其|그}대로 {握|쥐}고 {有|있}습니다. {其|그} 앱을 {閉|닫}았다가 {再|다시} {開|여}ㅂ니다. {然|그래}도 {同|같}으면 로그아웃하거나 Windows를 {再|다시} {始作|시작}합니다.' },
          { h: 'Linux에서 {釋讀|석독}이 {目錄|목록}에 {無|없}습니다', p: '`fcitx5 -rd`로 fcitx5를 {再|다시} {始作|시작}하고, fcitx5-configtool에서 「Only Show Current Language」를 {解除|해제}합니다.' },
          { h: '{設定|설정}이 {適用|적용}되지 {不|안}ㅎ습니다', p: '{異|다르}ㄴ {入力器|입력기}로 {去|가}ㅆ다가 {返|돌아}오면 config.toml을 {再|다시} {讀|읽}습니다. iOS는 앱의 {設定|설정}을 {用|쓰}려면 「{全體|전체} {接近|접근} {許容|허용}」이 {必要|필요}합니다.' },
        ],
      },
      {
        id: 'report',
        h: '{錯|틀}린 {變換|변환}을 {告|알}리는 {方法|방법}',
        body: [
          '{下|아래} {內容|내용}을 GitHub 이슈나 Telegram에 {記|적}어 {授|주}십시오.',
          [
            '{打|치}ㄴ {文|글}을 {韓㐎|한글} {其|그}대로. {短|짧}은 {例|예}면 {充分|충분}합니다.',
            '{出|나}온 {結果|결과}와 {望|바라}는 {結果|결과}.',
            '{用|쓰}는 {環境|환경}과 버전(Windows, Linux, iOS, macOS, Android).',
            '{表記|표기} {設定|설정}을 {變|바꾸}었다면 {其|그} {內容|내용}({末音添記|말음첨기} {方式|방식}, {國漢文|국한문} {混用|혼용} {等|등}).',
            '{個人的|개인적}인 {文|글}은 {記|적}지 {不|안}ㅎ습니다. {問題|문제}가 {成|되}는 {個言|낱말}만으로 {充分|충분}합니다.',
          ],
          '{機械|기계} {提案|제안} {辭典|사전}은 {人|사람}이 {檢討|검토}하지 {不|안}ㅎ아 {錯|틀}릴 {手|수} {有|있}습니다. {直|바로} {改|고치}려면 {此|이} 사이트의 「{自作|자작}」에서 {規則|규칙}이나 {個言|낱말}을 {加|더}하면 {成|되}ㅂ니다.',
        ],
      },
      {
        id: 'contact',
        h: '{連絡處|연락처}',
        body: [[`GitHub 이슈: ${ISSUES}`, `Telegram: ${TELEGRAM}`, `{電子郵便|전자우편}: ${MAIL}`]],
      },
    ],
  },
}

const en = {
  privacy: {
    eyebrow: 'Privacy',
    title: 'Privacy policy',
    updated: '2026-11-05',
    note: 'Written by the 釋讀 input method project, without legal review. Each statement describes the source code as checked on 2026-11-05.',
    lede: 'The 釋讀 input methods and this website collect no personal data. Text you type is converted on your device and is not sent anywhere.',
    sections: [
      {
        id: 'device',
        h: 'What stays on your device',
        body: [
          'The input methods (Windows, Linux, iOS, macOS, Android) convert with the dictionaries installed on the device. They contain no code that connects to a network, no analytics and no crash reporting. The Android app does not request the Internet permission.',
          [
            '`config.toml` holds your settings. Windows: `%APPDATA%\\seokdok`. Linux: `~/.config/seokdok`. macOS: `~/Library/Application Support/seokdok`. Android: the app’s private storage. iOS: the App Group container shared by the app and the keyboard.',
            '`custom.json` and `user.tsv` exist only if you put them there. They hold your own words and rules, not what you typed.',
            '`learned.tsv` holds the words you picked from the candidate list, with their readings and counts, so that a candidate picked twice comes first. Words you accept as first offered, sentences and times of day are not in it. It sits beside `config.toml` and is not sent anywhere.',
          ],
          'To clear the words you picked, press “Forget what I picked” in the settings or delete `learned.tsv`. Turning off “Learn picks” (`input.learn`) stops both learning and using them, and the file stays until you clear it.',
          'Nothing is learned in password, email, web address and number fields, or in fields that ask for no learning, such as private tabs. This relies on the app naming the kind of field, so an app that does not name it can still be learned from. macOS tells an input method only about secure entry, so there only password fields are told apart.',
          'The input methods never send this file, but a backup you or your organisation run (iCloud and computer backups, Time Machine, a Windows roaming profile) can copy this folder as it does any other.',
          'The Windows input method writes a log to `tsf.log` in `%LOCALAPPDATA%\\seokdok` only while a file named `debug` exists in that folder. The log has a timestamp, the program name, key codes and conversion results, so it can contain what you typed. Without the `debug` file nothing is written. The log stays on the device and is never sent; delete `debug` and `tsf.log` to remove it.',
          'The Android app turns off Android’s automatic backup, so its files are not copied to a Google account.',
        ],
      },
      {
        id: 'ios',
        h: 'iOS Full Access',
        body: [
          'An iOS keyboard can read the settings the app saved only when Full Access is on. This keyboard uses it for that, and to keep `learned.tsv` in the shared folder where the app can clear it.',
          [
            'With Full Access off, the keyboard works completely on its default settings. It keeps `learned.tsv` in its own folder, which the app cannot reach, so hold the keyboard’s 釋 key to clear it or to stop learning. If that folder cannot be written, the picks last only while the keyboard is running.',
            'Apple’s documentation says a keyboard with Full Access can send keystrokes and other input events over the network. This keyboard has no code that uses the network, so typed text does not leave the device.',
            'In every app, the keyboard handles the letters you type in memory to convert them. The only thing it stores is the words you pick from the candidate list (`learned.tsv`).',
          ],
        ],
      },
      {
        id: 'web',
        h: 'This website (seokdok.mkpo.li)',
        body: [
          [
            'Typed text: text in the input and converter fields on this page is processed in your browser (WebAssembly). The only requests to the server are for the dictionary files (`/dict/native.bin`, `/dict/sino.bin`). The copy buttons put text on your clipboard.',
            'Browser storage: `localStorage` keeps `seokdok:theme`, `seokdok:lang`, `seokdok:options`, `seokdok:custom` and `seokdok:learned`: your colour scheme, language, spelling settings, the rules and words you entered under Customize, and the words you picked from the candidate list. “Forget what I picked” in the input settings clears those words. They stay in your browser, no cookies are used, and clearing the site’s data removes them.',
            'Fonts: the site’s fonts (Noto Sans KR, Noto Serif KR, Gowun Batang) are served from the site itself.',
            'Hosting: Cloudflare serves the site. When you open a page, Cloudflare processes your IP address, the requested address and your browser details as needed to operate, as any web host does, under Cloudflare’s own policy.',
            'The project’s configuration (`wrangler.jsonc`) serves static files only, with no server code, analytics or log collection switched on, and the page itself includes no analytics script. The pages are served with `Cache-Control: no-transform`, so Cloudflare cannot add its analytics beacon, and they send no requests to other servers. What the project could see in the Cloudflare dashboard cannot be checked from the repository.',
            'Links to GitHub, Telegram and the wiki lead to those services, which apply their own policies.',
          ],
        ],
      },
      {
        id: 'dict',
        h: 'Where the dictionaries come from',
        body: [
          'The dictionaries come from libhangul, FusionIME, English Wiktionary (kaikki.org), JMdict, old books on Korean Wikisource and 標準國語釋讀辭典. Sources and licences are in `dict/README.md` and under “Dictionaries and sources” at the foot of the page.',
          'Usage counts were taken from the chat history of the 釋讀會 Telegram group (t.me/pimsdh) up to 2026-10-29. This was gathered separately from the input methods, and the chat messages themselves are not in this repository. Text typed in the input methods or on this page does not feed these counts.',
          'The machine-suggested entries (Hanja added by a language model) were made from English Wiktionary senses and libhangul glosses. Which words to include was decided by how often the Telegram chat uses them. Text typed into the input methods or this page was never used.',
        ],
      },
      {
        id: 'contact',
        h: 'Contact and changes',
        body: [
          [`Email: ${MAIL}`, `GitHub: mkpoli (${ISSUES})`, `Telegram: ${TELEGRAM}`],
          'This policy lives in the repository at `web/src/lib/pages.js`, where its history is visible on GitHub. The date above changes when the policy does.',
        ],
      },
    ],
  },
  support: {
    eyebrow: 'Support',
    title: 'Help and contact',
    lede: 'How to turn the keyboard on, common problems, and how to report a wrong conversion.',
    sections: [
      {
        id: 'enable',
        h: 'Turn it on',
        body: [
          { h: 'Windows', ol: 'windows' },
          { h: 'Linux (fcitx5)', ol: 'linux' },
          {
            h: 'iOS',
            ol: [
              'Install the app.',
              'Add 釋讀 under Settings › General › Keyboard › Keyboards › Add New Keyboard.',
              'Hold the globe key and pick 釋讀. Space converts; tap a candidate to confirm.',
              'To apply the app’s settings in the keyboard, turn on Allow Full Access for it. The keyboard works on its default settings without it.',
            ],
          },
          {
            h: 'macOS',
            ol: [
              'Copy the app to `~/Library/Input Methods` and log in again.',
              'In System Settings › Keyboard › Input Sources › Edit…, press + and add 釋讀 under Korean.',
              'Pick 釋讀 in the input menu of the menu bar. Settings are in that menu under Settings….',
            ],
          },
          {
            h: 'Android',
            ol: [
              'Install the APK and open the 釋讀 app.',
              'Tap “Open keyboard settings” and enable 釋讀.',
              'Pick 釋讀 with “Choose input method”, or hold the space bar while typing.',
            ],
          },
        ],
      },
      {
        id: 'keys',
        h: 'Keys that switch modes',
        body: [
          'There are three modes: 釋讀, 한글 and A (Latin). Korean ↔ Latin comes back to whichever of 釋讀 and 한글 you last used. The script key writes 釋讀, 國漢文混用 or 韓㐎 專用 to the settings in turn and enters 釋讀 mode, in every open application. Plain 한글 is in the Windows menu, the macOS input sources and the `hangul` list. A modifier written alone means pressing and releasing it with no other key. The lists are `[input.keys]` in `config.toml`.',
          {
            h: 'Windows',
            ul: [
              '`Han/Eng`, right `Alt`, `半角/全角`: Korean ↔ Latin. On a layout where right Alt is AltGr, it stays AltGr.',
              '`Shift+Han/Eng`, or left `Ctrl` and left `Alt` tapped together: 釋讀 → 國漢文混用 → 韓㐎 專用',
              '`Hanja`, right `Ctrl`, `変換`, `F7`: convert',
              '`無変換`: to Latin. `変換`: from Latin back to Korean',
            ],
          },
          {
            h: 'macOS',
            ul: [
              '`Option` tapped alone: Korean ↔ Latin',
              'Left `Control` and left `Option` tapped together: 釋讀 → 國漢文混用 → 韓㐎 專用',
              '`Option+Return`, `F7`: convert',
              '`英数`: to Latin. `かな`: from Latin back to Korean',
              '`Control+Space`, Globe and Caps Lock belong to macOS, which moves between input sources with them. 釋讀, 釋讀 한글 and 釋讀 A are separate input sources.',
            ],
          },
          {
            h: 'Linux (fcitx5)',
            ul: [
              '`Hangul`, `Shift+Space`, right `Alt`, `半角/全角`: Korean ↔ Latin',
              '`Shift+Hangul`, or left `Ctrl` and left `Alt` tapped together: 釋讀 → 國漢文混用 → 韓㐎 專用',
              '`Hanja`, `F9`, `変換`, `F7`: convert',
              '`無変換`: to Latin. `変換`: from Latin back to Korean',
              'While 釋讀 is composing, `Shift+Space` is a space inside the composition.',
              'With more than one input method in the fcitx5 group, fcitx5 itself takes `Ctrl+Space`, `Hangul`, `半角/全角` and a tap of left `Shift`.',
            ],
          },
          {
            h: 'Android',
            ul: [
              '`Han/Eng`, `半角/全角`, `Shift+Space` on a hardware keyboard: Korean ↔ Latin. While 釋讀 is composing, `Shift+Space` is a space inside the composition.',
              '`Shift+Han/Eng`, or left `Ctrl` and left `Alt` tapped together: 釋讀 → 國漢文混用 → 韓㐎 專用',
              '`Hanja`, `変換`, `F7`: convert',
              'Android keeps `Ctrl+Space` and the language key. Samsung devices also use `Shift+Space`; “Change language shortcut” in the physical keyboard settings turns that off.',
              'The mode key on screen goes 釋 → 混 → 한 → A.',
            ],
          },
          {
            h: 'iOS',
            p: 'The mode key on screen goes 釋 → 混 → 한 → A. Keys on a hardware keyboard do not reach this keyboard.',
          },
        ],
      },
      {
        id: 'problems',
        h: 'Common problems',
        body: [
          { h: 'Old letters show as boxes', p: 'The keyboard sends the letters correctly, but an app that uses its own font shows a box when that font has no old Hangul. Pick a font with old Hangul support in the app, such as Noto Serif CJK KR, Noto Sans CJK KR or HCR Batang. This website loads its own font, so the letters show here.' },
          { h: 'The system keyboard comes back on iPhone', p: 'iOS ends keyboards that use much memory. After loading its dictionaries this keyboard uses about 66 MB (measured on a simulator, 2026-11-04), above the published estimates of the limit. Pick 釋讀 again with the globe key.' },
          { h: 'After updating on Windows, the old behaviour remains', p: 'Apps that were open during the update keep the old version loaded. Close and reopen them. If it persists, sign out or restart Windows.' },
          { h: '釋讀 is missing from the list on Linux', p: 'Restart fcitx5 with `fcitx5 -rd`, then untick “Only Show Current Language” in fcitx5-configtool.' },
          { h: 'Settings have no effect', p: 'Switch to another input method and back; config.toml is read again then. On iOS, the app’s settings need Full Access.' },
        ],
      },
      {
        id: 'report',
        h: 'Report a wrong conversion',
        body: [
          'Write the following in a GitHub issue or in the Telegram group.',
          [
            'The text you typed, as Hangul. A short example is enough.',
            'What you got and what you expected.',
            'Your platform and version.',
            'Your spelling settings, if you changed them (末音添記 style, Hanja-mixed mode and so on).',
            'Leave out private text. The problem word alone is enough.',
          ],
          'The machine-suggested dictionary entries are not reviewed by a person and can be wrong. To fix a word for yourself at once, add a rule or word under Customize on this site.',
        ],
      },
      {
        id: 'contact',
        h: 'Contact',
        body: [[`GitHub issues: ${ISSUES}`, `Telegram: ${TELEGRAM}`, `Email: ${MAIL}`]],
      },
    ],
  },
}

export default { ko, en }
