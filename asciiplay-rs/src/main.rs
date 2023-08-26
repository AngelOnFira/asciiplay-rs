extern crate ffmpeg_next as ffmpeg;

use console::Term;
use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg::util::frame::video::Video;
use rascii_art::{render_image_to, RenderOptions};
use std::thread;

const CHARSET: &[&str] = &[
    " ", ".", "'", "`", "^", "\"", ",", ":", ";", "I", "l", "!", "i", ">", "<", "~", "+", "_", "-",
    "?", "]", "[", "}", "{", "1", "(", ")", "|", "\\", "/", "t", "f", "j", "r", "x", "n", "u", "v",
    "c", "z", "X", "Y", "U", "J", "C", "L", "Q", "0", "O", "Z", "m", "w", "q", "p", "d", "b", "k",
    "h", "a", "o", "*", "#", "M", "W", "&", "8", "%", "B", "@", "$",
];

fn main() -> Result<(), ffmpeg::Error> {
    // Load an mp4 video file
    ffmpeg::init().unwrap();

    // Initialize the terminal
    let term = Term::stdout();

    // Load the video
    if let Ok(mut ictx) = input(&"shrek.mp4".to_string()) {
        let input = ictx
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg::Error::StreamNotFound)?;
        let video_stream_index = input.index();

        let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;
        let mut decoder = context_decoder.decoder().video()?;

        let mut scaler = Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::RGB24,
            decoder.width(),
            decoder.height(),
            Flags::BILINEAR,
        )?;

        let mut receive_and_process_decoded_frames =
            |decoder: &mut ffmpeg::decoder::Video| -> Result<(), ffmpeg::Error> {
                let mut decoded = Video::empty();
                while decoder.receive_frame(&mut decoded).is_ok() {
                    let mut rgb_frame = Video::empty();
                    scaler.run(&decoded, &mut rgb_frame)?;

                    let mut buffer = String::new();

                    let header = format!("P6\n{} {}\n255\n", rgb_frame.width(), rgb_frame.height());
                    let header = header.as_bytes();
                    let data: &[u8] = rgb_frame.data(0);

                    let buf: &[u8] = &[header, data].concat();

                    render_image_to(
                        &image::load_from_memory_with_format(buf, image::ImageFormat::Pnm).unwrap(),
                        &mut buffer,
                        &RenderOptions::new()
                            .width(100)
                            .colored(true)
                            .charset(CHARSET),
                    )
                    .unwrap();

                    term.clear_screen().unwrap();
                    term.write_line(&buffer).unwrap();

                    thread::sleep(std::time::Duration::from_millis(10));
                }
                Ok(())
            };

        for (stream, packet) in ictx.packets() {
            if stream.index() == video_stream_index {
                decoder.send_packet(&packet)?;
                receive_and_process_decoded_frames(&mut decoder)?;
            }
        }
        decoder.send_eof()?;
        receive_and_process_decoded_frames(&mut decoder)?;
    }

    Ok(())
}
