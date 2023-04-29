use std::fs;
use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::process::ExitCode;
use image;
use clap::{Parser, Subcommand};
use thiserror::Error;

/// Errors from Picbin
#[derive(Error, Debug)]
pub enum PicbinError {

    /// File size is too large to represent the content in an image
    #[error("file size too large")]
    FileSizeTooLarge,

    /// The destination already exists and overwriting is not allowed
    #[error("destination exists; you might have forgot --overwrite option: {0}")]
    DestinationExists(String),

    /// Generic IO error
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),

    /// Generic imaging error
    #[error("Image Error: {0}")]
    Imaging(#[from] image::error::ImageError),
}

/// Command line arguments structure
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Overwrite the existing destination
    #[arg(short, long)]
    overwrite: bool,

    /// Subcommand
    #[command(subcommand)]
    command: Commands,
}

/// Subcommands
#[derive(Subcommand)]
enum Commands {
    /// Encode a binary file into an image file
    Encode {
        /// Binary file to be encoded
        bin: String,
        /// Path to the resulting image file
        dst: String,
    },
    /// Decode from an image
    Decode {
        /// Image file to be decoded
        img: String,
        /// Path to extract the original binary into
        dst: String,
    },
    /// Print color chart
    ColorChart,
}

/// Calculate appropriate dimensions of an image.
fn dimensions(filesize: u64) -> Result<(u32, u32), PicbinError> {
    let width = {
        // u64::MAX can't be handled in u32::MAX * u32::MAX
        if filesize > (u32::MAX as u64) * (u32::MAX as u64) {
            return Err(PicbinError::FileSizeTooLarge);
        }
        (filesize as f64).sqrt().ceil() as u32
    };
    let height = (filesize as f64 / (width as f64)).ceil() as u32;
    Ok((width, height))
}

/// Map a single byte to color.
fn byte_to_color(b: u8) -> image::Rgb<u8> {
    let b = b as u32;

    // hue in 360 degrees; 0 = red, 120 = lime and 240 = blue
    let hue = 360 * b / 256;
    // divide hue in 6 sections; red, yellow, lime, cyan, blue and fuchsia
    let hue_section_idx = hue / (360 / 6);
    // how far from the beginning of section
    let offset = b % 60;

    let secondary_inc = (256 * offset / 60) as u8;
    let secondary_dec = (255 - secondary_inc) as u8;
    let vals: [u8; 3] = match hue_section_idx {
        0 => [255, secondary_inc, 0],
        1 => [secondary_dec, 255, 0],
        2 => [0, 255, secondary_inc],
        3 => [0, secondary_dec, 255],
        4 => [secondary_inc, 0, 255],
        5 => [255, 0, secondary_dec],
        _ => unreachable!(),
    };
    image::Rgb(vals)
}

/// Encode the given content into an image.
fn encode_to_image(f: &mut fs::File) -> Result<image::RgbImage, PicbinError> {
    // decide the dimensions of an image based on file size
    let filesize = match f.metadata() {
        Ok(v) => v.len(),
        Err(e)  => return Err(PicbinError::IO(e)),
    };
    let (width, height) = dimensions(filesize)?;

    let mut img = image::RgbImage::new(width, height);
    let reader = BufReader::new(f);
    for (i, b) in reader.bytes().into_iter().enumerate() {
        // read each byte
        let b = match b {
            Ok(v) => v,
            Err(e) => return Err(PicbinError::IO(e)),
        };
        // ...and put it as a pixel on the image
        let px = byte_to_color(b);
        let i = i as u64;
        let x = (i % (width as u64)) as u32;
        let y = (i / (width as u64)) as u32;
        img.put_pixel(x, y, px)
    }
    Ok(img)
}

/// Decode the original binary content from the given image.
fn decode_from_image(image: &mut fs::File, out: &mut fs::File) -> Result<(), PicbinError>{
    // prepare inverse mapping, from color to byte
    let mut pixel_to_byte= HashMap::new();
    for i in u8::MIN..=u8::MAX {
        pixel_to_byte.insert(byte_to_color(i), i);
    }
    // prepare destination
    let mut writer = BufWriter::new(out);
    // prepare an image
    let reader = match image::io::Reader::new(BufReader::new(image)).with_guessed_format() {
        Ok(v) => v,
        Err(e) => return Err(PicbinError::IO(e)),
    };
    let img = match reader.decode() {
        Ok(v) => v,
        Err(e) => return Err(PicbinError::Imaging(e)),
    };
    // read each pixel and decode to a single byte
    for rgb in img.to_rgb8().pixels() {
        match pixel_to_byte.get(rgb) {
            Some(&b) => {
                match writer.write(&[b]) {
                    Ok(_) => {},
                    Err(e) => return Err(PicbinError::IO(e)),
                }
            },
            None => continue,
        }
    }
    Ok(())
}

/// Print color chart.
fn print_colorchart() {
    for i in u8::MIN..=u8::MAX {
        let color = byte_to_color(i);
        print!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2]);
        if i % 16 == 15 {
            // the last column
            println!();
        } else {
            print!(" ");
        }
    }
}

/// Main CLI program
fn cli() -> Result<(), PicbinError> {
    let cli = Args::parse();

    match &cli.command {

        Commands::Encode { bin, dst } => {
            if Path::exists(Path::new(dst)) && !cli.overwrite {
                return Err(PicbinError::DestinationExists(dst.to_string()))
            }
            let mut original_file = fs::File::open(&bin)?;
            let encoded = encode_to_image(&mut original_file)?;
            encoded.save(&dst)?;
        },

        Commands::Decode { img, dst } => {
            if Path::exists(Path::new(dst)) && !cli.overwrite {
                return Err(PicbinError::DestinationExists(dst.to_string()))
            }
            let mut encoded_file = fs::File::open(&img)?;
            let mut decoded_file = fs::File::create(&dst)?;
            decode_from_image(&mut encoded_file, &mut decoded_file)?;
        },

        Commands::ColorChart => print_colorchart(),
    };

    Ok(())
}

fn main() -> ExitCode {
    match cli() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Failed: {}", e);
            ExitCode::FAILURE
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::{dimensions, byte_to_color};
    use std::collections::HashSet;

    #[test]
    fn test_dimensions() {
        assert!(dimensions(0).is_ok());
        assert!(dimensions((u32::MAX as u64) * (u32::MAX as u64)).is_ok());
        assert!(dimensions((u32::MAX as u64) * (u32::MAX as u64) + 1).is_err());
    }

    #[test]
    fn unique_color_mapping() {
        let mut set = HashSet::new();
        for i in u8::MIN..=u8::MAX {
            set.insert(byte_to_color(i));
        }
        assert_eq!(set.len(), 256);
    }
}
