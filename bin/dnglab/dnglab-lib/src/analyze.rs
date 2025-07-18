// SPDX-License-Identifier: LGPL-2.1
// Copyright 2021 Daniel Vogelbacher <daniel@chaospixel.com>

use clap::ArgMatches;
use log::debug;
use rawler::analyze::{
  analyze_file_structure, analyze_metadata, extract_full_pixels, extract_preview_pixels, extract_raw_pixels, extract_thumbnail_pixels, raw_as_pgm,
  raw_pixels_digest, rgb8_as_ppm8,
};
use rawler::analyze::{raw_as_ppm16, raw_to_srgb};
use rawler::decoders::RawDecodeParams;
use serde::Serialize;
use std::{
  io::{BufWriter, Write},
  path::PathBuf,
};

fn print_output<T: Serialize + ?Sized>(obj: &T, options: &ArgMatches) -> crate::Result<()> {
  if options.get_flag("yaml") {
    let yaml = serde_yaml::to_string(obj)?;
    println!("{}", yaml);
  } else {
    let json = serde_json::to_string_pretty(obj)?;
    println!("{}", json);
  }
  Ok(())
}

/// Analyze a given image
pub async fn analyze(options: &ArgMatches) -> crate::Result<()> {
  let in_file: &PathBuf = options.get_one("FILE").expect("FILE not available");

  debug!("Infile: {:?}", in_file);

  if options.get_flag("meta") {
    let analyze = analyze_metadata(PathBuf::from(in_file))?;
    print_output(&analyze, options)?;
  } else if options.get_flag("structure") {
    let analyze = analyze_file_structure(PathBuf::from(in_file))?;
    print_output(&analyze, options)?;
  } else if options.get_flag("raw_checksum") {
    let digest = raw_pixels_digest(PathBuf::from(in_file), &RawDecodeParams::default())?;
    println!("{}", hex::encode(digest));
  } else if options.get_flag("raw_pixel") {
    let mut image = extract_raw_pixels(PathBuf::from(in_file), &RawDecodeParams::default())?;
    if let rawler::RawImageData::Float(_) = &image.data {
      image.apply_scaling()?;
      image.data.force_integer();
    }
    if let rawler::RawImageData::Integer(samples) = &image.data {
      if image.cpp == 3 {
        dump_ppm16(image.width, image.height, &samples)?;
      } else {
        dump_pgm(image.width, image.height, &samples)?;
      }
    } else {
      unreachable!()
    }
  } else if options.get_flag("preview_pixel") {
    let preview = extract_preview_pixels(PathBuf::from(in_file), &RawDecodeParams::default())?;
    let rgb = preview.into_rgb8();
    dump_rgb8_ppm8(rgb.width() as usize, rgb.height() as usize, rgb.as_flat_samples().samples)?;
  } else if options.get_flag("thumbnail_pixel") {
    let thumbnail = extract_thumbnail_pixels(PathBuf::from(in_file), &RawDecodeParams::default())?;
    let rgb = thumbnail.into_rgb8();
    dump_rgb8_ppm8(rgb.width() as usize, rgb.height() as usize, rgb.as_flat_samples().samples)?;
  } else if options.get_flag("full_pixel") {
    let full = extract_full_pixels(PathBuf::from(in_file), &RawDecodeParams::default())?;
    let rgb = full.into_rgb8();
    dump_rgb8_ppm8(rgb.width() as usize, rgb.height() as usize, rgb.as_flat_samples().samples)?;
  } else if options.get_flag("srgb") {
    // TODO: only RGB output is supported
    let image = raw_to_srgb(PathBuf::from(in_file), &RawDecodeParams::default())?;
    dump_ppm16(image.width() as usize, image.height() as usize, image.as_flat_samples_u16().unwrap().samples)?;
  }
  Ok(())
}

/// Write image to STDOUT as PGM
fn dump_pgm(width: usize, height: usize, buf: &[u16]) -> std::io::Result<()> {
  let out = std::io::stdout();
  let mut writer = BufWriter::new(out);
  raw_as_pgm(width, height, buf, &mut writer)?;
  writer.flush()
}

/// Write image to STDOUT as PGM
fn dump_ppm16(width: usize, height: usize, buf: &[u16]) -> std::io::Result<()> {
  let out = std::io::stdout();
  let mut writer = BufWriter::new(out);
  raw_as_ppm16(width, height, buf, &mut writer)?;
  writer.flush()
}

/// Write image to STDOUT as PGM
fn dump_rgb8_ppm8(width: usize, height: usize, buf: &[u8]) -> std::io::Result<()> {
  let out = std::io::stdout();
  let mut writer = BufWriter::new(out);
  rgb8_as_ppm8(width, height, buf, &mut writer)?;
  writer.flush()
}
