pub struct DisplayFrame {
    idx: usize,
    width: usize,
    height: usize,
}

pub struct LastTick {
    last_tick: u32,
}

pub struct FramePointer(pub *mut FrameBuffer);
unsafe impl Sync for FramePointer {}

pub trait FlushableBuffer {

    // mandatory
    fn flush(&self, dst: FramePointer);
    fn get_last_tick(&mut self) -> &mut LastTick;

    // optional
    fn flush_cursor(&self) { }

    fn flush_sync(&self, dst: FramePointer) { 
        let last = self.get_last_tick();
        crate::sys::time::SysTime::on_tick(&mut last.last_tick, || {
            self.flush(frame_buf);
        });
    }

}