use anyhow::{anyhow, Result};
use bevy::render::render_resource::TextureFormat;
use serde::{Deserialize, Serialize};

use crate::{math::RectExt, palette::Palette, prelude::*};

#[derive(Serialize, Deserialize, Clone, Reflect, Debug)]
pub(crate) struct PxImage {
    image: Vec<u8>,
    width: usize,
}

impl PxImage {
    pub(crate) fn new(image: Vec<u8>, width: usize) -> Self {
        Self { image, width }
    }

    pub(crate) fn empty(size: UVec2) -> Self {
        Self {
            image: vec![0; (size.x * size.y) as usize],
            width: size.x as usize,
        }
    }

    pub(crate) fn empty_from_image(image: &Image) -> Self {
        Self::empty(image.size())
    }

    pub(crate) fn palette_indices(palette: &Palette, image: &Image) -> Result<Self> {
        Ok(Self {
            image: image
                .convert(TextureFormat::Rgba8UnormSrgb)
                .ok_or_else(|| anyhow!("could not convert image to `Rgba8UnormSrgb`"))?
                .data
                .chunks_exact(4)
                .map(|color| {
                    if color[3] == 0 {
                        Ok(0)
                    } else {
                        palette
                            .indices
                            .get(&[color[0], color[1], color[2]])
                            .copied()
                            .ok_or_else(|| {
                                anyhow!(
                                    "a sprite contained a color `#{:02X}{:02X}{:02X}` \
                                    that wasn't in the palette",
                                    color[0],
                                    color[1],
                                    color[2]
                                )
                            })
                    }
                })
                .collect::<Result<_>>()?,
            width: image.texture_descriptor.size.width as usize,
        })
    }

    pub(crate) fn pixel(&self, position: IVec2) -> u8 {
        self.image[(position.x + position.y * self.width as i32) as usize]
    }

    pub(crate) fn get_pixel(&self, position: IVec2) -> Option<u8> {
        IRect {
            min: IVec2::splat(0),
            max: IVec2::new(self.width as i32, (self.image.len() / self.width) as i32),
        }
        .contains_exclusive(position)
        .then(|| self.pixel(position))
    }

    pub(crate) fn size(&self) -> UVec2 {
        UVec2::new(self.width as u32, (self.image.len() / self.width) as u32)
    }

    pub(crate) fn width(&self) -> usize {
        self.width
    }

    pub(crate) fn height(&self) -> usize {
        self.image.len() / self.width
    }

    pub(crate) fn area(&self) -> usize {
        self.image.len()
    }

    #[expect(unused)]
    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut u8> {
        self.image.iter_mut()
    }

    #[expect(unused)]
    pub(crate) fn slice_mut(&mut self, slice: IRect) -> PxImageSliceMut {
        PxImageSliceMut {
            slice,
            image: self.image.chunks_exact_mut(self.width).collect(),
            width: self.width,
        }
    }

    pub(crate) fn slice_all_mut(&mut self) -> PxImageSliceMut {
        PxImageSliceMut {
            slice: IRect {
                min: IVec2::splat(0),
                max: IVec2::new(self.width as i32, (self.image.len() / self.width) as i32),
            },
            image: self.image.chunks_exact_mut(self.width).collect(),
            width: self.width,
        }
    }

    pub(crate) fn split_vert(self, chunk_height: usize) -> Vec<Self> {
        self.image
            .chunks_exact(chunk_height * self.width)
            .map(|chunk| Self {
                image: chunk.into(),
                width: self.width,
            })
            .collect()
    }

    pub(crate) fn split_horz(self, chunk_width: usize) -> Vec<Self> {
        let chunk_count = self.width / chunk_width;
        let mut images = vec![Vec::with_capacity(self.area() / chunk_width); chunk_count];

        for (i, chunk_row) in self.image.chunks_exact(chunk_width).enumerate() {
            images[i % chunk_count].push(chunk_row);
        }

        images
            .into_iter()
            .map(|image| Self {
                image: image.into_iter().flatten().copied().collect(),
                width: chunk_width,
            })
            .collect()
    }

    pub(crate) fn trim_right(&mut self) {
        while (0..self.height()).all(|row| self.image[self.width * (row + 1) - 1] == 0) {
            for row in (0..self.height()).rev() {
                self.image.remove(row * self.width + self.width - 1);
            }

            self.width -= 1;
        }
    }

