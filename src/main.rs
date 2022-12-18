mod things;

use anyhow::Result;
use magick_rust::{bindings, magick_wand_genesis, MagickWand, PixelWand};
use std::sync::Once;
use things::{Mm, Size};

extern crate dimensioned as dim;

// Used to make sure MagickWand is initialized exactly once. Note that we
// do not bother shutting down, we simply exit when we're done.
static START: Once = Once::new();

const DPI: usize = 300;

fn load_and_resize(path: &str) -> Result<MagickWand> {
    let mut wand = MagickWand::new();
    wand.read_image(path)?;
    wand.set_compression_quality(100)?;
    wand.set_image_compression_quality(100)?;

    let image_size = Size::new(Mm(79.0), Mm(79.0));

    wand.resize_image(
        image_size.width.to_px(DPI),
        image_size.height.to_px(DPI),
        bindings::FilterType_BoxFilter,
    );

    Ok(wand)
}

fn add_thin_border(wand: &MagickWand) -> Result<Size<Mm>> {
    let mut snow = PixelWand::new();
    snow.set_color("snow4")?;

    let size = Size::new(Mm(0.2), Mm(0.2));

    wand.border_image(
        &snow,
        size.width.to_px(DPI),
        size.height.to_px(DPI),
        bindings::CompositeOperator_SrcOverCompositeOp,
    )?;

    Ok(size)
}

fn add_white_frame(wand: &MagickWand, border_size: &Size<Mm>) -> Result<()> {
    let frame_size = Size::new(Mm(88.), Mm(107.));
    let offset = Size::new(Mm(4.5), Mm(4.5)) - border_size;

    wand.extend_image(
        frame_size.width.to_px(DPI),
        frame_size.height.to_px(DPI),
        offset.width.to_px(DPI) as isize * -1,
        offset.height.to_px(DPI) as isize * -1,
    )?;

    Ok(())
}

fn put_on_blue(wand: &MagickWand) -> Result<MagickWand> {
    let blue_page = MagickWand::new();

    let print_size = Size::new(Mm(102.0), Mm(148.5));
    let mut blue = PixelWand::new();
    blue.set_color("blue")?;

    blue_page.new_image(
        print_size.width.to_px(DPI),
        print_size.height.to_px(DPI),
        &blue,
    )?;

    let width = wand.get_image_width() as isize;
    let height = wand.get_image_height() as isize;

    blue_page.compose_images(
        &wand,
        bindings::CompositeOperator_OverCompositeOp,
        false,
        print_size.width.to_px(DPI) as isize / 2 - width / 2,
        print_size.height.to_px(DPI) as isize / 2 - height / 2,
    )?;

    Ok(blue_page)
}

fn main() -> Result<()> {
    START.call_once(|| {
        magick_wand_genesis();
    });

    // let wand = load_and_resize("IMG_20201228_174301.jpg")?;
    let wand = load_and_resize("in.webp")?;
    let border_size = add_thin_border(&wand)?;
    let _frame_size = add_white_frame(&wand, &border_size)?;
    let mut on_blue = put_on_blue(&wand)?;

    on_blue.set_image_property("density", "300x300")?;
    on_blue.transform_image_colorspace(bindings::ColorspaceType_CMYKColorspace)?;

    on_blue.set_compression_quality(100)?;
    on_blue.set_image_compression_quality(100)?;

    on_blue.write_image("out.tif")?;

    Ok(())
}
