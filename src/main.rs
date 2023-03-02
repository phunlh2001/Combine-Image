mod args;
use std::{io::BufReader, fs::File};

use args::Args;
use image::{DynamicImage, ImageFormat, io::Reader, GenericImageView, imageops::FilterType::Triangle};

#[derive(Debug)]
enum ImageDataErrors {
    DifferentImageFormats
}

struct FloatingImage {
    width: u32,
    height: u32,
    data: Vec<u8>,
    name: String
}

impl FloatingImage {
    fn new(width: u32, height: u32, name: String) -> Self {
        // create buffer for data image... limit data: 956x956x4 (convert to Binary)
        let buffer_capacity = 3_655_744;
        let buffer: Vec<u8> = Vec::with_capacity(buffer_capacity);

        FloatingImage { 
            width: width, 
            height: height, 
            data: buffer, 
            name: name 
        }
    }

    fn set_data(&mut self, data: Vec<u8>) -> Result<(), ImageDataErrors> {
        // if the previously assigned buffer is too small to hold the new data
        if data.len() > self.data.capacity() {
            return Err(ImageDataErrors::DifferentImageFormats);
        }
        self.data = data;
        Ok(())
    }
}

fn main() -> Result<(), ImageDataErrors> {
    let args = Args::new();
    println!("{:?}", args);

    let (image_1, image_1_format) = find_image_from_path(args.image_1);
    let (image_2, image_2_format) = find_image_from_path(args.image_2);

    // handle error
    if image_1_format != image_2_format {
        return Err(ImageDataErrors::DifferentImageFormats);
    }

    let (image_1, image_2) = standardise_size(image_1, image_2);

    let mut output = FloatingImage::new(image_1.width(), image_1.height(), args.output);

    let combined_data = combine_images(image_1, image_2);

    output.set_data(combined_data)?;

    image::save_buffer_with_format(
        output.name, 
        &output.data, 
        output.width, 
        output.height, 
        image::ColorType::Rgba8, 
        image_1_format).unwrap();
    Ok(())
}

fn find_image_from_path(path: String) -> (DynamicImage, ImageFormat) {
    let image_reader: Reader<BufReader<File>> = Reader::open(path).unwrap();
    let image_format: ImageFormat = image_reader.format().unwrap();
    let image: DynamicImage = image_reader.decode().unwrap();
    // return tuple into local fn
    (image, image_format)
}

fn get_smallest_dimensions(dim_1: (u32, u32), dim_2: (u32, u32)) -> (u32, u32) {
    // get index
    let pix_1 = dim_1.0 * dim_1.1;
    let pix_2 = dim_2.0 * dim_2.1;
    return if pix_1 < pix_2 { dim_1 } else { dim_2 }
}


/**
 * resize image
 */
fn standardise_size(image_1: DynamicImage, image_2: DynamicImage) -> (DynamicImage, DynamicImage) {
    // find which image have smallest dimension by width and height
    let (width, height) = get_smallest_dimensions(image_1.dimensions(), image_2.dimensions());
    println!("width: {}, height: {}\n", width, height);

    // check to resize image larger
    if image_2.dimensions() == (width, height) {
        (image_1.resize_exact(width, height, Triangle), image_2)
    } else {
        (image_1, image_2.resize_exact(width, height, Triangle))
    }
}


/**
 * convert vector to RGBA pixel. 'cuz pixel can stored u8 as Binary data
 */
fn combine_images(image_1: DynamicImage, image_2: DynamicImage) -> Vec<u8> {
    let vec_1 = image_1.to_rgb8().to_vec();
    let vec_2 = image_2.to_rgb8().to_vec();

    alternate_pixels(vec_1, vec_2)
}

fn alternate_pixels(vec_1: Vec<u8>, vec_2: Vec<u8>) -> Vec<u8> {
    // A Vec<u8> is created with the same length as vec_1
    let mut combined_data = vec![0u8; vec_1.len()];

    let mut i = 0;
    while i < vec_1.len() {
        if i % 8 == 0 {
            combined_data.splice(i..=i+3, set_rgba(&vec_1, i, i+3));
        } else {
            combined_data.splice(i..=i+3, set_rgba(&vec_2, i, i+3));
        }
        i += 4;
    }

    combined_data
}

fn set_rgba(vec: &Vec<u8>, start: usize, end: usize) -> Vec<u8> {
    let mut rgba = Vec::new();

    for i in start..end {
        let val = match vec.get(i) {
            Some(d) => *d,
            None => panic!("Index out of bounds")
        };
        rgba.push(val);
    }
    rgba
}