//! What a person keeps picking: a small table of (reading, written form) that
//! puts their choice first the next time. See docs/learning.md.
//!
//! [`Learned`] is the table and its rules, free of files and threads, so the
//! site's WebAssembly build uses it as it stands (with `localStorage`). The
//! native front ends share a [`Store`], which keeps it in `learned.tsv` beside
//! config.toml. The file holds only words picked and logical counters; it has
//! no clock times and nothing here reaches a network.

use std::collections::HashMap;
use std::io::Write;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use seokdok_core::jamo::is_hangul;
use seokdok_core::Candidate;

use crate::config::{stamp_of, Config, FileStamp};

/// Picks of one form before it comes first. One pick can be a slip of the
/// finger, and a slip must change nothing; two picks with no use of the usual
/// form between them (see [`Learned::commit`]) mean the person wants it, and a
/// third would only make them correct the same word again.
pub const THRESHOLD: u32 = 2;
/// Forms kept in all; the oldest goes first.
pub const MAX_ENTRIES: usize = 4096;
/// The file beside config.toml.
pub const FILE_NAME: &str = "learned.tsv";

/// Longer spans are phrases or sentences, which this table does not keep.
/// Counted in `char`s: an old-Hangul syllable takes three.
const MAX_READING_CHARS: usize = 32;
const MAX_TEXT_CHARS: usize = 48;
/// Adjacent segments that a learned reading may join.
const MAX_JOINED_SEGMENTS: usize = 4;
const HEADER: &str = "# seokdok learned picks 1";
/// Every entry at its longest fits well below this.
const MAX_FILE_BYTES: u64 = 2 << 20;
/// How long a save waits for further picks; a crash can lose this much.
const SAVE_DELAY: Duration = Duration::from_millis(1000);
/// Picks held for a file that cannot be written; the table in memory has them all.
const MAX_PENDING: usize = 1024;

/// A form of a reading. `last` and `prev` are the table's logical clock at the
/// last pick and the one before it (0 for none).
#[derive(Clone, Debug, PartialEq, Eq)]
struct Form {
    text: Box<str>,
    count: u32,
    prev: u64,
    last: u64,
}

/// The table: readings to the forms picked for them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Learned {
    readings: HashMap<Box<str>, Vec<Form>>,
    len: usize,
    clock: u64,
}

/// What may be kept at all: a short run of Hangul, never a phrase or a number.
fn learnable(reading: &str, text: &str) -> bool {
    let clean = |s: &str, max: usize| !s.is_empty() && s.chars().count() <= max && !s.chars().any(|c| c.is_whitespace() || c.is_control());
    clean(reading, MAX_READING_CHARS) && clean(text, MAX_TEXT_CHARS) && reading.chars().any(is_hangul)
}

impl Learned {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The number of (reading, form) entries.
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// `text` was committed for `reading`; `first` says it was the first
    /// candidate shown. True when the table changed.
    ///
    /// A form other than the first is counted. Taking the first one counts it
    /// only if it has an entry already, which keeps a form in use from being
    /// evicted, and it drops rivals picked just once: the person had the
    /// choice and took the usual form, so an old slip and a new one do not add
    /// up to two picks. A reading with no entries stores nothing.
    pub fn commit(&mut self, reading: &str, text: &str, first: bool) -> bool {
        if !learnable(reading, text) {
            return false;
        }
        if first && !self.readings.contains_key(reading) {
            return false;
        }
        let tick = self.clock + 1;
        let forms = self.readings.entry(reading.into()).or_default();
        let before = forms.len();
        if first {
            forms.retain(|f| f.count >= THRESHOLD || &*f.text == text);
        }
        let mut changed = forms.len() != before;
        self.len -= before - forms.len();
        match forms.iter_mut().find(|f| &*f.text == text) {
            Some(f) => {
                f.count = f.count.saturating_add(1);
                f.prev = f.last;
                f.last = tick;
                changed = true;
            }
            None if first => {}
            None => {
                forms.push(Form { text: text.into(), count: 1, prev: 0, last: tick });
                self.len += 1;
                changed = true;
            }
        }
        if forms.is_empty() {
            self.readings.remove(reading);
        }
        if changed {
            self.clock = tick;
        }
        self.evict();
        changed
    }

