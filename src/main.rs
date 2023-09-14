use arboard::Clipboard;
use druid::widget::{Button, Container, Flex, Label};
use druid::{AppLauncher, PlatformError, Widget, WidgetExt, WindowDesc};
use druid::{Data, Lens};
use im::{vector, Vector};

#[derive(Clone, Data, Lens)]
struct TodoList {
    items: Vector<String>,
}

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder())
        .window_size((600.0, 400.0))
        .title("My first Druid App");
    //let data = 0_u32;

    let clip = TodoList {
        items: vector!["a".to_string()],
    };

    AppLauncher::with_window(main_window).launch(clip)
}

fn ui_builder() -> impl Widget<TodoList> {
    // The label text will be computed dynamically based on the current locale and count
    let label = Label::new("abc").padding(5.0).center();

    let button2 = Button::new("Store clipboard")
        .on_click(|_ctx, clip: &mut TodoList, _env| {
            let mut clipboard = Clipboard::new().unwrap();
            println!("Clipboard: {}", clipboard.get_text().unwrap());
            clip.items.push_back(clipboard.get_text().unwrap());
        })
        .padding(5.0);

    let button = Button::new("View")
        .on_click(|_ctx, clip: &mut TodoList, _env| {
            println!("Clipboard: {:?}", clip.items);
        })
        .padding(5.0);

    Container::new(
        Flex::column()
            .with_child(label)
            .with_child(button2)
            .with_child(button),
    )
}
