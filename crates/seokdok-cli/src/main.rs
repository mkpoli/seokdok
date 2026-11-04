//! `seokdok`: try the 釋讀 converter from the command line.
//!
//! seokdok [options] convert TEXT…   whole-phrase candidates
//! seokdok [options] segments TEXT…  best segmentation with alternatives
//! seokdok [options] check           load the dictionaries and list errors
//! seokdok [options] forms           dump written forms of 訓讀 entries (TSV)
//! seokdok [options] read TEXT…      read 釋讀 text back into Hangul
//! seokdok [options] add LEMMA WRITTEN POS [CLASS]   add to DIR/user.tsv
//!
//! Options: --dict DIR (default ./dict), --morpheme, --letter, --coda,
//! --resyllabify, --hada, --itda 有|在|在ㅆ, --old-ha (ᄒᆞ first), --sino (音讀 only). Without TEXT, lines are read from stdin.

use seokdok_core::{Engine, Fragment, Itda, Options, Style};
use std::io::BufRead;
use std::path::PathBuf;

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut dir = PathBuf::from("dict");
    let mut o = Options::default();
    let mut rest = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dict" => {
                dir = PathBuf::from(args.get(i + 1).expect("--dict DIR"));
                i += 1;
            }
            "--morpheme" => o.style = Style::Morpheme,
            "--letter" => o.fragment = Fragment::Letter,
            "--coda" => o.coda_always = true,
            "--resyllabify" => o.resyllabify = true,
            "--hada" => o.hada_kun = true,
            "--itda" => {
                o.itda = match args.get(i + 1).map(String::as_str) {
                    Some("有") => Itda::Yu,
                    Some("在") => Itda::Jae,
                    Some("在ㅆ") => Itda::JaeCoda,
                    _ => panic!("--itda 有|在|在ㅆ"),
                };
                i += 1;
            }
            "--old-ha" => o.old_ha = true,
            "--sino" => o.sino_only = true,
            _ => rest.push(std::mem::take(&mut args[i])),
        }
        i += 1;
    }
    let cmd = if rest.is_empty() { "convert".to_string() } else { rest.remove(0) };
    let t = std::time::Instant::now();
    let (eng, errors) = Engine::from_dir(&dir, o).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
    let texts: Vec<String> = if rest.is_empty() && !matches!(cmd.as_str(), "check" | "forms" | "attest" | "add") {
        std::io::stdin().lock().lines().map_while(Result::ok).collect()
    } else {
        vec![rest.join(" ")]
    };
    match cmd.as_str() {
        "check" => {
            for e in &errors {
                println!("{e}");
            }
            eprintln!(
                "{} native, {} sino entries, {} errors, loaded in {:?}",
                eng.natives.len(),
                eng.sino_count(),
                errors.len(),
                t.elapsed()
            );
        }
        "convert" => {
            for line in texts {
                for (k, c) in eng.convert(&line).iter().enumerate() {
                    println!("{k:>2}. {}\t{}", c.text, c.note);
                }
            }
        }
        "segments" => {
            for line in texts {
                for s in eng.segments(&line) {
                    let alts: Vec<&str> = s.candidates.iter().map(|c| c.text.as_str()).collect();
                    println!("[{}] {}", s.reading, alts.join(" | "));
                }
            }
        }
        "attest" => attest(&eng, &rest),
        "read" => {
            for line in texts {
                println!("{}", eng.read(&line));
            }
        }
        "add" => {
            // seokdok add LEMMA WRITTEN POS [CLASS]: append to DIR/user.tsv
            let get = |i: usize| rest.get(i).map(String::as_str).unwrap_or("");
            let line = format!("{}\t{}\t{}\t{}\tuser\t\t", get(0), get(1), get(2), get(3));
            let fields: Vec<&str> = line.split('\t').collect();
            if let Err(e) = seokdok_core::dict::parse_native(&fields) {
                eprintln!("not added: {e}");
                std::process::exit(1);
            }
            let p = dir.join("user.tsv");
            let mut text = std::fs::read_to_string(&p).unwrap_or_else(|_| "lemma\twritten\tpos\tclass\tsource\tcount\tnote\n".into());
            text.push_str(&line);
            text.push('\n');
            std::fs::write(&p, text).expect("write user.tsv");
            println!("added to {}: {} = {}", p.display(), get(0), get(1));
        }
        "forms" => {
            for (written, surface, lemma) in eng.written_forms() {
                println!("{written}\t{surface}\t{lemma}");
            }
        }
        other => {
            eprintln!("unknown command {other}");
            std::process::exit(2);
        }
    }
}

