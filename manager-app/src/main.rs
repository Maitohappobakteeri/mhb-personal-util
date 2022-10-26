use cursive::{
    event,
    theme::{BaseColor, Color, ColorType, Style},
    view::{Finder, Nameable},
    views::{LinearLayout, TextView},
    Cursive,
};

use std::io::BufReader;
use std::{thread, time::Duration};

struct State {
    list_index: i64,
    list_size: i64,
}

const INITIAL_STATE: State = State {
    list_index: 0,
    list_size: 2,
};

fn main() {
    let mut theme = cursive::theme::load_default();

    theme.palette[cursive::theme::PaletteColor::Background] =
        cursive::theme::Color::Dark(cursive::theme::BaseColor::Red);

    let mut siv = cursive::default();

    siv.set_user_data(INITIAL_STATE);
    siv.set_theme(theme);

    let mut list = LinearLayout::vertical();
    list.add_child(TextView::new("asd!").with_name("list-item-0"));
    list.add_child(TextView::new("lol!").with_name("list-item-1"));
    siv.add_layer(list.with_name("command-list"));

    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let play_sound = move || {
        let file = std::fs::File::open("./woomy.mp3").unwrap();
        let beep1 = stream_handle.play_once(BufReader::new(file)).unwrap();
        beep1.set_volume(1.0);
        thread::sleep(Duration::from_millis(200));
        beep1.detach();
    };

    let select_prev = |c: &mut Cursive| {
        c.call_on_name("command-list", |view: &mut LinearLayout| {
            let mut s = Style::default();
            s.color.back = ColorType::from(Color::Dark(BaseColor::Red));
            for i in 0..view.len() {
                view.call_on_name(
                    &("list-item-".to_string() + &i.to_string()),
                    |t: &mut TextView| t.set_style(s),
                );
            }
        });
        let state: &State = c
            .user_data::<State>()
            .map(|s| &*s)
            .unwrap_or(&INITIAL_STATE);
        let i = (state.list_index - 1).clamp(0, state.list_size - 1);
        let size = state.list_size;
        c.set_user_data(State {
            list_index: i,
            list_size: size,
        });
        c.call_on_name(
            &("list-item-".to_string() + &i.to_string()),
            |view: &mut TextView| {
                let mut s = Style::default();
                s.color.back = ColorType::from(Color::Dark(BaseColor::Blue));
                view.set_style(s);
            },
        );
    };

    let select_next = |c: &mut Cursive| {
        c.call_on_name("command-list", |view: &mut LinearLayout| {
            let mut s = Style::default();
            s.color.back = ColorType::from(Color::Dark(BaseColor::Red));
            for i in 0..view.len() {
                view.call_on_name(
                    &("list-item-".to_string() + &i.to_string()),
                    |t: &mut TextView| t.set_style(s),
                );
            }
        });
        let state: &State = c
            .user_data::<State>()
            .map(|s| &*s)
            .unwrap_or(&INITIAL_STATE);
        let i = (state.list_index + 1).clamp(0, state.list_size - 1);
        let size = state.list_size;
        c.set_user_data(State {
            list_index: i,
            list_size: size,
        });
        c.call_on_name(
            &("list-item-".to_string() + &i.to_string()),
            |view: &mut TextView| {
                let mut s = Style::default();
                s.color.back = ColorType::from(Color::Dark(BaseColor::Blue));
                view.set_style(s);
            },
        );
    };

    siv.add_global_callback(event::Key::Up, move |s| select_prev(s));
    siv.add_global_callback(event::Key::Down, move |s| select_next(s));
    siv.add_global_callback(event::Key::Enter, move |s| play_sound());

    siv.run();
}
