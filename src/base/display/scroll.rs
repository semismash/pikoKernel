use crate::base::display::displayframe::{self, DisplayFrame, FlushableBuffer};

pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

pub trait ScrollableBuffer : FlushableBuffer {

    // mandatory
    fn scroll(&mut self, dir: ScrollDirection);
    fn snap_to_cursor(&mut self, snap_row: bool, snap_col: bool);

    // optional
    fn try_snap_to_cursor(&mut self) {
        self.snap_to_cursor(true, true);
    }

    fn auto_scroll_down(&mut self) {
        self.scroll(ScrollDirection::Down);
    }

}