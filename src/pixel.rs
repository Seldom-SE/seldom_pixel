pub(crate) trait Pixel: Copy + Default {
    fn set(&mut self, pixel: impl Pixel) {
        if let Some(pixel) = pixel.get_value() {
            self.set_value(pixel);
        }
    }

    fn get_value(&self) -> Option<u8>;
    fn get_value_mut(&mut self) -> Option<&mut u8>;
    fn set_value(&mut self, pixel: u8);
}

impl Pixel for u8 {
    fn get_value(&self) -> Option<u8> {
        Some(*self)
    }

    fn get_value_mut(&mut self) -> Option<&mut u8> {
        Some(self)
    }

    fn set_value(&mut self, pixel: u8) {
        *self = pixel;
    }
}

impl Pixel for Option<u8> {
    fn get_value(&self) -> Option<u8> {
        *self
    }

    fn get_value_mut(&mut self) -> Option<&mut u8> {
        self.as_mut()
    }

    fn set_value(&mut self, pixel: u8) {
        *self = Some(pixel);
    }
}
