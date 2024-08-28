use hidapi::{HidApi, HidDevice};
use image::codecs::jpeg::JpegEncoder;
use image::imageops;
use image::GenericImageView;
use std::process::Command;
use std::thread;
use std::time::Duration;

fn sleep(duration: u64) {
    thread::sleep(Duration::from_millis(duration));
}

const ACTIONS: [&str; 15] = [
    "xdg-open https://www.google.com",
    "xdg-open https://www.youtube.com",
    "xdg-open https://www.reddit.com",
    "xdg-open https://www.github.com",
    "xdg-open https://www.linkedin.com",
    "xdg-open https://www.twitter.com",
    "xdg-open https://www.instagram.com",
    "xdg-open https://www.facebook.com",
    "xdg-open https://www.amazon.com",
    "xdg-open https://www.ebay.com",
    "xdg-open https://www.netflix.com",
    "xdg-open https://music.youtube.com",
    "xdg-open https://www.twitch.com",
    "xdg-open https://teams.microsoft.com",
    "flatpak run com.raggesilver.BlackBox",
];

const ACTION_ICONS: [&[u8]; 15] = [
    include_bytes!("../images/google.png"),
    include_bytes!("../images/youtube.png"),
    include_bytes!("../images/reddit.png"),
    include_bytes!("../images/github.png"),
    include_bytes!("../images/linkedin.png"),
    include_bytes!("../images/twitter.png"),
    include_bytes!("../images/instagram.png"),
    include_bytes!("../images/facebook.png"),
    include_bytes!("../images/amazon.png"),
    include_bytes!("../images/ebay.png"),
    include_bytes!("../images/netflix.png"),
    include_bytes!("../images/youtube-music.png"),
    include_bytes!("../images/twitch.png"),
    include_bytes!("../images/teams.png"),
    include_bytes!("../images/terminal.png"),
];

fn get_device(vendor_id: u16, product_id: u16, usage: u16, usage_page: u16) -> Option<HidDevice> {
    let api = HidApi::new().expect("Failed to create HID API");
    for dev in api.device_list() {
        if dev.vendor_id() == vendor_id
            && dev.product_id() == product_id
            && dev.usage() == usage
            && dev.usage_page() == usage_page
        {
            if let Ok(device) = dev.open_device(&api) {
                return Some(device);
            }
        }
    }
    eprintln!("Device not found");
    return None;
}

fn set_brightness(device: &HidDevice, percentage: usize) {
    let mut buf = [0u8; 32];
    buf[0] = 0x03;
    buf[1] = 0x08;
    buf[2] = percentage as u8;
    device.send_feature_report(&mut buf).unwrap();
}

fn launch_app(action: &str) {
    let path: Vec<&str> = action.split_whitespace().collect();
    let child = Command::new(&path[0]).args(&path[1..]).spawn();

    if let Err(e) = child {
        eprintln!("Error: {:?}", e);
    }
}

fn get_pressed_button(buf: &[u8]) {
    if let Some(index) = buf.iter().position(|&x| x == 1) {
        launch_app(ACTIONS[index as usize]);
    }
}

fn read_states(device: &HidDevice) {
    let mut buf = [0u8; 32];
    buf[0] = 19;
    if let Ok(_size) = device.read(&mut buf) {
        get_pressed_button(&buf[4..19]);
    }
}

fn set_key_image(device: &HidDevice, key: u8) {
    let img_data = ACTION_ICONS[key as usize];
    let img = get_image_data(img_data);
    let mut page_number = 0;
    let mut bytes_remaining = img.len();
    while bytes_remaining > 0 {
        let this_length = std::cmp::min(bytes_remaining, 1024 - 8);
        let bytes_sent = page_number * (1024 - 8);
        let header = [
            0x02,
            0x07,
            key as u8,
            if this_length == bytes_remaining { 1 } else { 0 },
            (this_length & 0xFF) as u8,
            (this_length >> 8) as u8,
            (page_number & 0xFF) as u8,
            (page_number >> 8) as u8,
        ];
        let mut payload = Vec::with_capacity(1024);
        payload.extend_from_slice(&header);
        payload.extend_from_slice(&img[bytes_sent..bytes_sent + this_length]);
        payload.resize(1024, 0);
        device.write(&payload).unwrap();
        bytes_remaining -= this_length;
        page_number += 1;
    }
}

fn get_image_data(img_data: &[u8]) -> Vec<u8> {
    let img = image::load_from_memory(img_data).unwrap();
    let (width, height) = img.dimensions();
    let crop_size = std::cmp::min(width, height); // Take the smallest dimension
    let x_offset = (width - crop_size) / 2;
    let y_offset = (height - crop_size) / 2;
    let mut img = imageops::crop_imm(&img, x_offset, y_offset, crop_size, crop_size).to_image();
    img = imageops::rotate180(&img);
    img = imageops::resize(&img, 72, 72, imageops::FilterType::Nearest);
    let mut data = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut data, 100);
    encoder.encode_image(&img).unwrap();
    data
}

fn main() {
    if let Some(device) = get_device(0x0fd9, 0x0080, 0x0001, 0x000c) {
        set_brightness(&device, 100);
        for i in 0..15 {
            set_key_image(&device, i);
        }
        loop {
            read_states(&device);
            sleep(1);
        }
    };
}
