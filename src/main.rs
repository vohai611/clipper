use arboard::Clipboard;
use druid::widget::{Button, Container, Flex, Label, LensWrap, List, ViewSwitcher};
use druid::{AppLauncher, PlatformError, Widget, WidgetExt, WindowDesc};
use druid::{Data, Lens};
use im::{vector, Vector};
use std::sync::{Arc, Mutex};

#[derive(Clone, Data, Lens, Debug)]
struct AppState {
    items: Arc<Mutex<Vector<String>>>,
}

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder())
        .window_size((600.0, 400.0))
        .title("Clipboard viewer");
    //let data = 0_u32;

    let init_state = AppState {
        items: Arc::new(Mutex::new(vector!["".to_string()])),
    };

    AppLauncher::with_window(main_window).launch(init_state)
}

fn ui_builder() -> impl Widget<AppState> {
    // The label text will be computed dynamically based on the current locale and count
    let label = Label::new("Clipboard list").padding(5.0).center();

    // Dynamically create a list of buttons, one for each clipboard.
    let list = Container::new(LensWrap::new(
        List::new(|| {
            Label::dynamic(|data, _env: &_| {
                let item = data.items.lock().unwrap().clone();
                let mut x = String::new();
                item.iter().for_each(|i| x.push_str(i));
                format!("Item: {}", x)
            })
        }),
        AppState::items,
    ));

    let button2 = Button::new("Store clipboard")
        .on_click(|_ctx, clip: &mut AppState, _env| {
            let mut clipboard = Clipboard::new().unwrap();
            println!("Clipboard: {}", clipboard.get_text().unwrap());
            clip.items
                .lock()
                .unwrap()
                .push_back(clipboard.get_text().unwrap());
        })
        .padding(5.0);

    let button = Button::new("View")
        .on_click(|_ctx, clip: &mut AppState, _env| {
            println!("Clipboard: {:?}", clip.items);
        })
        .padding(5.0);

    let button3 = Button::new("Clear")
        .on_click(|_ctx, data: &mut AppState, _env| {
            data.items.lock().unwrap().remove(0);
        })
        .padding(5.0);

    Flex::column()
        .with_child(label)
        .with_child(button2)
        .with_child(button)
        .with_child(list)
        .with_child(button3)
}
