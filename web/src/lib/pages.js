// The privacy policy and the support page, in Korean (with {漢字|한자} marks)
// and English. A section's body is a list: a string is a paragraph, an array
// is a bulleted list, and { h, ol | p } is a titled step list or paragraph
// (`ol: 'windows'` takes the steps of the install section). `code` is
// monospace.

const MAIL = 'mkpoli@mkpo.li'
const ISSUES = 'https://github.com/mkpoli/seokdok/issues'
const TELEGRAM = 'https://t.me/pimsdh'

const ko = {
  privacy: {
    eyebrow: '{個人情報|개인정보}',
    title: '{個人情報|개인정보} {處理|처리}{方針|방침}',
    updated: '2026-11-04',
    note: '{釋讀|석독} {入力器|입력기} 프로젝트가 직접 쓴 문서이며 {法律|법률} {檢討|검토}는 받지 않았습니다. 각 항목은 2026-11-04에 소스 코드를 확인한 내용입니다.',
    lede: '{釋讀|석독} {入力器|입력기}와 이 웹사이트는 {利用者|이용자}의 {個人情報|개인정보}를 {收集|수집}하지 않습니다. {打|치}ㄴ 글은 {機器|기기} 안에서만 {變換|변환}되며 어디로도 전송되지 않습니다.',
    sections: [
      {
        id: 'device',
        h: '{機器|기기} 안에 {置|두}는 것',
        body: [
          '{入力器|입력기}(Windows, Linux, iOS, macOS, Android)는 {機器|기기}에 {設置|설치}된 {辭典|사전}만으로 {變換|변환}합니다. 네트워크에 접속하는 코드, 분석 도구, 오류 보고 도구가 없습니다. Android 앱은 인터넷 {權限|권한}을 요청하지 않습니다.',
          [
            '`config.toml`: {設定|설정} 값을 {保存|저장}합니다. 위치는 Windows `%APPDATA%\\seokdok`, Linux `~/.config/seokdok`, macOS `~/Library/Application Support/seokdok`, Android 앱 전용 저장 공간, iOS 앱과 키보드가 함께 쓰는 App Group 컨테이너입니다.',
            '`custom.json`, `user.tsv`: {利用者|이용자}가 직접 {置|둔} 경우에만 {有|있}습니다. {自作|자작} {規則|규칙}과 {個言|낱말}이 들어 있고 {打|치}ㄴ 글은 들어 있지 않습니다.',
          ],
          'Windows {入力器|입력기}는 `%LOCALAPPDATA%\\seokdok` 폴더에 `debug`라는 이름의 파일이 {有|있}을 때만 같은 폴더의 `tsf.log`에 {記錄|기록}을 남깁니다. {記錄|기록}에는 시각, 프로그램 이름, 키 번호, {變換|변환} {結果|결과}가 들어 있어 {打|치}ㄴ 글이 드러날 수 있습니다. `debug` 파일이 {無|없}으면 아무것도 쓰지 않습니다. 이 {記錄|기록}은 {機器|기기} 안에만 {有|있}으며 전송되지 않습니다. `debug`와 `tsf.log`를 지우면 {無|없}어집니다.',
          'Android 앱은 Android의 자동 백업을 꺼 두었으므로 앱의 파일이 Google 계정으로 복사되지 않습니다.',
        ],
      },
      {
        id: 'ios',
        h: 'iOS의 {全體|전체} {接近|접근} {許用|허용}(Full Access)',
        body: [
          'iOS 키보드는 {全體|전체} {接近|접근}을 {許用|허용}해야 앱이 {保存|저장}한 {設定|설정}을 읽을 수 {有|있}습니다. 이 키보드가 {全體|전체} {接近|접근}을 쓰는 곳은 그것뿐입니다.',
          [
            '{許用|허용}하지 않아도 키보드는 기본 {設定|설정}으로 모든 기능이 동작합니다.',
            'Apple 문서에 따르면 {全體|전체} {接近|접근}을 받은 키보드는 키 입력을 네트워크로 보낼 수 {有|있}습니다. 이 키보드에는 네트워크를 쓰는 코드가 {無|없}으므로 {打|치}ㄴ 글은 {機器|기기}를 떠나지 않습니다.',
            '키보드는 어느 앱에서나 글자를 {變換|변환}하려고 메모리 안에서만 다루며 {保存|저장}하지 않습니다.',
          ],
        ],
      },
      {
        id: 'web',
        h: '웹사이트(seokdok.mkpo.li)',
        body: [
          [
            '{打|치}ㄴ 글: 이 페이지의 {入力|입력} 칸과 {變換|변환} 칸에 {置|둔} 글은 브라우저 안(WebAssembly)에서 처리됩니다. 서버로 가는 요청은 {辭典|사전} 파일(`/dict/native.bin`, `/dict/sino.bin`)뿐입니다. {複寫|복사} 버튼을 {押|누르}면 그 글이 클립보드에 들어갑니다.',
            '브라우저 {保存|저장}: `localStorage`에 `seokdok:theme`, `seokdok:lang`, `seokdok:options`, `seokdok:custom`이 {保存|저장}됩니다. 밝기, {言語|언어}, {表記|표기} {設定|설정}, 「{自作|자작}」에 {入|넣}은 {規則|규칙}과 {個言|낱말}입니다. 브라우저 안에만 {有|있}고 쿠키는 쓰지 않으며, 사이트 데이터를 지우면 {無|없}어집니다.',
            '글꼴: 이 사이트의 글꼴(Noto Sans KR, Noto Serif KR, Gowun Batang)은 사이트 자체에서 제공되며 다른 서버에 요청하지 않습니다.',
            '호스팅: Cloudflare가 사이트를 제공합니다. 페이지를 열면 Cloudflare는 어느 웹 호스트나 하듯 IP 주소, 요청한 주소, 브라우저 정보를 운영에 필요한 범위에서 처리하며, 이는 Cloudflare의 방침을 따릅니다.',
            '프로젝트의 {設定|설정}(`wrangler.jsonc`)은 정적 파일만 제공하고 서버 코드, 분석 도구, 로그 수집을 켜지 않았으며 페이지에도 분석 스크립트가 {無|없}습니다. Cloudflare 관리 화면에서 프로젝트가 볼 수 있는 집계는 이 저장소로 확인할 수 {無|없}습니다.',
            'GitHub, Telegram, 위키로 가는 링크는 각 서비스의 방침을 따릅니다.',
          ],
        ],
      },
      {
        id: 'dict',
        h: '{辭典|사전}의 {出處|출처}',
        body: [
          '{辭典|사전}은 libhangul, FusionIME, English Wiktionary(kaikki.org), JMdict, 標準國語釋讀辭典에서 왔습니다. {出處|출처}와 {許可|허가}는 `dict/README.md`와 페이지 아래의 「{辭典|사전}과 {出處|출처}」에 {有|있}습니다.',
          '{用例|용례} 횟수는 {釋讀會|석독회} Telegram {對話|대화}방(t.me/pimsdh)의 {對話|대화} {記錄|기록}(2026-10-29까지)에서 세었습니다. {入力器|입력기}와 따로 모은 것이며 {對話|대화} 원문은 이 저장소에 들어 있지 않습니다. {入力器|입력기}나 이 페이지에서 {打|치}ㄴ 글은 이 횟수에 들어가지 않습니다.',
          '기계 제안({言語|언어} 모델이 {漢字|한자}를 붙인 {辭典|사전}의 일부)은 English Wiktionary의 뜻풀이와 libhangul의 훈음 풀이에서 만들었습니다. 어떤 낱말을 고를지는 Telegram {對話|대화}에서 그 낱말이 쓰인 횟수로 정했습니다. {入力器|입력기}나 이 페이지에서 {打|치}ㄴ 글은 쓰인 적이 없습니다.',
        ],
      },
      {
        id: 'contact',
        h: '{問議|문의}와 {變更|변경}',
        body: [
          [`전자우편: ${MAIL}`, `GitHub: mkpoli (${ISSUES})`, `Telegram: ${TELEGRAM}`],
          '이 방침은 저장소의 `web/src/lib/pages.js`에 {有|있}으며 {變更|변경} 이력은 GitHub에서 볼 수 {有|있}습니다. 내용을 {變|바꾸}면 위의 날짜도 고칩니다.',
        ],
      },
    ],
  },
  support: {
    eyebrow: '{支援|지원}',
    title: '{使用|사용}법과 {問議|문의}',
    lede: '{釋讀|석독} {入力器|입력기}를 켜는 방법, 자주 겪는 {問題|문제}, 틀린 {變換|변환}을 알리는 방법입니다.',
    sections: [
      {
        id: 'enable',
        h: '켜는 방법',
        body: [
          { h: 'Windows', ol: 'windows' },
          { h: 'Linux (fcitx5)', ol: 'linux' },
          {
            h: 'iOS',
            ol: [
              '앱을 {設置|설치}합니다.',
              '{設定|설정} › 일반 › 키보드 › 키보드 › 새로운 키보드 추가에서 {釋讀|석독}을 {加|더}합니다.',
              '{地球儀|지구본} {鍵|키}를 길게 {押|눌러} {釋讀|석독}을 {選|고르}ㅂ니다. Space로 {變換|변환}하고 {候補|후보}를 {押|눌러} {確定|확정}합니다.',
              '앱의 {設定|설정}을 키보드에 {反映|반영}하려면 이 키보드의 「{全體|전체} {接近|접근} {許用|허용}」을 켭니다. 꺼 {置|두}어도 기본 {設定|설정}으로 동작합니다.',
            ],
          },
          {
            h: 'macOS',
            ol: [
              '앱을 `~/Library/Input Methods`에 {置|넣}고 다시 로그인합니다.',
              '시스템 설정 › 키보드 › 입력 소스 › 편집…에서 +를 {押|눌러} 한국어의 {釋讀|석독}을 {加|더}합니다.',
              '메뉴 막대의 입력 메뉴에서 {釋讀|석독}을 {選|고르}ㅂ니다. {設定|설정}은 같은 메뉴의 「{設定|설정}…」에 {有|있}습니다.',
            ],
          },
          {
            h: 'Android',
            ol: [
              'APK를 {設置|설치}하고 {釋讀|석독} 앱을 엽니다.',
              '「키보드 {設定|설정} 열기」에서 {釋讀|석독}을 켭니다.',
              '「{入力|입력} 방법 고르기」나 Space 키를 길게 {押|눌러} {釋讀|석독}을 {選|고르}ㅂ니다.',
            ],
          },
        ],
      },
      {
        id: 'problems',
        h: '자주 겪는 {問題|문제}',
        body: [
          { h: '{古|예}ㅅ{韓㐎|한글}이 네모로 보입니다', p: '{入力器|입력기}는 글자를 바르게 보내지만, 앱이 자기 글꼴을 쓰고 그 글꼴에 {古|예}ㅅ{韓㐎|한글}이 {無|없}으면 네모로 나옵니다. 앱에서 Noto Serif CJK KR, Noto Sans CJK KR, 함초롬바탕처럼 {古|예}ㅅ{韓㐎|한글}을 {支援|지원}하는 글꼴을 고릅니다. 이 사이트는 자체 글꼴을 불러오므로 여기서는 제대로 보입니다.' },
          { h: 'iPhone에서 기본 키보드로 돌아갑니다', p: 'iOS는 메모리를 많이 쓰는 키보드를 끝냅니다. 이 키보드는 {辭典|사전}을 읽은 뒤 약 66 MB를 쓰며(시뮬레이터에서 잰 값, 2026-11-04), 알려진 한도 추정치보다 큽니다. {地球儀|지구본} {鍵|키}로 {釋讀|석독}을 다시 {選|고르}ㅂ니다.' },
          { h: 'Windows에서 {更新|갱신}한 뒤에도 옛 동작이 남습니다', p: '{更新|갱신}할 때 열려 있던 앱은 옛 버전을 그대로 쥐고 있습니다. 그 앱을 닫았다가 다시 엽니다. 그래도 같으면 로그아웃하거나 Windows를 다시 시작합니다.' },
          { h: 'Linux에서 {釋讀|석독}이 목록에 {無|없}습니다', p: '`fcitx5 -rd`로 fcitx5를 다시 시작하고, fcitx5-configtool에서 「Only Show Current Language」를 해제합니다.' },
          { h: '{設定|설정}이 적용되지 않습니다', p: '다른 {入力器|입력기}로 갔다가 돌아오면 config.toml을 다시 읽습니다. iOS는 앱의 {設定|설정}을 쓰려면 「{全體|전체} {接近|접근} {許用|허용}」이 필요합니다.' },
        ],
      },
      {
        id: 'report',
        h: '틀린 {變換|변환}을 알리는 방법',
        body: [
          '아래 내용을 GitHub 이슈나 Telegram에 적어 주십시오.',
          [
            '{打|치}ㄴ 글을 {韓㐎|한글} 그대로. 짧은 {例|예}면 충분합니다.',
            '나온 {結果|결과}와 바라는 {結果|결과}.',
            '쓰는 {環境|환경}과 버전(Windows, Linux, iOS, macOS, Android).',
            '{表記|표기} {設定|설정}을 {變|바꾸}었다면 그 {內容|내용}(송한자 방식, {國漢文|국한문} {混用|혼용} 등).',
            '개인적인 글은 적지 않습니다. 문제가 되는 낱말만으로 충분합니다.',
          ],
          '기계 제안 {辭典|사전}은 사람이 {檢討|검토}하지 않아 틀릴 수 {有|있}습니다. 바로 고치려면 이 사이트의 「{自作|자작}」에서 {規則|규칙}이나 {個言|낱말}을 더하면 됩니다.',
        ],
      },
      {
        id: 'contact',
        h: '{連絡處|연락처}',
        body: [[`GitHub 이슈: ${ISSUES}`, `Telegram: ${TELEGRAM}`, `전자우편: ${MAIL}`]],
      },
    ],
  },
}

