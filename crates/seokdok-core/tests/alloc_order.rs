//! Loading makes the same allocations in the same order every time. The
//! resident peak depends on where the allocator places large blocks, so a
//! load whose order varies with the hash seed peaks differently between runs.
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    static DIGEST: Cell<u64> = const { Cell::new(0xcbf2_9ce4_8422_2325) };
}

fn note(op: u64, size: usize) {
    // `try_with`: the allocator also runs while a thread is being torn down.
    let _ = DIGEST.try_with(|d| d.set((d.get() ^ (op << 56) ^ size as u64).wrapping_mul(0x0100_0000_01b3)));
}

struct Recording;

// SAFETY: every call forwards to the system allocator with the same arguments.
unsafe impl GlobalAlloc for Recording {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        note(1, l.size());
        System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        note(2, l.size());
        System.dealloc(p, l)
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        note(3, n);
        System.realloc(p, l, n)
    }
}

#[global_allocator]
static RECORDING: Recording = Recording;

fn load() -> u64 {
    let native = "읽다\t讀다\tverb\t\teditorial\t\n하늘\t天\tnoun\t\teditorial\t\n";
    let sino = "학교\t學校\t3\t\n학교\t學校\t2\t\n산\t山\t1\t\n";
    let start = DIGEST.with(Cell::get);
    DIGEST.with(|d| d.set(0));
    let (engine, errors) = seokdok_core::engine::Engine::new(&[("n", native)], &[("s", sino)], Default::default());
    let digest = DIGEST.with(Cell::get);
    DIGEST.with(|d| d.set(start));
    assert!(errors.is_empty(), "{errors:?}");
    drop(engine);
    digest
}

#[test]
fn load_allocates_the_same_way_under_every_hash_seed() {
    // The first load fills process-wide tables.
    load();
    // Each thread gets its own hash keys.
    let digests: Vec<u64> = (0..8).map(|_| std::thread::spawn(load).join().unwrap()).collect();
    assert!(digests.iter().all(|d| *d == digests[0]), "{digests:x?}");
}
