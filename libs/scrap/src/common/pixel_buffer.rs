use crate::{Pixfmt, TraitPixelBuffer};

pub struct PixelBuffer<'a> {
    data: &'a [u8],
    pixfmt: Pixfmt,
    width: usize,
    height: usize,
    stride: Vec<usize>,
}

impl<'a> PixelBuffer<'a> {
    pub fn new(data: &'a [u8], pixfmt: Pixfmt, width: usize, height: usize) -> Self {
        let stride0 = data.len() / height.max(1);
        Self {
            data,
            pixfmt,
            width,
            height,
            stride: vec![stride0],
        }
    }
}

impl<'a> TraitPixelBuffer for PixelBuffer<'a> {
    fn data(&self) -> &[u8] {
        self.data
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn stride(&self) -> Vec<usize> {
        self.stride.clone()
    }

    fn pixfmt(&self) -> crate::Pixfmt {
        self.pixfmt
    }
}
