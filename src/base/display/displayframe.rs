pub struct DisplayFrame {
    pub idx: usize,
    width: usize,
    height: usize,
}

impl DisplayFrame {

    pub const fn new(frame_width: usize, frame_height: usize) -> Self {
        Self {
            idx: 0,
            width: frame_width,
            height: frame_height,
        }
    }

    pub fn get_width(&self) -> usize { self.width }
    pub fn get_height(&self) -> usize { self.height }

}

pub struct LastTick(pub u32);

pub struct FramePointer<F>(pub *mut F);
unsafe impl Sync for FramePointer<F> {}

pub trait FlushableBuffer {

    // mandatory
    fn flush(&self, dst: FramePointer<F>);
    fn get_last_tick(&mut self) -> &mut LastTick;

    // optional
    fn flush_cursor(&self) { }

    fn flush_sync(&self, dst: FramePointer<F>) { 
        let last = self.get_last_tick();
        crate::sys::time::SysTime::on_tick(&mut last.0, || {
            self.flush(frame_buf);
        });
    }

}