    /// Whether some form of exactly this reading has been picked enough.
    pub fn promotes(&self, reading: &str) -> bool {
        self.readings.get(reading).is_some_and(|forms| forms.iter().any(|f| f.count >= THRESHOLD))
    }

    /// Where among `texts` the learned form of exactly this reading is. Of
    /// several forms picked enough, the one that reached its second pick last
    /// wins, so two recent picks outweigh any number of old ones and one
    /// recent pick changes nothing. A reading is matched whole: picks for 나
    /// say nothing about 나라.
    pub fn learned_at<'a>(&self, reading: &str, texts: impl Iterator<Item = &'a str> + Clone) -> Option<usize> {
        self.readings
            .get(reading)?
            .iter()
            .filter(|f| f.count >= THRESHOLD)
            .filter_map(|f| texts.clone().position(|t| t == &*f.text).map(|at| (f.prev, at)))
            .max_by_key(|&(prev, _)| prev)
            .map(|(_, at)| at)
    }

    /// Move the learned form of this reading to the front, if it is among
    /// `cands`, keeping the others in order. True when a learned form now leads.
    pub fn rank(&self, reading: &str, cands: &mut [Candidate]) -> bool {
        match self.learned_at(reading, cands.iter().map(|c| c.text.as_str())) {
            Some(at) => {
                cands[..=at].rotate_right(1);
                true
            }
            None => false,
        }
    }

    /// The runs of two or more adjacent segments whose joined reading was
    /// learned as one span (the person resized a segment and picked for it).
    /// Only whole segments join, longest first from the left, so a word inside
    /// a segment is never cut.
    pub fn joins(&self, readings: &[&str]) -> Vec<Range<usize>> {
        let mut out = Vec::new();
        let mut i = 0;
        while i < readings.len() {
            let longest = (i + MAX_JOINED_SEGMENTS).min(readings.len());
            let hit = (i + 2..=longest).rev().find(|&end| self.promotes(&readings[i..end].concat()));
            match hit {
                Some(end) => {
                    out.push(i..end);
                    i = end;
                }
                None => i += 1,
            }
        }
        out
    }

    /// The table as text, oldest first: `reading\ttext\tcount\tprev\tlast`.
    pub fn to_text(&self) -> String {
        let mut rows: Vec<(&str, &Form)> = self.readings.iter().flat_map(|(r, forms)| forms.iter().map(move |f| (&**r, f))).collect();
        rows.sort_by_key(|(_, f)| f.last);
        let mut text = format!("{HEADER}\n");
        for (r, f) in rows {
            text.push_str(&format!("{r}\t{}\t{}\t{}\t{}\n", f.text, f.count, f.prev, f.last));
        }
        text
    }

    /// Read [`Learned::to_text`]. `None` for anything else: a file that is not
    /// exactly this is ignored whole, and replaced by the next save. An
    /// unfinished or edited file must not reorder anyone's candidates.
    pub fn parse(text: &str) -> Option<Self> {
        let mut lines = text.lines();
        if lines.next()? != HEADER {
            return None;
        }
        let mut table = Self::new();
        for line in lines {
            let mut cols = line.split('\t');
            let (reading, form) = (cols.next()?, cols.next()?);
            let (count, prev, last) = (cols.next()?.parse().ok()?, cols.next()?.parse().ok()?, cols.next()?.parse().ok()?);
            let order_holds = last > prev && (count >= 2) == (prev > 0);
            if cols.next().is_some() || !learnable(reading, form) || count == 0 || !order_holds {
                return None;
            }
            let forms = table.readings.entry(reading.into()).or_default();
            if forms.iter().any(|f: &Form| &*f.text == form) {
                return None;
            }
            forms.push(Form { text: form.into(), count, prev, last });
            table.len += 1;
            table.clock = table.clock.max(last);
        }
        table.evict();
        Some(table)
    }

    /// Bring the table within [`MAX_ENTRIES`], oldest pick first.
    fn evict(&mut self) {
        while self.len > MAX_ENTRIES {
            let oldest = self
                .readings
                .iter()
                .flat_map(|(r, forms)| forms.iter().map(move |f| (f.last, r, &f.text)))
                .min_by_key(|&(last, ..)| last)
                .map(|(_, r, text)| (r.clone(), text.clone()));
            let Some((reading, text)) = oldest else { return };
            if let Some(forms) = self.readings.get_mut(&*reading) {
                forms.retain(|f| f.text != text);
                if forms.is_empty() {
                    self.readings.remove(&*reading);
                }
            }
            self.len -= 1;
        }
    }
}

