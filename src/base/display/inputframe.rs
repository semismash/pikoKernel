use crate::base::input::inputbuffer::{self, InputBuffer};

pub struct InputFrame {
    idx: usize,
    width: usize,
    height: usize,
}

pub trait InputToBuffer {

    pub fn write_from_input();

}