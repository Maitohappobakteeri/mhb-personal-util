use std::{
    error::Error,
    io::BufReader,
    thread,
    time::{Duration, SystemTime},
};

use clap::{arg, command, Parser};
use cursive::{
    event,
    theme::{BorderStyle, Theme},
    view::{Nameable, SizeConstraint},
    views::{Layer, LinearLayout, Panel, ResizedView, StackView, TextView, ThemedView},
    Cursive,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Seconds that should pass between alarms
    #[arg(short, long, default_value_t = 60 * 30)]
    seconds_between_alarms: u64,
}

#[derive(Clone)]
struct State {
    start: Option<SystemTime>,
    limit: u64,
}

const INITIAL_STATE: State = State {
    start: None,
    limit: 60 * 30,
};

fn get_default_theme() -> Theme {
    let mut theme = cursive::theme::load_default();
    theme.palette[cursive::theme::PaletteColor::Background] =
        cursive::theme::Color::Light(cursive::theme::BaseColor::Red);
    theme.palette[cursive::theme::PaletteColor::View] =
        cursive::theme::Color::Light(cursive::theme::BaseColor::Red);
    theme.palette[cursive::theme::PaletteColor::Shadow] =
        cursive::theme::Color::Dark(cursive::theme::BaseColor::Black);
    theme.borders = BorderStyle::Outset;
    theme
}

fn get_header_theme() -> Theme {
    let mut theme = Theme::default();
    theme.palette[cursive::theme::PaletteColor::Background] =
        cursive::theme::Color::Light(cursive::theme::BaseColor::Red);
    theme.palette[cursive::theme::PaletteColor::View] =
        cursive::theme::Color::Light(cursive::theme::BaseColor::Red);
    theme
}

fn get_alarm_theme() -> Theme {
    let mut theme = Theme::default();
    theme.palette[cursive::theme::PaletteColor::Background] =
        cursive::theme::Color::Light(cursive::theme::BaseColor::Magenta);
    theme.palette[cursive::theme::PaletteColor::View] =
        cursive::theme::Color::Light(cursive::theme::BaseColor::Magenta);
    theme
}

fn timer_layer() -> Layer<LinearLayout> {
    let mut l = LinearLayout::vertical();
    l.add_child(TextView::new("-0:0:0").with_name("timer-text"));
    Layer::new(l)
}

fn reset_timer(siv: &mut Cursive) -> Result<(), Box<dyn Error>> {
    let mut state: State = siv
        .user_data::<State>()
        .as_deref()
        .unwrap_or(&INITIAL_STATE)
        .clone();
    state.start = Some(SystemTime::now());
    siv.set_user_data(state);
    siv.set_theme(get_default_theme());

    Ok(())
}

fn update_timer(siv: &mut Cursive) -> Result<(), Box<dyn Error>> {
    let mut state: State = siv
        .user_data::<State>()
        .as_deref()
        .unwrap_or(&INITIAL_STATE)
        .clone();

    let now = SystemTime::now();
    let start = state.start.unwrap_or(now);
    let since_start = now.duration_since(start)?;

    let remaining = state.limit - since_start.as_secs().clamp(0, state.limit);
    siv.call_on_name("timer-text", |view: &mut TextView| {
        view.set_content(format!("{:?}s", remaining))
    });

    state.start = Some(start);

    if remaining == 0 {
        siv.set_theme(get_alarm_theme());
        let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let file = std::fs::File::open("woomy.mp3").unwrap();
        let beep1 = stream_handle.play_once(BufReader::new(file)).unwrap();
        beep1.set_volume(1.0);
        beep1.detach();
        thread::sleep(Duration::from_millis(200));
    }

    siv.set_user_data(state);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut siv = cursive::default();

    siv.set_autorefresh(true);
    siv.set_fps(1);

    let mut state: State = INITIAL_STATE.clone();
    state.limit = args.seconds_between_alarms;
    siv.set_user_data(state);

    siv.set_theme(get_default_theme());

    let mut main = LinearLayout::vertical();

    let title = TextView::new("Task switch alarm");
    let mut header = LinearLayout::horizontal();
    header.add_child(title);
    let header = ThemedView::new(get_header_theme(), Layer::new(Panel::new(header)));
    let header = ResizedView::new(SizeConstraint::Full, SizeConstraint::Free, header);
    main.add_child(header);

    let mut stack = StackView::new();
    stack.add_fullscreen_layer(timer_layer());
    main.add_child(stack.with_name("main-content"));

    let main = ResizedView::new(SizeConstraint::Full, SizeConstraint::Full, main);
    let main = Panel::new(main);
    siv.add_fullscreen_layer(main);

    siv.add_global_callback(event::Key::Enter, move |s| {
        reset_timer(s).unwrap();
    });
    siv.add_global_callback(event::Event::Refresh, move |s| {
        update_timer(s).unwrap();
    });

    siv.run();

    Ok(())
}
