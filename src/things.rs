use std::ops::{Add, Sub};

const MM_PER_INCH: f64 = 25.4;

#[derive(Clone, Copy)]
pub struct Mm(pub f64);

impl Mm {
    pub fn to_px(self, dpi: usize) -> usize {
        let Self(mm) = self;

        (mm * dpi as f64 / MM_PER_INCH).round() as usize
    }

    pub fn from_px(px: usize, dpi: usize) -> Self {
        Mm(px as f64 * MM_PER_INCH / dpi as f64)
    }
}

impl Add<&Self> for Mm {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub<&Self> for Mm {
    type Output = Self;

    fn sub(self, rhs: &Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> {
    pub fn new(width: T, height: T) -> Self {
        return Self { width, height };
    }
}

impl Size<Mm> {
    pub fn to_px(&self, dpi: usize) -> Size<usize> {
        Size {
            width: self.width.to_px(dpi),
            height: self.height.to_px(dpi),
        }
    }

    pub fn ratio(&self) -> f64 {
        self.width.0 as f64 / self.height.0 as f64
    }
}

impl Size<usize> {
    fn to_mm(&self, dpi: usize) -> Size<Mm> {
        Size {
            width: Mm::from_px(self.width, dpi),
            height: Mm::from_px(self.height, dpi),
        }
    }

    pub fn ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }
}

impl<T> Add for Size<T>
where
    T: Add<T, Output = T>,
{
    type Output = Size<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl<T> Sub<&Self> for Size<T>
where
    T: Sub<T, Output = T> + Copy,
{
    type Output = Size<T>;

    fn sub(self, rhs: &Self) -> Self::Output {
        Self {
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}
