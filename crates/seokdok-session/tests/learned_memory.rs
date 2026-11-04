//! The heap a full table takes, measured with a counting allocator. This runs
//! in its own test binary because the allocator counts the whole process.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use seokdok_session::learned::{Learned, MAX_ENTRIES};

struct Counting;

static LIVE: AtomicUsize = AtomicUsize::new(0);

// SAFETY: every call goes to the system allocator unchanged.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        LIVE.fetch_add(layout.size(), Ordering::SeqCst);
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Ordering::SeqCst);
        System.dealloc(ptr, layout)
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        LIVE.fetch_add(new_size, Ordering::SeqCst);
        LIVE.fetch_sub(layout.size(), Ordering::SeqCst);
        System.realloc(ptr, layout, new_size)
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

#[test]
fn a_full_table_stays_small() {
    let before = LIVE.load(Ordering::SeqCst);
    let mut t = Learned::new();
    // Words of a usual length: four syllables, written in four Hanja.
    for i in 0..MAX_ENTRIES as u32 {
        let reading: String = (0..4).map(|k| char::from_u32(0xAC00 + (i * 7 + k) % 11172).unwrap()).collect();
        let text: String = (0..4).map(|k| char::from_u32(0x4E00 + (i * 11 + k) % 20000).unwrap()).collect();
        assert!(t.commit(&reading, &text, false));
    }
    let used = LIVE.load(Ordering::SeqCst) - before;
    assert_eq!(t.len(), MAX_ENTRIES);
    let file = t.to_text().len();
    println!("{MAX_ENTRIES} entries: {} KiB in memory, {} KiB on disk", used / 1024, file / 1024);
    assert!(used < 800 * 1024, "{used} bytes in memory");
    assert!(file < 200 * 1024, "{file} bytes on disk");
}
