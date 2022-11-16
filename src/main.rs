use inotify::{EventMask, Inotify, WatchMask};
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const BASE_PATH: &'static str = "/sys/class/backlight/intel_backlight";

fn read(file: &mut File) -> Result<u32, Box<dyn Error>> {
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    file.seek(SeekFrom::Start(0))?;
    Ok(content.trim().parse()?)
}

struct Brightness<'a> {
    max: f64,
    file: File,
    current: u32,
    path: &'a str,
}

impl<'a> Brightness<'a> {
    fn new(p: &'a str) -> Result<Brightness, Box<dyn Error>> {
        let brightness_path = Path::new(p).join("brightness");
        let mut file = OpenOptions::new().read(true).open(&brightness_path)?;
        let max: f64 = fs::read_to_string(Path::new(p).join("max_brightness"))?
            .trim()
            .parse()?;
        let current = read(&mut file)?;
        Ok(Brightness {
            max,
            file,
            current,
            path: p,
        })
    }

    fn out(&mut self) {
        match read(&mut self.file) {
            Ok(v) => {
               let res: f64 = (v as f64) / self.max * 100.0;  
               self.current = res.ceil() as u32;
               println!("{}", self.current);
            },
            Err(_) => {println!("ERR");}
        }
    } 
}

fn main() {
    let mut brightness = Brightness::new(BASE_PATH).expect("INIT ERROR");
    brightness.out();
    let mut inotify = Inotify::init().expect("INIT ERROR");
    let brightness_path = Path::new(brightness.path).join("brightness");
    inotify
        .add_watch(&brightness_path, WatchMask::MODIFY)
        .expect("ERROR ADDING WATCH");

    let mut buffer = [0u8; 1024];

    loop {
        let events = inotify
            .read_events_blocking(&mut buffer)
            .expect("ERROR READING EVENTS");
        for event in events {
            if event.mask.contains(EventMask::MODIFY) {
               brightness.out(); 
            }
        }
    }
}
