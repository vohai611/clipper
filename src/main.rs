use arboard::Clipboard;
use arboard::ImageData;
use chrono::offset::Utc;
use chrono::DateTime;
use druid::widget::prelude::*;
use druid::widget::Controller;
use druid::widget::{Button, Container, Flex, Label, List, ViewSwitcher};
use druid::Color;
use druid::ImageBuf;
use druid::{AppLauncher, PlatformError, Selector, Widget, WidgetExt, WindowDesc};
use druid::{Data, Lens};
use im::{vector, Vector};
use image::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;

#[derive(Clone, Data, Lens)]
struct AppState {
    items: Vector<Clip>,
}

#[derive(Clone, Data, PartialEq, Debug)]
enum Clip {
    Text(String),
    Img(String),
}

impl Clip {
    fn is_img(&self) -> bool {
        match self {
            Clip::Text(_) => false,
            Clip::Img(_) => true,
        }
    }
}

fn img_to_file(img: RgbaImage, file: &str) {
    let path = "/tmp/".to_owned() + file + ".png";
    let image = DynamicImage::ImageRgba8(img);
    image.save(path).unwrap();
}

fn file_to_img(file: &str) -> ImageData {
    let path = "/tmp/".to_owned() + file + ".png";
    let image = image::open(path).unwrap();
    ImageData {
        width: image.width() as usize,
        height: image.height() as usize,
        bytes: image.into_bytes().into(),
    }
}

impl std::fmt::Display for Clip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let now: DateTime<Utc> = SystemTime::now().into();
        let now: String = now.format("[%H:%M] ").to_string();
        match self {
            Clip::Text(t) => {
                let trimmed_text = t.get(0..20).unwrap_or(t);
                write!(f, " {now}: {trimmed_text}")
            }
            Clip::Img(_) => {
                write!(f, "{now}:")
            }
        }
    }
}

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder())
        .window_size((600.0, 400.0))
        .title("Clipboard viewer");
    //let data = 0_u32;

    let init_state = AppState { items: vector![] };

    let launcher = AppLauncher::with_window(main_window);
    let x = launcher.get_external_handle();

    thread::spawn(move || call_clipboard(x));
    launcher.launch(init_state)
}

fn call_clipboard(x: druid::ExtEventSink) {
    loop {
        x.add_idle_callback(|data: &mut AppState| {
            let mut clipboard = Clipboard::new().unwrap();
            let clip_text = clipboard.get_text();
            let clip_img = clipboard.get_image();
            let items = &mut data.items;

            match (clip_text, clip_img) {
                (Ok(stri), Err(_)) => {
                    // ignore if text already in the list
                    let new = Clip::Text(stri.to_owned());
                    if !items.contains(&new) {
                        items.push_back(new);
                    }
                }
                (Err(_), Ok(img)) => {
                    let mut hash = DefaultHasher::new();
                    let image: RgbaImage = ImageBuffer::from_raw(
                        img.width.try_into().unwrap(),
                        img.height.try_into().unwrap(),
                        img.bytes.into_owned(),
                    )
                    .unwrap();
                    image.hash(&mut hash);
                    let k = hash.finish().to_string();
                    let new = Clip::Img(k.clone());

                    if !items.contains(&new) {
                        items.push_back(new);
                        img_to_file(image, &k)
                    }
                }
                _ => {}
            }
        });
        thread::sleep(Duration::from_secs_f64(0.2));
    }
}

fn ui_builder() -> impl Widget<AppState> {
    // The label text will be computed dynamically based on the current locale and count
    let label = Label::new("Clipboard list").padding(5.0).center();

    // Dynamically create a list of buttons, one for each clipboard.
    let list = List::new(|| {
        druid::widget::SizedBox::new(
            Flex::row()
                .with_child(Button::new("copy").on_click(|_ctx, data: &mut Clip, _env| {
                    println!("{data}");
                    let mut clipboard = Clipboard::new().unwrap();

                    let _ = match data {
                        Clip::Text(text) => clipboard.set_text(text.clone()),
                        Clip::Img(img) => clipboard.set_image(file_to_img(img)),
                    };
                }))
                .with_child(Label::dynamic(|item: &Clip, _env: &_| format!("{}", item)))
                .with_child(ViewSwitcher::new(
                    |data: &Clip, _env| data.is_img(),
                    |selector: &bool, data: &Clip, _env| {
                        if *selector {
                            Box::new(druid::widget::Image::new({
                                match data {
                                    Clip::Text(_) => ImageBuf::empty(),
                                    Clip::Img(img) => {
                                        let img_path = "/tmp/".to_owned() + img + ".png";
                                        ImageBuf::from_file(img_path).unwrap()
                                    }
                                }
                            }))
                        } else {
                            Box::new(druid::widget::Image::new(ImageBuf::empty()))
                        }
                    },
                ))
                .expand_width()
                .padding(5.0),
        )
        .width(401.0)
        .height(40.0)
        .border(Color::WHITE, 1.0)
    })
    .lens(AppState::items);

    let button = Button::new("View")
        .on_click(|_ctx, clip: &mut AppState, _env| {
            println!("Clipboard: {:?}", clip.items);
        })
        .padding(5.0);

    let button3 = Button::new("Clear")
        .on_click(|_ctx, data: &mut AppState, _env| {
            data.items.remove(0);
        })
        .padding(5.0);

    Container::new(
        Flex::column()
            .with_child(label)
            .with_child(button)
            .with_child(button3)
            .with_child(list),
    )
}

struct LabelControler;

const a: Selector = Selector::new("label");

impl Controller<AppState, Label<AppState>> for LabelControler {
    fn event(
        &mut self,
        child: &mut Label<AppState>,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        match event {
            Event::Timer(token) => {
                ctx.submit_command(a);
                println!("heloo");
            }
            _ => child.event(ctx, event, data, env),
        }
    }
}
