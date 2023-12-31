use arboard::Clipboard;
use arboard::ImageData;
use chrono::offset::Utc;
use chrono::DateTime;
use druid::widget::prelude::*;
use druid::widget::Controller;
use druid::widget::{Button, Container, Flex, Label, List, ViewSwitcher};
use druid::Color;
use druid::ImageBuf;
use druid::Point;
use druid::WindowId;
use druid::{AppLauncher, PlatformError, Widget, WidgetExt, WindowDesc};
use druid::{Data, Lens};
use druid_shell::WindowLevel;
use im::{vector, Vector};
use image::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;

#[derive(Clone, Data, Lens)]
struct AppState {
    items: Vector<Clip>,
    hover: Option<Arc<WindowId>>,
}

#[derive(Clone, Data, Lens, Debug)]
struct Clip {
    item: ClipType,
    hover: Option<Arc<WindowId>>,
    initilzie: String,
}

impl PartialEq for Clip {
    fn eq(&self, other: &Self) -> bool {
        match (&self.item, &other.item) {
            (ClipType::Text(x), ClipType::Text(y)) => x == y,
            (ClipType::Img(x), ClipType::Img(y)) => x == y,
            _ => false,
        }
    }
}

#[derive(Clone, Data, PartialEq, Debug)]
enum ClipType {
    Text(String),
    Img(String),
}

impl Clip {
    fn is_img(&self) -> bool {
        match self.item {
            ClipType::Text(_) => false,
            ClipType::Img(_) => true,
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
        let time_stampt = &self.initilzie;
        match &self.item {
            ClipType::Text(t) => {
                let trimmed_text = t.get(0..20).unwrap_or(&t);
                write!(f, " {time_stampt}: {trimmed_text}")
            }
            ClipType::Img(_) => {
                write!(f, "{time_stampt}:")
            }
        }
    }
}

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder())
        .window_size((600.0, 400.0))
        .title("Clipboard viewer");
    //let data = 0_u32;

    let init_state = AppState {
        items: vector![],
        hover: None,
    };

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

            let now: DateTime<Utc> = SystemTime::now().into();
            let now: String = now.format("[%H:%M] ").to_string();
            match (clip_text, clip_img) {
                (Ok(stri), Err(_)) => {
                    // ignore if text already in the list
                    let new = Clip {
                        item: ClipType::Text(stri.to_owned()),
                        hover: None,
                        initilzie: now,
                    };
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
                    let new = Clip {
                        item: ClipType::Img(k.clone()),
                        hover: None,
                        initilzie: now,
                    };

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

                    let _ = match &data.item {
                        ClipType::Text(text) => clipboard.set_text(text.clone()),
                        ClipType::Img(img) => clipboard.set_image(file_to_img(&img)),
                    };
                }))
                .with_child(
                    Label::dynamic(|item: &Clip, _env: &_| format!("{}", item))
                        .controller(LabelController),
                )
                .with_child(ViewSwitcher::new(
                    |data: &Clip, _env| data.is_img(),
                    |selector: &bool, data: &Clip, _env| {
                        if *selector {
                            Box::new(druid::widget::Image::new({
                                match &data.item {
                                    ClipType::Text(_) => ImageBuf::empty(),
                                    ClipType::Img(img) => {
                                        let img_path = "/tmp/".to_owned() + &img + ".png";
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
    .scroll()
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
    .scroll()
}

struct LabelController;

/// This controler create/remove tooltip for each label
impl Controller<Clip, Label<Clip>> for LabelController {
    fn event(
        &mut self,
        _child: &mut Label<Clip>,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut Clip,
        env: &Env,
    ) {
        match event {
            Event::MouseMove(mouse) => {
                if ctx.is_hot() {
                    if data.hover.is_none() {
                        let id = ctx.widget_id();
                        println!("Moving to: {:?}", id);
                        dbg!(mouse);
                        let druid::Point { x: p_x, y: p_y } = ctx.to_screen(mouse.pos);
                        let current_screen_pos = Point::new(p_x + 20.0, p_y + 20.0);

                        let id = ctx.new_sub_window(
                            druid::WindowConfig::default()
                                .set_position(current_screen_pos)
                                .set_level(WindowLevel::Tooltip(ctx.window().clone()))
                                .show_titlebar(false),
                            ViewSwitcher::new(
                                |data: &Clip, _env| data.is_img(),
                                |selector: &bool, data: &Clip, _env| {
                                    if *selector {
                                        Box::new(druid::widget::Image::new({
                                            match &data.item {
                                                ClipType::Text(_) => ImageBuf::empty(),
                                                ClipType::Img(img) => {
                                                    let img_path =
                                                        "/tmp/".to_owned() + &img + ".png";
                                                    ImageBuf::from_file(img_path).unwrap()
                                                }
                                            }
                                        }))
                                    } else {
                                        let value = match &data.item {
                                            ClipType::Text(t) => t.clone(),
                                            ClipType::Img(t) => t.clone(),
                                        };
                                        Box::new(Label::new(value))
                                    }
                                },
                            ),
                            data.clone(),
                            env.clone(),
                        );
                        data.hover = Some(id.into());
                    }
                } else {
                    let id = WindowId::from(*data.hover.clone().unwrap());
                    ctx.submit_command(druid::commands::CLOSE_WINDOW.to(id));
                    data.hover = None;
                    println!("Out");
                }
            }
            _ => {}
        }
    }
}

// mouse move
// out -> out : nothing
// out -> in : hover  = true,
// in -> in2 : last hove = false, newhove = true
// in -> out : hover = false
