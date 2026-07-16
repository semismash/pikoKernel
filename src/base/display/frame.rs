pub struct Frame {
    idx: usize,
    width: usize,
    height: usize,
}

impl Frame {

    pub fn new(origin: usize, width: usize, height: usize) -> Self {
        Self {
            idx: origin,
            width: width,
            height: height,
        }
    }

}