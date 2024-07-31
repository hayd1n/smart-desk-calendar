use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::{Dimensions, DrawTarget, OriginDimensions, PixelColor, Point, PointsIter, Size},
    Pixel,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FakeDisplay<C> {
    size: Size,
    pub(crate) pixels: Vec<C>,
}

impl<C: PixelColor> FakeDisplay<C> {
    /// Creates a new display filled with a color.
    ///
    /// This constructor can be used if `C` doesn't implement `From<BinaryColor>` or another
    /// default color is wanted.
    pub fn with_default_color(size: Size, default_color: C) -> Self {
        let pixel_count = size.width as usize * size.height as usize;
        let pixels = vec![default_color; pixel_count];

        FakeDisplay { size, pixels }
    }

    /// Returns the color of the pixel at a point.
    ///
    /// # Panics
    ///
    /// Panics if `point` is outside the display.
    pub fn get_pixel(&self, point: Point) -> C {
        self.point_to_index(point)
            .and_then(|index| self.pixels.get(index).copied())
            .expect("can't get point outside of display")
    }

    fn point_to_index(&self, point: Point) -> Option<usize> {
        if let Ok((x, y)) = <(u32, u32)>::try_from(point) {
            if x < self.size.width && y < self.size.height {
                return Some((x + y * self.size.width) as usize);
            }
        }

        None
    }

    /// Compares the content of this display with another display.
    ///
    /// If both displays are equal `None` is returned, otherwise a difference image is returned.
    /// All pixels that are different will be filled with `BinaryColor::On` and all equal pixels
    /// with `BinaryColor::Off`.
    ///
    /// # Panics
    ///
    /// Panics if the both display don't have the same size.
    pub fn diff(&self, other: &FakeDisplay<C>) -> Option<FakeDisplay<BinaryColor>> {
        assert!(
            self.size == other.size,
            // TODO: use Display impl for Size
            "both displays must have the same size (self: {}x{}, other: {}x{})",
            self.size.width,
            self.size.height,
            other.size.width,
            other.size.height,
        );

        let pixels = self
            .bounding_box()
            .points()
            .map(|p| BinaryColor::from(self.get_pixel(p) != other.get_pixel(p)))
            .collect::<Vec<_>>();

        if pixels.iter().any(|p| *p == BinaryColor::On) {
            Some(FakeDisplay {
                pixels,
                size: self.size,
            })
        } else {
            None
        }
    }
}

impl<C> FakeDisplay<C>
where
    C: PixelColor + From<BinaryColor>,
{
    /// Creates a new display.
    ///
    /// The display is filled with `C::from(BinaryColor::Off)`.
    pub fn new(size: Size) -> Self {
        Self::with_default_color(size, C::from(BinaryColor::Off))
    }
}

impl<C: PixelColor> DrawTarget for FakeDisplay<C> {
    type Color = C;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels.into_iter() {
            if let Some(index) = self.point_to_index(point) {
                self.pixels[index] = color;
            }
        }

        Ok(())
    }
}

impl<C> OriginDimensions for FakeDisplay<C> {
    fn size(&self) -> Size {
        self.size
    }
}