/// A commit to apply to the file's table when saving.
#[derive(Clone, Debug)]
struct Pick {
    reading: String,
    text: String,
    first: bool,
}

#[derive(Default)]
struct State {
    table: Learned,
    /// Picks not yet in the file.
    pending: Vec<Pick>,
    /// The version of the file `table` follows.
    stamp: Option<FileStamp>,
    loading: bool,
    saving: bool,
    /// Bumped by [`Store::forget`], so work begun before it does not write.
    epoch: u64,
}

/// Threads of every store that are running; see [`idle`].
static ACTIVE: AtomicUsize = AtomicUsize::new(0);

/// True when no store has a thread running: a DLL may then be unloaded.
pub fn idle() -> bool {
    ACTIVE.load(Ordering::SeqCst) == 0
}

/// The table of one `learned.tsv`, shared by the sessions of a process. Calls
/// made while typing take a short lock on memory only: the file is read and
/// written on short-lived background threads.
///
/// Other processes (each application hosts its own input method on Windows)
/// write the same file. A save reads the file and applies this process's picks
/// on top; [`Store::refresh`] follows what they wrote.
pub struct Store {
    path: Option<PathBuf>,
    state: Mutex<State>,
}

impl std::fmt::Debug for Store {
    // The words stay out of debug logs.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store").field("path", &self.path).finish_non_exhaustive()
    }
}

impl Store {
    /// A store over this file, starting empty until a background read
    /// finishes. With no path it lives in memory only.
    pub fn open(path: Option<PathBuf>) -> Arc<Self> {
        let store = Arc::new(Store { path, state: Mutex::new(State::default()) });
        store.start_load();
        store
    }

    /// The table in memory, whoever panicked while holding it.
    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn is_empty(&self) -> bool {
        self.state().table.is_empty()
    }

    pub fn len(&self) -> usize {
        self.state().table.len()
    }

    pub fn promotes(&self, reading: &str) -> bool {
        self.state().table.promotes(reading)
    }

    pub fn rank(&self, reading: &str, cands: &mut [Candidate]) -> bool {
        self.state().table.rank(reading, cands)
    }

    pub fn joins(&self, readings: &[&str]) -> Vec<Range<usize>> {
        self.state().table.joins(readings)
    }

    /// See [`Learned::commit`]; the file follows shortly.
    pub fn commit(self: &Arc<Self>, reading: &str, text: &str, first: bool) {
        let mut st = self.state();
        if !st.table.commit(reading, text, first) || self.path.is_none() {
            return;
        }
        st.pending.push(Pick { reading: reading.into(), text: text.into(), first });
        if !st.saving {
            st.saving = true;
            if !self.spawn(Self::save_after_delay) {
                st.saving = false;
            }
        }
    }