fn is_hanja(c: char) -> bool {
    matches!(c as u32, 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF | 0x20000..=0x3134F)
}
fn is_hangul(c: char) -> bool {
    matches!(c as u32, 0xAC00..=0xD7A3 | 0x3131..=0x318E)
}

/// Tokens of the form Hanja+ (Hangul|jamo)*, as 釋讀 words appear in text.
fn tokens(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_tail = false;
    for c in text.chars() {
        if is_hanja(c) {
            if in_tail {
                out.push(std::mem::take(&mut cur));
                in_tail = false;
            }
            cur.push(c);
        } else if is_hangul(c) && !cur.is_empty() {
            cur.push(c);
            in_tail = true;
        } else {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            in_tail = false;
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// seokdok attest CORPUS.txt OUT.tsv UNEXPLAINED.tsv
/// Count corpus tokens matching each 訓讀 entry's written forms.
fn attest(eng: &Engine, args: &[String]) {
    use std::collections::{HashMap, HashSet};
    let corpus = std::fs::read_to_string(&args[0]).expect("corpus");
    let mut counts: HashMap<String, u32> = HashMap::new();
    for line in corpus.lines() {
        for t in tokens(&seokdok_core::dict::unify(line)) {
            *counts.entry(t).or_default() += 1;
        }
    }
    // Written form → entries producing it; credit goes to the most trusted.
    let mut producers: HashMap<String, Vec<usize>> = HashMap::new();
    for i in 0..eng.natives.len() {
        let mut seen = HashSet::new();
        // A bare Hanja token is a Sino-Korean word, never a verb stem (右 in 左右).
        let inflects = eng.natives[i].pos.inflects();
        for (w, _) in eng.forms_of(i) {
            if inflects && !w.chars().any(is_hangul) {
                continue;
            }
            if counts.contains_key(&w) && seen.insert(w.clone()) {
                producers.entry(w).or_default().push(i);
            }
        }
    }
    let mut credit = vec![0u32; eng.natives.len()];
    let mut explained: HashSet<String> = HashSet::new();
    for (w, ps) in &producers {
        let best = ps.iter().map(|&i| eng.natives[i].source.rank()).min().unwrap();
        // A noun with a particle (右와 = 오른쪽 + 와) beats a verb form that
        // happens to be spelled the same.
        let noun = ps.iter().any(|&i| eng.natives[i].source.rank() == best && !eng.natives[i].pos.inflects());
        for &i in ps {
            if eng.natives[i].source.rank() == best && !(noun && eng.natives[i].pos.inflects()) {
                credit[i] += counts[w];
            }
        }
        explained.insert(w.clone());
    }
    let mut out = String::from("file\tline\tlemma\twritten\tcount\n");
    for (i, n) in eng.natives.iter().enumerate() {
        if credit[i] > 0 {
            out.push_str(&format!("{}\t{}\t{}\t{}\t{}\n", n.origin.0, n.origin.1, n.lemma, n.hanja, credit[i]));
        }
    }
    std::fs::write(&args[1], out).expect("write");
    // Tokens no entry explains, excluding pure Hanja runs (Sino words).
    let mut un: Vec<(&String, &u32)> = counts
        .iter()
        .filter(|(t, _)| !explained.contains(*t) && t.chars().any(is_hangul))
        .collect();
    un.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    let mut s = String::from("token\tcount\n");
    for (t, c) in un.iter().take(3000) {
        s.push_str(&format!("{t}\t{c}\n"));
    }
    std::fs::write(&args[2], s).expect("write");
}
