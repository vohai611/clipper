use arboard::Clipboard;
use druid::widget::{Button, Container, Flex, Label, LensWrap, List, ViewSwitcher};
use druid::{AppLauncher, PlatformError, Widget, WidgetExt, WindowDesc};
use druid::{Data, Lens};
use im::{vector, Vector};

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

    AppLauncher::with_window(main_window).launch(init_state)
}

fn ui_builder() -> impl Widget<AppState> {
    // The label text will be computed dynamically based on the current locale and count
    let label = Label::new("Clipboard list").padding(5.0).center();

    // Dynamically create a list of buttons, one for each clipboard.
    let list = LensWrap::new(
        List::new(|| {
            Container::new(
                Flex::column().with_child(
                    Label::dynamic(|item: &String, _env: &_| format!("Item: {}", item))
                        .expand_width()
                        .padding(5.0),
                ),
            )
        }),
        AppState::items,
    );

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