const en = {
  privacy: {
    eyebrow: 'Privacy',
    title: 'Privacy policy',
    updated: '2026-11-04',
    note: 'Written by the 釋讀 input method project, without legal review. Each statement describes the source code as checked on 2026-11-04.',
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
          ],
          'The Windows input method writes a log to `tsf.log` in `%LOCALAPPDATA%\\seokdok` only while a file named `debug` exists in that folder. The log has a timestamp, the program name, key codes and conversion results, so it can contain what you typed. Without the `debug` file nothing is written. The log stays on the device and is never sent; delete `debug` and `tsf.log` to remove it.',
          'The Android app turns off Android’s automatic backup, so its files are not copied to a Google account.',
        ],
      },
      {
        id: 'ios',
        h: 'iOS Full Access',
        body: [
          'An iOS keyboard can read the settings the app saved only when Full Access is on. Reading those settings is the only use this keyboard makes of it.',
          [
            'With Full Access off, the keyboard works completely on its default settings.',
            'Apple’s documentation says a keyboard with Full Access can send keystrokes and other input events over the network. This keyboard has no code that uses the network, so typed text does not leave the device.',
            'In every app, the keyboard handles the letters you type in memory to convert them and does not store them.',
          ],
        ],
      },
      {
        id: 'web',
        h: 'This website (seokdok.mkpo.li)',
        body: [
          [
            'Typed text: text in the input and converter fields on this page is processed in your browser (WebAssembly). The only requests to the server are for the dictionary files (`/dict/native.bin`, `/dict/sino.bin`). The copy buttons put text on your clipboard.',
            'Browser storage: `localStorage` keeps `seokdok:theme`, `seokdok:lang`, `seokdok:options` and `seokdok:custom`: your colour scheme, language, spelling settings, and the rules and words you entered under Customize. They stay in your browser, no cookies are used, and clearing the site’s data removes them.',
            'Fonts: the site’s fonts (Noto Sans KR, Noto Serif KR, Gowun Batang) are served from the site itself, and the page requests nothing from other servers.',
            'Hosting: Cloudflare serves the site. When you open a page, Cloudflare processes your IP address, the requested address and your browser details as needed to operate, as any web host does, under Cloudflare’s own policy.',
            'The project’s configuration (`wrangler.jsonc`) serves static files only, with no server code, analytics or log collection switched on, and the page has no analytics script. What the project could see in the Cloudflare dashboard cannot be checked from the repository.',
            'Links to GitHub, Telegram and the wiki lead to those services, which apply their own policies.',
          ],
        ],
      },
      {
        id: 'dict',
        h: 'Where the dictionaries come from',
        body: [
          'The dictionaries come from libhangul, FusionIME, English Wiktionary (kaikki.org), JMdict and 標準國語釋讀辭典. Sources and licences are in `dict/README.md` and under “Dictionaries and sources” at the foot of the page.',
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
            'Your spelling settings, if you changed them (送韓字 style, Hanja-mixed mode and so on).',
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
