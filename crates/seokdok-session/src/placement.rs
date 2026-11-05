//! Where the popup goes on the screen, given where the text is. The rules are
//! kept apart from the Windows calls that gather the facts, so they can be tested.

/// The line of text being composed, in screen pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Caret {
    pub x: i32,
    pub top: i32,
    pub bottom: i32,
}

/// A rectangle in screen pixels: a monitor's work area. Its origin is negative
/// on a monitor left of or above the primary one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Area {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// Space kept between the line and the popup.
const GAP: i32 = 2;

/// The popup's left and top edges and its width, kept inside `area`: below the
/// line, or above it when only that side has room, or on the roomier side,
/// pushed back inside, when neither has. A line outside the area, such as the
/// bottom of a window hanging off the monitor, also comes back inside.
pub fn place(caret: Caret, width: i32, height: i32, area: Area) -> (i32, i32, i32) {
    let width = width.min(area.right - area.left);
    let x = caret.x.min(area.right - width).max(area.left);
    let below = caret.bottom + GAP;
    let above = caret.top - GAP - height;
    let y = if below + height <= area.bottom {
        below
    } else if above >= area.top {
        above
    } else if area.bottom - below >= caret.top - area.top {
        area.bottom - height
    } else {
        area.top
    };
    (x, y.min(area.bottom - height).max(area.top), width)
}

/// Whether a position the application gave can be the text in its window
/// (`client`, in screen pixels; empty if the application does not say). Without
/// a caret to report, the IMM32 bridge answers with a line of no height at the
/// corner of the work area, which a maximized window contains.
pub fn believable(caret: Caret, client: Area) -> bool {
    let inside = client.left >= client.right
        || client.top >= client.bottom
        || (client.left..=client.right).contains(&caret.x) && caret.top >= client.top && caret.bottom <= client.bottom;
    caret.bottom > caret.top && inside
}

/// How many windows are remembered.
const REMEMBERED: usize = 8;

/// Where each window's caret last was, kept relative to the window so that it
/// follows when the window moves. The latest is last.
#[derive(Debug, Default)]
pub struct CaretMemory {
    windows: Vec<(usize, Caret)>,
}

impl CaretMemory {
    pub fn remember(&mut self, window: usize, (x, y): (i32, i32), caret: Caret) {
        self.windows.retain(|(w, _)| *w != window);
        if self.windows.len() == REMEMBERED {
            self.windows.remove(0);
        }
        self.windows.push((window, Caret { x: caret.x - x, top: caret.top - y, bottom: caret.bottom - y }));
    }

