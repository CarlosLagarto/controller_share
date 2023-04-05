#![allow(dead_code)] // Manter para referência de um API completo

/// Dimension = 8
pub struct Scale {
    pub min: f32,
    pub max: f32,
}

#[rustfmt::skip]
impl Scale {
    #[inline]
    pub fn new(min: f32, max: f32) -> Scale { Scale { min, max } }

    #[inline]
    pub fn get(&self, value: f32) -> f32 { self.min + (value * (self.max - self.min)) }

    #[inline]
    pub fn span(&self) -> f32 { (self.max - self.min) / 2. }

    #[inline]
    pub fn middle(&self) -> f32 { self.min + (self.span() / 2.) }
}