    /// Follow a write from another process or a settings window. This only
    /// reads the file's timestamp and length, so it can run at every
    /// conversion; a changed file is read in the background.
    pub fn refresh(self: &Arc<Self>) {
        let Some(path) = &self.path else { return };
        let stamp = stamp_of(path);
        let mut st = self.state();
        if st.stamp == stamp || st.loading || st.saving {
            return;
        }
        if stamp.is_none() {
            // Forgotten elsewhere: this needs no reading, and must not wait.
            st.table.clear();
            st.stamp = None;
            return;
        }
        drop(st);
        self.start_load();
    }

    /// Empty the table and delete the file, also a temporary one left by a
    /// crash. Work already running writes nothing afterwards.
    pub fn forget(&self) -> std::io::Result<()> {
        let mut st = self.state();
        st.table.clear();
        st.pending.clear();
        st.stamp = None;
        st.epoch += 1;
        match &self.path {
            Some(path) => remove_files(path),
            None => Ok(()),
        }
    }

    /// Write now what is waiting, on this thread. For tests and tools: typing
    /// never comes here.
    pub fn save_now(&self) -> std::io::Result<()> {
        self.save_pass().map(|_| ())
    }

    /// Wait for the background read. For tests and tools.
    pub fn wait_loaded(&self) {
        while self.state().loading {
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn start_load(self: &Arc<Self>) {
        if self.path.is_none() {
            return;
        }
        let mut st = self.state();
        if !st.loading {
            st.loading = true;
            if !self.spawn(Self::load) {
                st.loading = false;
            }
        }
    }

    /// Run `work` on a thread of its own; false if none could be made.
    fn spawn(self: &Arc<Self>, work: fn(&Self)) -> bool {
        let store = Arc::clone(self);
        ACTIVE.fetch_add(1, Ordering::SeqCst);
        let made = std::thread::Builder::new().name("seokdok-learned".into()).spawn(move || {
            work(&store);
            ACTIVE.fetch_sub(1, Ordering::SeqCst);
        });
        if made.is_err() {
            ACTIVE.fetch_sub(1, Ordering::SeqCst);
        }
        made.is_ok()
    }

    fn load(&self) {
        let Some(path) = &self.path else { return };
        let epoch = self.state().epoch;
        // The stamp comes first: a write between the two is seen as a change.
        let stamp = stamp_of(path);
        let file = read_table(path);
        let mut st = self.state();
        st.loading = false;
        if st.epoch != epoch {
            return;
        }
        let mut table = file;
        for p in &st.pending {
            table.commit(&p.reading, &p.text, p.first);
        }
        st.table = table;
        st.stamp = stamp;
    }

    fn save_after_delay(&self) {
        loop {
            std::thread::sleep(SAVE_DELAY);
            if !matches!(self.save_pass(), Ok(true)) {
                return;
            }
        }
    }

    /// Put the waiting picks in the file. `Ok(true)` when more arrived
    /// meanwhile. A file that cannot be written keeps them waiting and the
    /// table in memory goes on serving: the person sees no error, and what was
    /// learned lasts as long as the process.
    fn save_pass(&self) -> std::io::Result<bool> {
        let Some(path) = &self.path else { return Ok(false) };
        let (batch, epoch) = {
            let mut st = self.state();
            (std::mem::take(&mut st.pending), st.epoch)
        };
        if batch.is_empty() {
            let mut st = self.state();
            st.saving = st.saving && !st.pending.is_empty();
            return Ok(st.saving);
        }
        let mut merged = read_table(path);
        for p in &batch {
            merged.commit(&p.reading, &p.text, p.first);
        }
        let written = write_temp(path, &merged.to_text());
        let mut st = self.state();
        let keep_waiting = |st: &mut State| {
            let newer = std::mem::replace(&mut st.pending, batch.clone());
            st.pending.extend(newer);
            let excess = st.pending.len().saturating_sub(MAX_PENDING);
            st.pending.drain(..excess);
            st.saving = false;
        };
        let tmp = match written {
            Ok(tmp) => tmp,
            Err(e) => {
                keep_waiting(&mut st);
                return Err(e);
            }
        };
        if st.epoch != epoch {
            // Forgotten meanwhile. What arrived since is the new table's.
            let _ = std::fs::remove_file(&tmp);
            st.saving = st.saving && !st.pending.is_empty();
            return Ok(st.saving);
        }
        // Under the lock, so `forget` cannot delete between this and a write.
        if let Err(e) = std::fs::rename(&tmp, path) {
            let _ = std::fs::remove_file(&tmp);
            keep_waiting(&mut st);
            return Err(e);
        }
        st.stamp = stamp_of(path);
        for p in &st.pending {
            merged.commit(&p.reading, &p.text, p.first);
        }
        st.table = merged;
        st.saving = st.saving && !st.pending.is_empty();
        Ok(st.saving)
    }
}

/// The file's table; a missing, oversized or malformed file is an empty one.
fn read_table(path: &Path) -> Learned {
    let fits = std::fs::metadata(path).is_ok_and(|m| m.len() <= MAX_FILE_BYTES);
    fits.then(|| std::fs::read_to_string(path).ok()).flatten().and_then(|t| Learned::parse(&t)).unwrap_or_default()
}

/// Write a whole temporary file, to disk, readable only by its owner. Renaming
/// it over the old one means a crash leaves one version or the other.
fn write_temp(path: &Path, text: &str) -> std::io::Result<PathBuf> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = temp_path(path);
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let written = options.open(&tmp).and_then(|mut f| {
        f.write_all(text.as_bytes())?;
        f.sync_all()
    });
    match written {
        Ok(()) => Ok(tmp),
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            Err(e)
        }
    }
}

