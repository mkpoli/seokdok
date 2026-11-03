"""Extract a Telegram Desktop HTML export into JSONL (one message per line).

Usage: uv run tools/extract_telegram.py <export-dir> <out.jsonl>

The output holds chat text only (id, date, sender, text, reply_to). It stays
under data/private/, which git ignores: the chat belongs to its members.
"""
import html
import json
import re
import sys
from html.parser import HTMLParser
from pathlib import Path


class Parser(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.msgs = []
        self.cur = None
        self.capture = None  # 'text' | 'from'
        self.depth = 0
        self.cap_depth = 0
        self.last_sender = None

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        cls = a.get("class", "") or ""
        if tag == "div":
            self.depth += 1
        if tag == "div" and cls.startswith("message default"):
            self.cur = {"id": int(a["id"].removeprefix("message")) if a.get("id", "").startswith("message") and a["id"][7:].isdigit() else None,
                        "date": None, "from": None, "text": "", "reply_to": None}
            self.msgs.append(self.cur)
            return
        if self.cur is None:
            return
        if tag == "div" and "date details" in cls and a.get("title"):
            self.cur["date"] = a["title"]
        elif tag == "div" and cls == "from_name" and self.capture is None:
            self.capture, self.cap_depth, self.cur["from"] = "from", self.depth, ""
        elif tag == "div" and cls == "text" and self.capture is None:
            self.capture, self.cap_depth = "text", self.depth
        elif tag == "a" and self.cur and a.get("href", "").startswith("#go_to_message") and self.capture is None:
            self.cur["reply_to"] = int(re.sub(r"\D", "", a["href"]) or 0)
        elif tag == "br" and self.capture == "text":
            self.cur["text"] += "\n"

    def handle_endtag(self, tag):
        if tag == "div":
            if self.capture and self.depth == self.cap_depth:
                self.capture = None
            self.depth -= 1

    def handle_data(self, data):
        if self.capture == "text":
            self.cur["text"] += data
        elif self.capture == "from":
            self.cur["from"] += data


def main():
    src, out = Path(sys.argv[1]), Path(sys.argv[2])
    files = sorted(src.glob("messages*.html"), key=lambda p: int(re.sub(r"\D", "", p.stem) or 1))
    n = 0
    last = None
    with out.open("w", encoding="utf-8") as f:
        for p in files:
            ps = Parser()
            ps.feed(p.read_text(encoding="utf-8"))
            for m in ps.msgs:
                m["text"] = m["text"].strip()
                if m["from"] is not None:
                    m["from"] = m["from"].strip()
                    last = m["from"]
                else:
                    m["from"] = last  # "joined" messages continue the previous sender
                if m["text"]:
                    f.write(json.dumps(m, ensure_ascii=False) + "\n")
                    n += 1
    print(f"{n} messages -> {out}", file=sys.stderr)


if __name__ == "__main__":
    main()
