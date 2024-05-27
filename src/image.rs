use anyhow::{anyhow, Result};
use bevy::render::render_resource::TextureFormat;
use serde::{Deserialize, Serialize};

use crate::{palette::Palette, pixel::Pixel, prelude::*};

#[derive(Serialize, Deserialize, Reflect, Debug)]
pub(crate) struct PxImage<P: Pixel> {
    image: Vec<P>,
    width: usize,
}

impl<P: Pixel> PxImage<P> {
    pub(crate) fn new(image: Vec<P>, width: usize) -> Self {
        Self { image, width }
    }

    pub(crate) fn empty(size: UVec2) -> Self {
        Self {
            image: vec![default(); (size.x * size.y) as usize],
            width: size.x as usize,
        }
    }

    pub(crate) fn empty_from_image(image: &Image) -> Self {
        Self {
            image: vec![default(); image.data.len()],
            width: image.texture_descriptor.size.width as usize,
        }
    }

    pub(crate) fn pixel(&self, position: IVec2) -> P {
        self.image[(position.x + position.y * self.width as i32) as usize]
    }

    pub(crate) fn get_pixel(&self, position: IVec2) -> Option<P> {
        IRect {
            min: IVec2::splat(0),
            max: IVec2::new(self.width as i32, (self.image.len() / self.width) as i32),
        }
        .contains(position)
        .then(|| self.pixel(position))
    }

    #[allow(dead_code)]
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

    pub(crate) fn slice_mut(&mut self, slice: IRect) -> PxImageSliceMut<P> {
        PxImageSliceMut {
            slice,
            image: self.image.chunks_exact_mut(self.width).collect(),
            width: self.width,
        }
    }

    pub(crate) fn slice_all_mut(&mut self) -> PxImageSliceMut<P> {
        PxImageSliceMut {
            slice: IRect {
                min: IVec2::splat(0),
                max: IVec2::new(self.width as i32, (self.image.len() / self.width) as i32),
            },
            image: self.image.chunks_exact_mut(self.width).collect(),
            width: self.width,
        }
    }

    pub(crate) fn flip_vert(&self) -> Self {
        PxImage {
            image: self
                .image
                .chunks_exact(self.width)
                .rev()
                .flatten()
                .copied()
                .collect(),
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

impl PxImage<Option<u8>> {
    pub(crate) fn palette_indices(palette: &Palette, image: &Image) -> Result<Self> {
        Ok(Self {
            image: image
                .convert(TextureFormat::Rgba8UnormSrgb)
                .ok_or_else(|| anyhow!("could not convert image to `Rgba8UnormSrgb`"))?
                .data
                .chunks_exact(image.texture_descriptor.size.width as usize * 4)
                .rev()
                .flatten()
                .copied()
                .collect::<Vec<_>>()
                .chunks_exact(4)
                .map(|color| {
                    (color[3] != 0)
                        .then(|| {
                            palette
                                .indices
                                .get(&[color[0], color[1], color[2]])
                                .copied()
                                .ok_or_else(|| {
                                    anyhow!("a sprite contained a color that wasn't in the palette")
                                })
                        })
                        .transpose()
                })
                .collect::<Result<_>>()?,
            width: image.texture_descriptor.size.width as usize,
        })
    }

    pub(crate) fn palette_indices_unaligned(palette: &Palette, image: &Image) -> Result<Self> {
        Ok(Self {
            image: image
                .convert(TextureFormat::Rgba8UnormSrgb)
                .unwrap()
                .data
                .chunks_exact(4)
                .map(|color| {
                    (color[3] != 0)
                        .then(|| {
                            palette
                                .indices
                                .get(&[color[0], color[1], color[2]])
                                .copied()
                                .ok_or_else(|| {
                                    anyhow!("a sprite contained a color that wasn't in the palette")
                                })
                        })
                        .transpose()
                })
                .collect::<Result<_>>()?,
            width: image.texture_descriptor.size.width as usize,
        })
    }

    pub(crate) fn trim_right(&mut self) {
        while (0..self.height()).all(|row| self.image[self.width * (row + 1) - 1].is_none()) {
            for row in (0..self.height()).rev() {
                self.image.remove(row * self.width + self.width - 1);
            }

            self.width -= 1;
        }
    }
}

pub(crate) struct PxImageSliceMut<'a, P: Pixel> {
    image: Vec<&'a mut [P]>,
    width: usize,
    slice: IRect,
}

impl<'a, P: Pixel> PxImageSliceMut<'a, P> {
    /// First `usize` is the index in the slice. Second `usize` is the index in the image.
    pub(crate) fn for_each_mut(&mut self, f: impl Fn(usize, usize, &mut P)) {
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

    pub(crate) fn pixel_mut(&mut self, position: IVec2) -> &mut P {
        &mut self.image[(self.slice.min.y + position.y) as usize]
            [(self.slice.min.x + position.x) as usize]
    }

    pub(crate) fn get_pixel_mut(&mut self, position: IVec2) -> Option<&mut P> {
        (IRect {
            min: IVec2::splat(0),
            max: IVec2::new(self.width as i32, self.image.len() as i32),
        }
        .contains(position + self.slice.min)
            && self.slice.contains(position))
        .then(|| self.pixel_mut(position))
    }

    pub(crate) fn width(&self) -> u32 {
        self.slice.size().x as u32
    }

    pub(crate) fn image_width(&self) -> usize {
        self.width
    }

    pub(crate) fn draw(&mut self, image: &PxImage<impl Pixel>) {
        self.for_each_mut(|i, _, pixel| {
            pixel.set(image.image[i]);
        });
    }
}

impl<'a> PxImageSliceMut<'a, u8> {
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
                .rev()
                .collect(),
            width: image.texture_descriptor.size.width as usize,
        }
    }
}
