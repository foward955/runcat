#![allow(dead_code)]

use std::fs::File;
use std::io::BufReader;
use image::{AnimationDecoder, ImageFormat};
use image::codecs::gif::{GifDecoder};
use crate::util::current_exe_dir;

pub fn get_frames() {
    let gif_path = current_exe_dir().unwrap().join("cute.gif");
    let buf_reader = BufReader::new(File::open(gif_path).unwrap());

    let gif_decoder = GifDecoder::new(buf_reader).unwrap();

    let frames = gif_decoder.into_frames().collect_frames().unwrap();

    let out_dir = current_exe_dir().unwrap().join("temp");

    if !out_dir.exists() {
        std::fs::create_dir(out_dir.clone()).unwrap();
    }

    for (i, frame) in frames.iter().enumerate() {
        let file_path = out_dir.clone().join(format!("{}.png", i + 1));
        frame.clone().into_buffer().save_with_format(file_path, ImageFormat::Png).unwrap();
    }
}