    pub(crate) fn from_parts_vert(parts: impl IntoIterator<Item = Self>) -> Option<Self> {
        let (images, widths): (Vec<_>, Vec<_>) = parts
            .into_iter()
            .map(|image| (image.image, image.width))
            .unzip();

        match (&widths) as &[_] {
            [width, other_widths @ ..] => other_widths
                .iter()
                .all(|other_width| other_width == width)
                .then(|| Self {
                    image: images.into_iter().flatten().collect(),
                    width: *width,
                }),
            [] => None,
        }
    }

    pub(crate) fn clear(&mut self) {
        self.image.fill(default());
    }
}

pub(crate) struct PxImageSliceMut<'a> {
    // TODO Currently, this is the entire image. Trim it down to the slice that this should have
    // access to.
    image: Vec<&'a mut [u8]>,
    width: usize,
    slice: IRect,
}

impl<'a> PxImageSliceMut<'a> {
    pub(crate) fn from_image_mut(image: &'a mut Image) -> Self {
        Self {
            slice: IRect {
                min: IVec2::splat(0),
                max: IVec2::new(
                    image.texture_descriptor.size.width as i32,
                    image.texture_descriptor.size.height as i32,
                ),
            },
            image: image
                .data
                .chunks_exact_mut(image.texture_descriptor.size.width as usize)
                .collect(),
            width: image.texture_descriptor.size.width as usize,
        }
    }

    /// First `usize` is the index in the slice. Second `usize` is the index in the image.
    pub(crate) fn for_each_mut(&mut self, f: impl Fn(usize, usize, &mut u8)) {
        let row_min = self.slice.min.x.clamp(0, self.width as i32) as usize;
        let row_max = self.slice.max.x.clamp(0, self.width as i32) as usize;
        let max_y = self.image.len() as i32;

        self.image.iter_mut().enumerate().collect::<Vec<_>>()
            [self.slice.min.y.clamp(0, max_y) as usize..self.slice.max.y.clamp(0, max_y) as usize]
            .iter_mut()
            .for_each(|(i, row)| {
                row.iter_mut().enumerate().collect::<Vec<_>>()[row_min..row_max]
                    .iter_mut()
                    .for_each(|(j, pixel)| {
                        f(
                            ((*i as i32 - self.slice.min.y) * (self.slice.max.x - self.slice.min.x)
                                + (*j as i32 - self.slice.min.x))
                                as usize,
                            *i * self.width + *j,
                            pixel,
                        );
                    });
            });
    }

    pub(crate) fn contains_pixel(&self, position: IVec2) -> bool {
        IRect {
            min: IVec2::splat(0),
            max: IVec2::new(self.width as i32, self.image.len() as i32),
        }
        .contains_exclusive(position - self.slice.min)
            && self.slice.contains_exclusive(position)
    }

    pub(crate) fn pixel_mut(&mut self, position: IVec2) -> &mut u8 {
        &mut self.image[(self.slice.min.y + position.y) as usize]
            [(self.slice.min.x + position.x) as usize]
    }

    pub(crate) fn get_pixel_mut(&mut self, position: IVec2) -> Option<&mut u8> {
        self.contains_pixel(position)
            .then(|| self.pixel_mut(position))
    }

    pub(crate) fn image_pixel_mut(&mut self, position: IVec2) -> &mut u8 {
        &mut self.image[position.y as usize][position.x as usize]
    }

    #[expect(unused)]
    pub(crate) fn size(&self) -> UVec2 {
        self.slice.size().as_uvec2()
    }

    pub(crate) fn width(&self) -> u32 {
        self.slice.width() as u32
    }

    pub(crate) fn height(&self) -> u32 {
        self.slice.height() as u32
    }

    pub(crate) fn image_width(&self) -> usize {
        self.width
    }

    pub(crate) fn image_height(&self) -> usize {
        self.image.len()
    }

    #[allow(unused)]
    pub(crate) fn offset(&self) -> IVec2 {
        self.slice.min
    }

    pub(crate) fn slice_mut(&mut self, slice: IRect) -> PxImageSliceMut {
        PxImageSliceMut {
            image: self.image.iter_mut().map(|row| &mut **row).collect(),
            width: self.width,
            slice: IRect {
                min: slice.min + self.slice.min,
                max: slice.max + self.slice.min,
            },
        }
    }

    pub(crate) fn draw(&mut self, image: &PxImage) {
        self.for_each_mut(|i, _, pixel| {
            let new_pixel = image.image[i];
            if new_pixel != 0 {
                *pixel = new_pixel;
            }
        });
    }
}
