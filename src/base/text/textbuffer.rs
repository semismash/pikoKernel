pub struct TextBuffer<T, const N: usize> {
    container: [T; N],
}

pub enum TextBufferError {
    UnknownError,
    OutOfBounds,
}

//access and mutation
impl<T, const N: usize> TextBuffer<T, N> {

    pub const fn new(fill: T) -> Self {
        Self {
            container: [fill; N],
        }
    }

    pub fn default() -> Self 
    where
        T: Default,
    {
        Self {
            container: [T::default(); N],
        }
    }

    pub fn get(&self, pos: usize) -> Option<&T> {
        self.container.get(pos)
    }

    pub fn get_copy(&self, pos: usize) -> Option<T>
    where
        T: Copy,
    {
        self.container.get(pos).copied()
    }

    pub fn get_mut(&mut self, pos: usize) -> Option<&mut T> {
        self.container.get_mut(pos)
    }

    pub fn set(&mut self, pos: usize, new_value: T) -> Result<(), TextBufferError> {
        let cell = self.container.get_mut(pos);
        match cell {
            Some(c) => { *c = new_value; Ok(()) },
            None => { Err(TextBufferError::OutOfBounds) },
        }
    }

}

//struct-wide access
impl<T, const N: usize> TextBuffer<T, N> {

    pub fn as_slice(&self) -> &[T] { &self.container }
    pub fn as_slice_mut(&mut self) -> &mut [T] { &mut self.container }
    pub const fn capacity(&self) -> usize { N }

}
