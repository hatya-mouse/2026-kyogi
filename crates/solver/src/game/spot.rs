/// An identifier used to distinguish brands of spots.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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
    stocks: u32,
}

impl Spot {
    pub fn new(brand: Brand, stocks: u32) -> Self {
        Self { brand, stocks }
    }

    pub(crate) fn brand(&self) -> &Brand {
        &self.brand
    }

    pub(crate) fn stocks(&self) -> u32 {
        self.stocks
    }
}
