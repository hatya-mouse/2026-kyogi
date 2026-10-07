/// An identifier used to distinguish brands of spots.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Brand(i64);

impl Brand {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

/// Spot that offers some Udons.
pub struct Spot {
    /// The brand of the spot.
    brand: Brand,
    /// Number of stocks of the spot.
    max_stocks: u32,
}

impl Spot {
    pub fn new(brand: Brand, max_stocks: u32) -> Self {
        Self { brand, max_stocks }
    }

    pub(crate) fn brand(&self) -> &Brand {
        &self.brand
    }

    pub(crate) fn stocks(&self) -> u32 {
        self.max_stocks
    }
}