/// Each process has its own, so two writers cannot mix their text.
fn temp_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".{}.tmp", std::process::id()));
    path.with_file_name(name)
}

/// Delete the file and any temporary one beside it.
fn remove_files(path: &Path) -> std::io::Result<()> {
    if let (Some(dir), Some(name)) = (path.parent(), path.file_name().and_then(|n| n.to_str())) {
        let prefix = format!("{name}.");
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let file = entry.file_name();
            if file.to_str().is_some_and(|f| f.starts_with(&prefix) && f.ends_with(".tmp")) {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
    match std::fs::remove_file(path) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

/// The store this process shares.
static SHARED: Mutex<Option<Arc<Store>>> = Mutex::new(None);

fn shared_lock() -> MutexGuard<'static, Option<Arc<Store>>> {
    SHARED.lock().unwrap_or_else(|e| e.into_inner())
}

/// The store a session should use under `input.learn`, beside the current
/// config.toml. Off frees the table; the file stays until it is forgotten.
pub fn shared(learn: bool) -> Option<Arc<Store>> {
    let mut slot = shared_lock();
    if !learn {
        *slot = None;
        return None;
    }
    let path = Config::user_dir().map(|d| d.join(FILE_NAME));
    match slot.as_ref() {
        Some(s) if s.path == path => Some(Arc::clone(s)),
        _ => {
            let store = Store::open(path);
            *slot = Some(Arc::clone(&store));
            Some(store)
        }
    }
}

/// "Forget what I picked" for the folder holding config.toml: the file goes,
/// and so does this process's table if it is that file's. Other processes
/// notice the file is gone at their next conversion.
pub fn forget(dir: &Path) -> std::io::Result<()> {
    let path = dir.join(FILE_NAME);
    let store = shared_lock().clone().filter(|s| s.path.as_deref() == Some(&path));
    match store {
        Some(s) => s.forget(),
        None => remove_files(&path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cands(list: &[&str]) -> Vec<Candidate> {
        list.iter().map(|t| Candidate { text: t.to_string(), note: String::new() }).collect()
    }

    fn first_of(table: &Learned, reading: &str, list: &[&str]) -> String {
        let mut c = cands(list);
        table.rank(reading, &mut c);
        c[0].text.clone()
    }

    const NA: [&str; 3] = ["吾", "出", "我"];

    #[test]
    fn one_pick_changes_nothing_and_two_come_first() {
        let mut t = Learned::new();
        assert!(t.commit("나", "我", false));
        assert_eq!(first_of(&t, "나", &NA), "吾");
        assert!(!t.promotes("나"));
        assert!(t.commit("나", "我", false));
        assert_eq!(first_of(&t, "나", &NA), "我");
        assert!(t.promotes("나"));
    }

    #[test]
    fn the_rest_of_the_list_keeps_its_order() {
        let mut t = Learned::new();
        t.commit("나", "我", false);
        t.commit("나", "我", false);
        let mut c = cands(&["吾", "出", "我", "經"]);
        assert!(t.rank("나", &mut c));
        assert_eq!(c.iter().map(|k| k.text.as_str()).collect::<Vec<_>>(), ["我", "吾", "出", "經"]);
    }

    #[test]
    fn a_reading_never_picked_or_a_longer_word_is_left_alone() {
        let mut t = Learned::new();
        t.commit("나", "我", false);
        t.commit("나", "我", false);
        assert_eq!(first_of(&t, "나라", &["國", "我라"]), "國");
        assert_eq!(first_of(&t, "너", &["汝", "我"]), "汝");
        // A form the list no longer offers: another script, say.
        let mut c = cands(&["吾", "出"]);
        assert!(!t.rank("나", &mut c));
        assert_eq!(c[0].text, "吾");
    }

    #[test]
    fn a_recent_form_outweighs_an_old_one() {
        let mut t = Learned::new();
        for _ in 0..10 {
            t.commit("나", "我", false);
        }
        assert_eq!(first_of(&t, "나", &NA), "我");
        // One pick of another form changes nothing, however few 我 had.
        t.commit("나", "吾", false);
        assert_eq!(first_of(&t, "나", &NA), "我");
        t.commit("나", "吾", false);
        assert_eq!(first_of(&t, "나", &NA), "吾");
        t.commit("나", "我", false);
        assert_eq!(first_of(&t, "나", &NA), "吾");
        t.commit("나", "我", false);
        assert_eq!(first_of(&t, "나", &NA), "我");
    }

    #[test]
    fn a_mistaken_pick_while_a_form_leads_does_not_unseat_it() {
        let mut t = Learned::new();
        t.commit("나", "我", false);
        t.commit("나", "我", false);
        t.commit("나", "出", false);
        assert_eq!(first_of(&t, "나", &NA), "我");
    }

    #[test]
    fn taking_the_usual_form_clears_an_earlier_slip() {
        let mut t = Learned::new();
        t.commit("나", "我", false);
        assert!(t.commit("나", "吾", true));
        assert!(t.is_empty());
        t.commit("나", "我", false);
        assert_eq!(first_of(&t, "나", &NA), "吾");
    }

    #[test]
    fn taking_the_first_form_of_an_unknown_reading_stores_nothing() {
        let mut t = Learned::new();
        assert!(!t.commit("나", "吾", true));
        assert!(t.is_empty());
    }

    #[test]
    fn taking_a_learned_form_keeps_it_from_eviction() {
        let mut t = Learned::new();
        t.commit("나", "我", false);
        t.commit("나", "我", false);
        for i in 0..MAX_ENTRIES - 1 {
            t.commit(&format!("가{i}"), "x", false);
        }
        assert!(t.commit("나", "我", true));
        t.commit("다", "x", false);
        assert!(t.promotes("나"));
        assert_eq!(t.len(), MAX_ENTRIES);
    }

    #[test]
    fn the_oldest_entry_is_evicted_at_the_bound() {
        let mut t = Learned::new();
        t.commit("옛", "昔", false);
        t.commit("옛", "昔", false);
        t.commit("둘", "二", false);
        for i in 0..MAX_ENTRIES - 2 {
            t.commit(&format!("가{i}"), "x", false);
        }
        assert_eq!(t.len(), MAX_ENTRIES);
        assert!(t.promotes("옛"));
        t.commit("새", "新", false);
        assert_eq!(t.len(), MAX_ENTRIES);
        assert!(!t.promotes("옛"));
        assert!(t.readings.contains_key("둘"));
        assert!(t.readings.contains_key("새"));
    }

    #[test]
    fn phrases_numbers_and_field_breakers_are_not_kept() {
        let mut t = Learned::new();
        let long = "가".repeat(MAX_READING_CHARS + 1);
        for (r, x) in [("1234", "x"), ("나 는", "我 는"), (long.as_str(), "x"), ("나", "a\tb"), ("나", "a\nb"), ("", "x"), ("나", "")] {
            assert!(!t.commit(r, x, false), "{r:?} {x:?}");
        }
        assert!(t.is_empty());
    }

    #[test]
    fn joined_segments_are_found_whole() {
        let mut t = Learned::new();
        t.commit("한가", "閑暇", false);
        assert!(t.joins(&["한", "가"]).is_empty());
        t.commit("한가", "閑暇", false);
        assert_eq!(t.joins(&["한", "가", " ", "나"]), [0..2]);
        assert_eq!(t.joins(&["나", "한", "가"]), [1..3]);
        // Inside a longer segment, or across a boundary that is not there.
        assert!(t.joins(&["한가위"]).is_empty());
        assert!(t.joins(&["대한", "가"]).is_empty());
    }

    #[test]
    fn the_text_round_trips() {
        let mut t = Learned::new();
        t.commit("나", "我", false);
        t.commit("나", "我", false);
        t.commit("나", "吾", false);
        t.commit("ᄒᆞᆯ까요", "爲ᄒᆞᆯ까요", false);
        let back = Learned::parse(&t.to_text()).unwrap();
        assert_eq!(back, t);
        assert_eq!(back.len(), 3);
        // The clock carries on after what was read.
        let mut after = back.clone();
        after.commit("나", "吾", false);
        assert_eq!(first_of(&after, "나", &NA), "吾");
    }

    #[test]
    fn a_corrupted_file_is_ignored_whole() {
        let good = format!("{HEADER}\n나\t我\t2\t1\t2\n");
        assert!(Learned::parse(&good).unwrap().promotes("나"));
        for bad in [
            "".to_string(),
            "나\t我\t2\t1\t2\n".to_string(),
            "# seokdok learned picks 2\n나\t我\t2\t1\t2\n".to_string(),
            format!("{good}나\t我"),
            format!("{good}garbage\n"),
            format!("{good}다\tx\ttwo\t1\t2\n"),
            format!("{good}다\tx\t0\t0\t3\n"),
            format!("{good}다\tx\t2\t0\t3\n"),
            format!("{good}다\tx\t1\t3\t3\n"),
            format!("{good}다\tx\t1\t0\t3\t9\n"),
            format!("{good}나\t我\t1\t0\t3\n"),
            format!("{HEADER}\n1234\tx\t1\t0\t3\n"),
        ] {
            assert!(Learned::parse(&bad).is_none(), "{bad:?}");
        }
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("seokdok-learned-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A store whose first read has finished.
    fn opened(path: &Path) -> Arc<Store> {
        let s = Store::open(Some(path.to_path_buf()));
        s.wait_loaded();
        s
    }

    #[test]
    fn a_store_survives_a_restart_and_a_corrupted_file_is_replaced() {
        let dir = scratch("restart");
        let path = dir.join(FILE_NAME);
        let s = opened(&path);
        s.commit("나", "我", false);
        s.commit("나", "我", false);
        s.save_now().unwrap();
        let again = opened(&path);
        assert!(again.promotes("나"));

        std::fs::write(&path, "not a table").unwrap();
        let broken = opened(&path);
        assert!(broken.is_empty());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "not a table");
        broken.commit("너", "汝", false);
        broken.save_now().unwrap();
        assert!(opened(&path).state().table.readings.contains_key("너"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_oversized_file_is_not_read() {
        let dir = scratch("oversized");
        let path = dir.join(FILE_NAME);
        std::fs::write(&path, format!("{HEADER}\n나\t我\t2\t1\t2\n{}", "\n".repeat(MAX_FILE_BYTES as usize))).unwrap();
        assert!(opened(&path).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_save_leaves_no_temporary_file_and_is_private() {
        let dir = scratch("tmp");
        let path = dir.join(FILE_NAME);
        let s = opened(&path);
        s.commit("나", "我", false);
        s.save_now().unwrap();
        let names: Vec<String> = std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
        assert_eq!(names, [FILE_NAME]);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_interrupted_save_leaves_the_old_file() {
        // A crash between the temporary file and the rename.
        let dir = scratch("crash");
        let path = dir.join(FILE_NAME);
        let s = opened(&path);
        s.commit("나", "我", false);
        s.commit("나", "我", false);
        s.save_now().unwrap();
        std::fs::write(temp_path(&path), "half a wri").unwrap();
        assert!(opened(&path).promotes("나"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_save_merges_what_another_process_wrote() {
        let dir = scratch("merge");
        let path = dir.join(FILE_NAME);
        let a = opened(&path);
        let b = opened(&path);
        a.commit("나", "我", false);
        a.save_now().unwrap();
        b.commit("너", "汝", false);
        b.save_now().unwrap();
        // The second writer kept the first one's pick, and took it into memory.
        assert!(b.state().table.readings.contains_key("나"));
        // The first process sees the file changed.
        a.refresh();
        a.wait_loaded();
        assert!(a.state().table.readings.contains_key("너"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn forgetting_clears_memory_file_and_leftovers() {
        let dir = scratch("forget");
        let path = dir.join(FILE_NAME);
        let s = opened(&path);
        s.commit("나", "我", false);
        s.commit("나", "我", false);
        s.save_now().unwrap();
        std::fs::write(temp_path(&path), "left by a crash").unwrap();
        // Another process that holds the table in memory.
        let other = opened(&path);
        assert!(other.promotes("나"));

        s.commit("너", "汝", false);
        s.forget().unwrap();
        assert!(s.is_empty());
        assert!(std::fs::read_dir(&dir).unwrap().next().is_none());
        // Nothing waiting comes back.
        s.save_now().unwrap();
        assert!(!path.exists());

        other.refresh();
        assert!(other.is_empty(), "a deleted file empties the table at once");
        assert!(!opened(&path).promotes("나"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn forgetting_with_no_file_is_fine() {
        let dir = scratch("forget-none");
        forget(&dir).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_folder_that_cannot_be_written_still_learns_in_memory() {
        let dir = scratch("readonly");
        // A file where the folder should be: every write fails, for root too.
        let blocked = dir.join("blocked");
        std::fs::write(&blocked, "").unwrap();
        let s = opened(&blocked.join(FILE_NAME));
        s.commit("나", "我", false);
        s.commit("나", "我", false);
        assert!(s.save_now().is_err());
        assert!(s.promotes("나"));
        // Still in memory after further failed saves, with the picks waiting.
        assert!(s.save_now().is_err());
        assert!(s.promotes("나"));
        assert_eq!(s.state().pending.len(), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_store_with_no_path_learns_in_memory() {
        let s = Store::open(None);
        s.commit("나", "我", false);
        s.commit("나", "我", false);
        assert!(s.promotes("나"));
        s.refresh();
        s.forget().unwrap();
        assert!(s.is_empty());
    }

    #[test]
    fn the_background_save_reaches_the_file_without_waiting_on_the_caller() {
        let dir = scratch("background");
        let path = dir.join(FILE_NAME);
        let s = opened(&path);
        let started = std::time::Instant::now();
        s.commit("나", "我", false);
        assert!(started.elapsed() < SAVE_DELAY, "the commit waited for the save");
        assert!(!path.exists());
        for _ in 0..600 {
            if path.exists() {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(std::fs::read_to_string(&path).unwrap().contains("나\t我"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