    pub fn recall(&self, window: usize, (x, y): (i32, i32)) -> Option<Caret> {
        let (_, caret) = self.windows.iter().find(|(w, _)| *w == window)?;
        Some(Caret { x: caret.x + x, top: caret.top + y, bottom: caret.bottom + y })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: Area = Area { left: 0, top: 0, right: 1920, bottom: 1040 };
    /// Left of the primary monitor.
    const LEFT: Area = Area { left: -1600, top: 100, right: 0, bottom: 1000 };

    fn line(x: i32, top: i32) -> Caret {
        Caret { x, top, bottom: top + 20 }
    }

    const WINDOW: Area = Area { left: 112, top: 35, right: 912, bottom: 660 };

    #[test]
    fn a_caret_in_the_window_is_believed() {
        assert!(believable(Caret { x: 398, top: 131, bottom: 149 }, WINDOW));
    }

    #[test]
    fn the_bridge_corner_is_not_believed_outside_or_inside_the_window() {
        let corner = Caret { x: 1023, top: 719, bottom: 719 };
        assert!(!believable(corner, WINDOW));
        assert!(!believable(corner, Area { left: 0, top: 0, right: 1024, bottom: 720 }));
    }

    #[test]
    fn a_line_outside_the_window_is_not_believed() {
        assert!(!believable(Caret { x: 1000, top: 100, bottom: 120 }, WINDOW));
        assert!(!believable(Caret { x: 200, top: 650, bottom: 670 }, WINDOW));
    }

    #[test]
    fn a_window_that_gives_no_rectangle_is_taken_at_its_word() {
        assert!(believable(Caret { x: 5, top: 5, bottom: 25 }, Area { left: 0, top: 0, right: 0, bottom: 0 }));
    }

    #[test]
    fn a_remembered_caret_follows_its_window() {
        let mut memory = CaretMemory::default();
        memory.remember(1, (100, 100), Caret { x: 150, top: 400, bottom: 420 });
        assert_eq!(memory.recall(1, (100, 100)), Some(Caret { x: 150, top: 400, bottom: 420 }));
        assert_eq!(memory.recall(1, (300, 50)), Some(Caret { x: 350, top: 350, bottom: 370 }));
        assert_eq!(memory.recall(2, (100, 100)), None);
    }

    #[test]
    fn a_window_keeps_one_caret_and_the_oldest_window_goes_first() {
        let mut memory = CaretMemory::default();
        let at = |top| Caret { x: 0, top, bottom: top + 20 };
        memory.remember(1, (0, 0), at(10));
        memory.remember(1, (0, 0), at(20));
        assert_eq!(memory.recall(1, (0, 0)), Some(at(20)));
        for window in 2..=9 {
            memory.remember(window, (0, 0), at(30));
        }
        assert_eq!(memory.recall(1, (0, 0)), None);
        assert_eq!(memory.recall(9, (0, 0)), Some(at(30)));
    }

    #[test]
    fn it_opens_below_the_line() {
        assert_eq!(place(line(100, 200), 300, 200, SCREEN), (100, 222, 300));
    }

    #[test]
    fn it_stays_on_the_screen_at_the_right_edge() {
        assert_eq!(place(line(1800, 200), 300, 200, SCREEN), (1620, 222, 300));
    }

    #[test]
    fn it_flips_above_a_message_field_at_the_bottom() {
        assert_eq!(place(line(100, 1000), 300, 200, SCREEN), (100, 798, 300));
    }

    #[test]
    fn it_fits_exactly_above_the_taskbar_without_flipping() {
        assert_eq!(place(line(100, 818), 300, 200, SCREEN), (100, 840, 300));
        assert_eq!(place(line(100, 819), 300, 200, SCREEN), (100, 617, 300));
    }

    #[test]
    fn it_is_never_wider_than_the_monitor() {
        assert_eq!(place(line(100, 200), 3000, 200, SCREEN), (0, 222, 1920));
    }

    #[test]
    fn a_monitor_left_of_the_primary_keeps_it() {
        assert_eq!(place(line(-1500, 300), 300, 200, LEFT), (-1500, 322, 300));
        assert_eq!(place(line(-100, 300), 300, 200, LEFT), (-300, 322, 300));
        assert_eq!(place(line(-1700, 300), 300, 200, LEFT), (-1600, 322, 300));
    }

    #[test]
    fn a_monitor_starting_below_zero_flips_within_itself() {
        assert_eq!(place(line(-1500, 970), 300, 200, LEFT), (-1500, 768, 300));
    }

    #[test]
    fn a_popup_taller_than_either_side_stays_inside_on_the_roomier_one() {
        assert_eq!(place(line(100, 300), 300, 900, SCREEN), (100, 140, 300));
        assert_eq!(place(line(100, 800), 300, 900, SCREEN), (100, 0, 300));
    }

    #[test]
    fn a_line_off_the_monitor_brings_the_popup_back_inside() {
        assert_eq!(place(line(100, 1200), 48, 40, SCREEN), (100, 1000, 48));
        assert_eq!(place(line(100, -300), 48, 40, SCREEN), (100, 0, 48));
    }

    #[test]
    fn a_popup_taller_than_the_monitor_starts_at_its_top() {
        assert_eq!(place(line(100, 300), 300, 2000, SCREEN), (100, 0, 300));
    }
}
