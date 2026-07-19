pub struct DisplayFrame {
    idx: usize,
    width: usize,
    height: usize,
}

pub trait FlushableBuffer {

    // mandatory
    pub fn flush(&self, dst: usize);

    // optional
    pub fn flush_cursor(&self) { }
    pub fn flush_sync(&self) { }

}