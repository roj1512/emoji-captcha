use actix_web::get;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;
use image::imageops;
use image::Rgba;
use image::RgbaImage;
use rand::seq::SliceRandom;
use std::env::var;
use std::fs::read_to_string;
use std::io;
use std::io::Cursor;
use std::path::Path;

static mut EMOJIS: Vec<(String, String)> = vec![];
static POSITIONS: [(i64, i64); 6] = [
    (50, 50),
    (260, 50),
    (470, 50),
    (50, 260),
    (266, 260),
    (470, 260),
];

#[actix_web::main]
async fn main() -> io::Result<()> {
    let codes = read_to_string("allowed-emojis.txt")?
        .split("\n")
        .filter(|c| !c.is_empty() && c.to_lowercase() != "2705" && c.to_lowercase() != "274c")
        .map(|c| c.to_string())
        .collect::<Vec<_>>();
    let files = codes
        .iter()
        .map(|c| {
            Path::new("./emoji-data-ios/img-apple-160")
                .join(c.to_lowercase().to_owned() + ".png")
                .into_os_string()
                .into_string()
                .unwrap()
        })
        .collect::<Vec<_>>();
    for (i, c) in codes.iter().enumerate() {
        unsafe {
            EMOJIS.push((c.to_owned(), files[i].to_owned()));
        }
    }
    HttpServer::new(|| App::new().service(handle_request))
        .bind((
            match var("SERVER_ADDR") {
                Ok(addr) => addr,
                Err(_) => "127.0.0.1".to_string(),
            },
            match var("SERVER_PORT") {
                Ok(port) => port.parse::<u16>().unwrap(),
                Err(_) => 8080,
            },
        ))?
        .run()
        .await
}

#[get("/")]
async fn handle_request() -> impl Responder {
    let mut emojis = unsafe {
        EMOJIS
            .choose_multiple(&mut rand::thread_rng(), 15)
            .cloned()
            .collect::<Vec<_>>()
    };
    let correct_emojis = &emojis.clone()[0..6];
    let mut image = RgbaImage::from_fn(680, 470, |_, _| Rgba([0, 0, 0, 255]));
    imageops::vertical_gradient(
        &mut image,
        &Rgba([20, 20, 20, 255]),
        &Rgba([25, 25, 25, 255]),
    );
    for i in 0..6 {
        let path = &emojis.get(i).unwrap().1;
        let mut emoji = image::open(path).unwrap().into_rgba8();
        let (x, y) = POSITIONS[i];
        imageops::overlay(&mut image, &mut emoji, x, y)
    }
    emojis.shuffle(&mut rand::thread_rng());
    let mut body: Vec<u8> = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut body), image::ImageOutputFormat::Png)
        .unwrap();
    HttpResponse::Ok()
        .content_type(mime::IMAGE_PNG)
        .body(body)
        .customize()
        .insert_header((
            "x-emojis",
            emojis
                .iter()
                .map(|e| e.0.as_str())
                .collect::<Vec<_>>()
                .join(";")
                .as_str(),
        ))
        .insert_header((
            "x-correct-emojis",
            correct_emojis
                .iter()
                .map(|e| e.0.as_str())
                .collect::<Vec<_>>()
                .join(";")
                .as_str(),
        ))
}
