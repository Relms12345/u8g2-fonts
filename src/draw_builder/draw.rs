use embedded_graphics_core::prelude::{DrawTarget, Point};

use crate::{
    font_reader::FontReader, types::RenderedDimensions, utils::combine_bounding_boxes, DrawBuilder,
    Error,
};

use super::{content::Content, DrawColor};

fn render_glyph<Display>(
    ch: char,
    position: Point,
    color_fg: Display::Color,
    color_bg: Option<Display::Color>,
    font: &FontReader,
    display: &mut Display,
) -> Result<RenderedDimensions, Error<Display::Error>>
where
    Display: DrawTarget,
    Display::Error: core::fmt::Debug,
{
    let glyph = font.retrieve_glyph_data(ch)?;

    let advance = glyph.advance();
    let size = glyph.size();

    let bounding_box = if size.width > 0 && size.height > 0 {
        let renderer = glyph.create_renderer(font);
        Some(match color_bg {
            None => renderer.render_transparent(position, display, color_fg)?,
            Some(color_bg) => renderer.render_as_box_fill(position, display, color_fg, color_bg)?,
        })
    } else {
        None
    };

    Ok(RenderedDimensions {
        advance: Point::new(advance as i32, 0),
        bounding_box,
    })
}

pub fn draw_unaligned<T, Display>(
    args: &DrawBuilder<'_, T, DrawColor<Display::Color>>,
    display: &mut Display,
) -> Result<RenderedDimensions, Error<Display::Error>>
where
    T: Content,
    Display: DrawTarget,
    Display::Error: core::fmt::Debug,
{
    let mut position = args.position;
    let font = args.font;

    let mut advance = Point::new(0, 0);

    let mut bounding_box = None;

    position.y += args
        .content
        .compute_vertical_offset(font, args.vertical_pos);

    args.content
        .for_each_char(|ch| -> Result<(), Error<Display::Error>> {
            if ch == '\n' {
                advance.x = 0;
                advance.y += font.font_bounding_box_height as i32 + 1;
            } else {
                let dimensions = render_glyph(
                    ch,
                    position + advance,
                    args.color.fg,
                    args.color.bg,
                    font,
                    display,
                )?;
                advance += dimensions.advance;
                bounding_box = combine_bounding_boxes(bounding_box, dimensions.bounding_box);
            }

            Ok(())
        })?;

    Ok(RenderedDimensions {
        advance,
        bounding_box,
    })
}
