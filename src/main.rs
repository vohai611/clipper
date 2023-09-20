use arboard::Clipboard;
use druid::keyboard_types::Key;
use druid::widget::prelude::*;
use druid::widget::Controller;
use druid::widget::{Button, Container, Flex, Label, LensWrap, List, ViewSwitcher};
use druid::{AppLauncher, PlatformError, Selector, Widget, WidgetExt, WindowDesc};
use druid::{Data, Lens};
use im::{vector, Vector};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone, Data, Lens)]
struct AppState {
    items: Vector<String>,
}

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder())
        .window_size((600.0, 400.0))
        .title("Clipboard viewer");
    //let data = 0_u32;

    let init_state = AppState {
        items: vector!["".to_string()],
    };

    let launcher = AppLauncher::with_window(main_window);
    let x = launcher.get_external_handle();

    thread::spawn(move || call_clipboard(x));
    launcher.launch(init_state)
}

fn call_clipboard(x: druid::ExtEventSink) {
    loop {
        x.add_idle_callback(|data: &mut AppState| {
            let clipboard = Clipboard::new().unwrap().get_text().unwrap();
            let items = &mut data.items;
            // ignore if text already in the list
            if !items.contains(&clipboard) {
                items.push_back(clipboard);
            }
        });
        thread::sleep(Duration::from_secs_f64(0.2));
    }
}

fn ui_builder() -> impl Widget<AppState> {
    // The label text will be computed dynamically based on the current locale and count
    let label = Label::new("Clipboard list").padding(5.0).center();

    // Dynamically create a list of buttons, one for each clipboard.
    let list = Label::dynamic(|data: &AppState, _env: &_| {
        let val = data
            .items
            .iter()
            .fold("".to_string(), |acc, item| acc + "\n" + &item);
        format!("Item: {}", val)
    })
    .expand_width();

    let button2 = Button::new("Store clipboard")
        .on_click(|_ctx, clip: &mut AppState, _env| {
            let mut clipboard = Clipboard::new().unwrap();
            println!("Clipboard: {}", clipboard.get_text().unwrap());
            clip.items.push_back(clipboard.get_text().unwrap());
        })
        .padding(5.0);

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
            .with_child(button2)
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
