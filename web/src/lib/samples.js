// Sample texts, quoted verbatim from their sources. `text` is what the page
// converts; `source` is shown on the card. Titles use {漢字|한자} marks.

/** @typedef {{ label: string, url: string, rev?: string, licence: string, note?: string }} Source */
/** @typedef {{ id: string, kind: 'classic'|'poem'|'wiki'|'law', title: string, by: string, text: string, source: Source }} Sample */

/** @type {Sample[]} */
export const SAMPLES = [
  {
    id: 'hunmin',
    kind: 'classic',
    title: '{訓民正音|훈민정음} {序文|서문}',
    by: '世宗 · 1446',
    text: '나라의 말이\n중국과는 달라\n한자와는 서로 맞지 아니하므로\n이런 까닭으로 글을 모르는 백성들이 말하고자 하는 바 있어도\n마침내 제 뜻을 능히 펴지 못할 사람이 많으니라\n내 이를 위하여, 가엾게 여겨\n새로 스물여덟 자를 만드노니\n사람마다 쉽게 익혀 날마다 씀에 편안케 하고자 할 따름이니라',
    source: {
      label: '한국어 위키백과 「훈민정음」, 언해본 현대 한국어 풀이',
      url: 'https://ko.wikipedia.org/w/index.php?title=%ED%9B%88%EB%AF%BC%EC%A0%95%EC%9D%8C&oldid=42464837',
      rev: 'oldid 42464837',
      licence: 'CC BY-SA 4.0',
      note: '현대어 풀이의 번역자는 문서에 밝혀져 있지 않음',
    },
  },
  {
    id: 'seosi',
    kind: 'poem',
    title: '{序詩|서시}',
    by: '尹東柱 · 1941',
    text: '죽는 날까지 하늘을 우르러\n한점 부끄럼이 없기를,\n잎새에 이는 바람에도\n나는 괴로워했다.\n별을 노래하는 마음으로\n모든 죽어가는것을 사랑해야지\n그리고 나한테 주어진 길을\n걸어가야겠다.\n\n오늘 밤에도 별이 바람에 스치운다.',
    source: {
      label: '위키문헌 「하늘과 바람과 별과 시 (1948년)/서시」, 초판본(1948)',
      url: 'https://ko.wikisource.org/w/index.php?title=%ED%95%98%EB%8A%98%EA%B3%BC_%EB%B0%94%EB%9E%8C%EA%B3%BC_%EB%B3%84%EA%B3%BC_%EC%8B%9C_(1948%EB%85%84)/%EC%84%9C%EC%8B%9C&oldid=387685',
      rev: 'oldid 387685',
      licence: 'Public domain',
    },
  },
  {
    id: 'jindallae',
    kind: 'poem',
    title: '진달래꽃',
    by: '金素月 · 1925',
    text: '나 보기가 역겨워\n가실 때에는\n말없이 고이 보내드리오리다.\n\n영변(寧邊)에 약산(藥山)\n진달래꽃\n아름따다 가실 길에 뿌리오리다.',
    source: {
      label: '위키문헌 「진달래꽃 (시집)/진달래꽃」 현대어',
      url: 'https://ko.wikisource.org/w/index.php?title=%EC%A7%84%EB%8B%AC%EB%9E%98%EA%BD%83_(%EC%8B%9C%EC%A7%91)/%EC%A7%84%EB%8B%AC%EB%9E%98%EA%BD%83&oldid=430575',
      rev: 'oldid 430575',
      licence: 'Public domain',
      note: '첫 두 연',
    },
  },
  {
    id: 'nim',
    kind: 'poem',
    title: '님의 {沈默|침묵}',
    by: '韓龍雲 · 1926',
    text: '임은 갔습니다. 아아, 사랑하는 나의 임은 갔습니다.\n푸른 산빛을 깨치고 단풍나무 숲을 향하여 난 작은 길을 걸어서 차마 떨치고 갔습니다.\n황금의 꽃같이 굳고 빛나던 옛 맹세는 차디찬 티끌이 되어서 한숨의 미풍에 날아갔습니다.\n날카로운 첫키스의 추억은 나의 운명의 지침을 돌려 놓고 뒷걸음쳐서 사라졌습니다.',
    source: {
      label: '위키문헌 「님의 침묵/님의 침묵」 현대어',
      url: 'https://ko.wikisource.org/w/index.php?title=%EB%8B%98%EC%9D%98_%EC%B9%A8%EB%AC%B5/%EB%8B%98%EC%9D%98_%EC%B9%A8%EB%AC%B5&oldid=430576',
      rev: 'oldid 430576',
      licence: 'Public domain',
      note: '첫 네 줄',
    },
  },
  {
    id: 'hangul',
    kind: 'wiki',
    title: '{韓㐎|한글}',
    by: '{韓國語|한국어} 위키백과',
    text: '한글은 한국어의 공식 표기 문자이며 세종대왕이 한국어를 표기하기 위하여 1443년 창제한 《훈민정음》을 20세기 초반부터 달리 이르는 명칭이다.',
    source: {
      label: '한국어 위키백과 「한글」 첫 문장',
      url: 'https://ko.wikipedia.org/w/index.php?title=%ED%95%9C%EA%B8%80&oldid=42460039',
      rev: 'oldid 42460039',
      licence: 'CC BY-SA 4.0',
    },
  },
  {
    id: 'idu',
    kind: 'wiki',
    title: '{吏讀|이두}',
    by: '{韓國語|한국어} 위키백과',
    text: '이두(吏讀, 문화어: 리두)는 한자에 의한 한국어 표기법의 한 가지이다.',
    source: {
      label: '한국어 위키백과 「이두」 첫 문장',
      url: 'https://ko.wikipedia.org/w/index.php?title=%EC%9D%B4%EB%91%90&oldid=39013882',
      rev: 'oldid 39013882',
      licence: 'CC BY-SA 4.0',
    },
  },
  {
    id: 'hyangchal',
    kind: 'wiki',
    title: '{鄕札|향찰}',
    by: '{韓國語|한국어} 위키백과',
    text: '향찰(鄕札)은 한자를 이용한 한국어 표기법의 하나이다. 주로 신라 향가의 표기에 사용되었으며 고대 한국어인 신라어를 분석하기 위한 중요한 자료가 된다.',
    source: {
      label: '한국어 위키백과 「향찰」 첫 문단',
      url: 'https://ko.wikipedia.org/w/index.php?title=%ED%96%A5%EC%B0%B0&oldid=41488527',
      rev: 'oldid 41488527',
      licence: 'CC BY-SA 4.0',
      note: '괄호 안의 중세 한국어 표기는 생략',
    },
  },
  {
    id: 'gugyeol',
    kind: 'wiki',
    title: '{口訣|구결}',
    by: '{韓國語|한국어} 위키백과',
    text: '구결(口訣)은 한문을 쓸 때 단어나 구절 사이에 붙이는 한국어 토씨 표시용으로 사용되던 문자로, 반대 어순인 한문을 쉽게 읽고자 또는 올바른 풀이, 문법 구조를 나타내고자 각 구절마다 한국어 토를 다는 데 쓰였다.',
    source: {
      label: '한국어 위키백과 「구결」 첫 문장',
      url: 'https://ko.wikipedia.org/w/index.php?title=%EA%B5%AC%EA%B2%B0&oldid=42105874',
      rev: 'oldid 42105874',
      licence: 'CC BY-SA 4.0',
    },
  },
  {
    id: 'honyong',
    kind: 'wiki',
    title: '{國漢文混用|국한문혼용}',
    by: '{韓國語|한국어} 위키백과',
    text: '국한문혼용(國漢文混用) 또는 한자혼용(漢字混用)은 한국어를 문자로 표기할 때 한글과 한자를 섞어서 표기하는 방식을 말한다.',
    source: {
      label: '한국어 위키백과 「한글전용과 국한문혼용」',
      url: 'https://ko.wikipedia.org/w/index.php?title=%ED%95%9C%EA%B8%80%EC%A0%84%EC%9A%A9%EA%B3%BC_%EA%B5%AD%ED%95%9C%EB%AC%B8%ED%98%BC%EC%9A%A9&oldid=40128865',
      rev: 'oldid 40128865',
      licence: 'CC BY-SA 4.0',
    },
  },
  {
    id: 'constitution',
    kind: 'law',
    title: '{大韓民國憲法|대한민국헌법} {前文|전문}',
    by: '1987',
    text: '유구한 역사와 전통에 빛나는 우리 대한국민은 3·1 운동으로 건립된 대한민국 임시 정부의 법통과 불의에 항거한 4·19 민주 이념을 계승하고, 조국의 민주 개혁과 평화적 통일의 사명에 입각하여 정의·인도와 동포애로써 민족의 단결을 공고히 하고',
    source: {
      label: '한국어 위키백과 「한글전용과 국한문혼용」 예시에 실린 헌법 전문',
      url: 'https://ko.wikipedia.org/w/index.php?title=%ED%95%9C%EA%B8%80%EC%A0%84%EC%9A%A9%EA%B3%BC_%EA%B5%AD%ED%95%9C%EB%AC%B8%ED%98%BC%EC%9A%A9&oldid=40128865',
      rev: 'oldid 40128865',
      licence: '저작권법 제7조 (보호받지 못하는 저작물)',
      note: '법제처 원문이 아닌 위키백과의 인용본, 앞부분',
    },
  },
  {
    id: 'udhr',
    kind: 'law',
    title: '{世界人權宣言|세계인권선언} {第一條|제1조}',
    by: '1948',
    text: '모든 인간은 태어날 때부터 자유로우며 그 존엄과 권리에 있어 동등하다. 인간은 천부적으로 이성과 양심을 부여받았으며 서로 형제애의 정신으로 행동하여야 한다.',
    source: {
      label: '위키문헌 「세계인권선언」',
      url: 'https://ko.wikisource.org/w/index.php?title=%EC%84%B8%EA%B3%84%EC%9D%B8%EA%B6%8C%EC%84%A0%EC%96%B8&oldid=382299',
      rev: 'oldid 382299',
      licence: 'PD-UN',
      note: '한국어 번역의 출처는 문서에 밝혀져 있지 않음',
    },
  },
]
