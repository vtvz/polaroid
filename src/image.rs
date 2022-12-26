use magick_rust::{bindings, MagickWand, PixelWand};

use crate::things::{Mm, Size};
use anyhow::{anyhow, Result};

const DEFAULT_DPI: usize = 600;

pub struct Polaroid {
    wand: Option<MagickWand>,
    dpi: usize,
    image_size: Size<Mm>,
    border_thickness: Mm,
    border_color: PixelWand,
    frame_size: Size<Mm>,
    frame_color: PixelWand,
    frame_top_offset: Option<Mm>,
    output_size: Size<Mm>,
    output_color: PixelWand,
}

impl Default for Polaroid {
    fn default() -> Self {
        let mut border_color = PixelWand::new();
        border_color
            .set_color("snow4")
            .expect("Should be compilable");

        let mut frame_color = PixelWand::new();
        frame_color
            .set_color("white")
            .expect("Should be compilable");

        let mut output_color = PixelWand::new();
        output_color
            .set_color("blue")
            .expect("Should be compilable");

        Self {
            wand: None,
            dpi: DEFAULT_DPI,
            image_size: Size::new(Mm(0.0), Mm(0.0)),
            border_thickness: Mm(0.2),
            border_color,
            frame_size: Size::new(Mm(0.0), Mm(0.0)),
            frame_top_offset: None,
            frame_color,
            output_size: Size::new(Mm(102.0), Mm(148.5)),
            output_color,
        }
    }
}

impl Polaroid {
    pub fn new_square() -> Self {
        Self {
            image_size: Size::new(Mm(79.0), Mm(79.0)),
            frame_size: Size::new(Mm(88.), Mm(107.)),
            ..Default::default()
        }
    }

    pub fn new_horisontal() -> Self {
        Self {
            image_size: Size::new(Mm(92.), Mm(73.)),
            frame_size: Size::new(Mm(102.), Mm(102.)),
            output_size: Size::new(Mm(148.5), Mm(102.0)),
            ..Default::default()
        }
    }

    pub fn new_vertical() -> Self {
        Self {
            image_size: Size::new(Mm(61.), Mm(82.)),
            frame_size: Size::new(Mm(70.), Mm(105.)),
            ..Default::default()
        }
    }

    pub fn new_load_predict(path: &str) -> Result<Self> {
        let mut wand = MagickWand::new();
        wand.read_image(path)?;
        wand.auto_orient();
        wand.set_compression_quality(100)?;
        wand.set_image_compression_quality(100)?;

        let (width, height, _, _) = wand.get_image_page();

        let ratio = ((width * 10) as f64 / height as f64).round() / 10.0;

        let mut polaroid = if ratio > 1.1 {
            Self::new_horisontal()
        } else if ratio < 1.0 / 1.1 {
            Self::new_vertical()
        } else {
            Self::new_square()
        };

        polaroid.wand = Some(wand);

        Ok(polaroid)
    }

    pub fn load(&mut self, path: &str) -> Result<()> {
        let mut wand = MagickWand::new();
        wand.read_image(path)?;
        wand.set_compression_quality(100)?;
        wand.set_image_compression_quality(100)?;

        self.wand = Some(wand);

        Ok(())
    }

    pub fn get_wand(&self) -> Result<&MagickWand> {
        self.wand
            .as_ref()
            .ok_or_else(|| anyhow!("Need wand to be loaded"))
    }

    pub fn get_wand_mut(&mut self) -> Result<&mut MagickWand> {
        self.wand
            .as_mut()
            .ok_or_else(|| anyhow!("Need wand to be loaded"))
    }

    pub fn set_dpi(&mut self, dpi: usize) {
        self.dpi = dpi;
    }

    pub fn resize(&mut self) -> Result<()> {
        let wand = self.get_wand()?;

        // size of input image
        let in_size = {
            let (in_width, in_height, _, _) = wand.get_image_page();

            Size::new(in_width, in_height)
        };

        let out_size = self.image_size.to_px(self.dpi);

        let resize_size = if in_size.ratio() > self.image_size.ratio() {
            // height is min
            Size::new(
                (out_size.height as f64 * in_size.ratio()) as usize,
                out_size.height,
            )
        } else {
            // width is min
            Size::new(
                out_size.width,
                (out_size.width as f64 / in_size.ratio()).round() as usize,
            )
        };

        wand.resize_image(
            resize_size.width,
            resize_size.height,
            bindings::FilterType_BoxFilter,
        );

        let offset_size = {
            Size::new(
                if out_size.width > resize_size.width {
                    0
                } else {
                    ((resize_size.width - out_size.width) / 2) as isize
                },
                if out_size.height > resize_size.height {
                    0
                } else {
                    ((resize_size.height - out_size.height) / 2) as isize
                },
            )
        };

        wand.crop_image(
            out_size.width,
            out_size.height,
            offset_size.width,
            offset_size.height,
        )?;

        Ok(())
    }

    pub fn add_border(&self) -> Result<()> {
        self.get_wand()?.border_image(
            &self.border_color,
            self.border_thickness.to_px(self.dpi),
            self.border_thickness.to_px(self.dpi),
            bindings::CompositeOperator_SrcOverCompositeOp,
        )?;

        Ok(())
    }

    pub fn add_frame(&self) -> Result<()> {
        let wand = self.get_wand()?;
        let offset_left = (self.frame_size.width.to_px(self.dpi) - wand.get_image_width()) / 2;
        let offset_top = self
            .frame_top_offset
            .unwrap_or(Mm::from_px(offset_left, self.dpi))
            - &self.border_thickness;

        wand.extend_image(
            self.frame_size.width.to_px(self.dpi),
            self.frame_size.height.to_px(self.dpi),
            offset_left as isize * -1,
            offset_top.to_px(self.dpi) as isize * -1,
        )?;

        Ok(())
    }

    pub fn add_output_filler(&mut self) -> Result<()> {
        let wand = self.get_wand()?;
        let blue_page = MagickWand::new();

        blue_page.new_image(
            self.output_size.width.to_px(self.dpi),
            self.output_size.height.to_px(self.dpi),
            &self.output_color,
        )?;

        let width = wand.get_image_width() as isize;
        let height = wand.get_image_height() as isize;

        blue_page.compose_images(
            &wand,
            bindings::CompositeOperator_OverCompositeOp,
            false,
            self.output_size.width.to_px(self.dpi) as isize / 2 - width / 2,
            self.output_size.height.to_px(self.dpi) as isize / 2 - height / 2,
        )?;

        self.wand = Some(blue_page);

        Ok(())
    }

    pub fn write(mut self, path: &str) -> Result<()> {
        let density = format!("{0}x{0}", self.dpi);
        let wand = self.get_wand_mut()?;

        wand.set_image_property("density", &density)?;
        wand.transform_image_colorspace(bindings::ColorspaceType_CMYKColorspace)?;

        wand.set_compression_quality(100)?;
        wand.set_image_compression_quality(100)?;

        wand.write_image(path)?;

        Ok(())
    }
}
