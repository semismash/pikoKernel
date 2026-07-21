use crate::base::input::inputbuffer::{self, InputBuffer};

pub struct InputFrame {
    pub idx: usize,
    size: usize
}

impl InputFrame {

    pub const fn new(frame_size: usize) -> Self {
        Self {
            idx: 0,
            size: frame_size,
        }
    }

    pub fn get_size(&self) -> usize { self.size }

}

pub trait InputToBuffer {

    fn write_from_input();

}