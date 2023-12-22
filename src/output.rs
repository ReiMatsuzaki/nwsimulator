use std::sync::Mutex;
use lazy_static::lazy_static;

#[derive(Clone, Copy)]
pub enum Level {
    Byte,
    Frame,
    Transport,
}

lazy_static! {
    static ref OUTPUT_LEVEL: Mutex<Level> = Mutex::new(Level::Byte);
}

pub fn set_level(level: Level) {
    let mut output_level = OUTPUT_LEVEL.lock().unwrap();
    *output_level = level;
}

// pub fn get_level() -> Level {
//     let output_level = OUTPUT_LEVEL.lock().unwrap();
//     *output_level
// }

pub fn is_byte_level() -> bool {
    let output_level = OUTPUT_LEVEL.lock().unwrap();
    match *output_level {
        Level::Byte => true,
        _ => false,
    }
}

pub fn is_frame_level() -> bool {
    let output_level = OUTPUT_LEVEL.lock().unwrap();
    match *output_level {
        Level::Byte => false,
        Level::Frame => true,
        Level::Transport => false,
    }
}

pub fn is_transport_level() -> bool {
    let output_level = OUTPUT_LEVEL.lock().unwrap();
    match *output_level {
        Level::Byte => false,
        Level::Frame => false,
        Level::Transport => true,
    }
}