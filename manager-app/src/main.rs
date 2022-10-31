use clap::{arg, command, ArgMatches, Command};
use cursive::{
    event,
    theme::{BaseColor, BorderStyle, Color, ColorType, Effect, Style, Theme},
    view::{Finder, Nameable, SizeConstraint},
    views::{Layer, LinearLayout, NamedView, Panel, ResizedView, StackView, TextView, ThemedView},
    Cursive,
};
use serde::Deserialize;
use std::{
    env::{self, args},
    error::Error,
    fs::File,
    io::BufReader,
};

#[derive(Clone)]
struct State {
    list_index: i64,
    list_size: i64,
    path: Vec<MHBUtilCommand>,
    command: Option<String>,
}

const INITIAL_STATE: State = State {
    list_index: 0,
    list_size: 2,
    path: vec![],
    command: None,
};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct MHBUtilCommand {
    name: String,
    description: String,

    command_to_execute: Option<String>,
    sub_commands: Option<Vec<MHBUtilCommand>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct MHBUtilConfig {
    commands: Vec<MHBUtilCommand>,
}

fn open_config_file() -> Result<File, Box<dyn Error>> {
    let mut dir = env::current_exe()?;
    dir.pop();
    dir.push("config.json");
    let file = File::open(dir);

    if file.is_ok() {
        return Ok(file.unwrap());
    }

    Ok(File::open("test/config.json")?)
}

fn load_config() -> Result<&'static MHBUtilConfig, Box<dyn Error>> {
    let f = open_config_file()?;
    let reader = BufReader::new(f);
    let mhb_util: MHBUtilConfig = serde_json::from_reader(reader)?;
    Ok(Box::leak(Box::new(mhb_util)))
}

fn build_subcommand_args(command: &'static MHBUtilCommand) -> Command {
    let mut args_config = Command::new(command.name.as_str()).about(&command.description);

    if (command.sub_commands.is_some()) {
        args_config = args_config.subcommand_required(true);
    }

    match &command.sub_commands {
        Some(sub_coms) => {
            for sub_com in sub_coms {
                let sc = build_subcommand_args(sub_com);
                args_config = args_config.subcommand(sc);
            }
        }
        _ => (),
    }

    args_config
}

fn build_args(config: &'static MHBUtilConfig) -> Command {
    let mut args_config = command!()
        .about("MHB Utilities")
        .long_about("MHB Utilities. Run with no command for interactive.")
        .propagate_version(true)
        .subcommand_required(false);

    for command in &config.commands {
        args_config = args_config.subcommand(build_subcommand_args(command));
    }

    args_config
}

trait ListCommand {
    fn command_name(&self) -> String;
    fn command_description(&self) -> String;
}

impl ListCommand for MHBUtilCommand {
    fn command_name(&self) -> String {
        self.name.to_owned()
    }

    fn command_description(&self) -> String {
        self.description.to_owned()
    }
}

fn command_selection<T: ListCommand>(
    commands: &Vec<T>,
) -> cursive::views::NamedView<Layer<ResizedView<LinearLayout>>> {
    let mut list = LinearLayout::vertical();
    for (i, command) in commands.iter().enumerate() {
        let mut command_info = LinearLayout::vertical();

        let mut title = TextView::new(&command.command_name());
        let mut style = Style::default();
        style.color.front = ColorType::Color(cursive::theme::Color::Dark(
            cursive::theme::BaseColor::Magenta,
        ));
        style.effects = Effect::Underline | Effect::Bold;
        title.set_style(style);
        command_info.add_child(title);

        let mut description = TextView::new(&command.command_description());
        let mut style = Style::default();
        style.color.front = ColorType::Color(cursive::theme::Color::Light(
            cursive::theme::BaseColor::Magenta,
        ));
        description.set_style(style);
        command_info.add_child(description);

        let command_info =
            ResizedView::new(SizeConstraint::Full, SizeConstraint::Free, command_info);
        let command_info = Layer::new(command_info);
        let command_info = Panel::new(command_info);
        let command_info = ThemedView::new(get_default_theme(), command_info);

        list.add_child(command_info.with_name(String::from("list-item-") + &i.to_string()));
    }

    let list = ResizedView::new(SizeConstraint::Full, SizeConstraint::Full, list);
    let list = Layer::new(list);
    list.with_name("command-list")
}

fn clear_list_themes(list: &mut LinearLayout) {
    let mut s = Style::default();
    s.color.back = ColorType::from(Color::Dark(BaseColor::Red));
    for i in 0..list.len() {
        list.call_on_name(
            &("list-item-".to_string() + &i.to_string()),
            |view: &mut ThemedView<Panel<Layer<ResizedView<LinearLayout>>>>| {
                view.set_theme(get_list_theme());
            },
        );
    }
}

fn update_colors(app: &mut Cursive) {
    let state: State = app
        .user_data::<State>()
        .map(|s| s.clone())
        .unwrap_or(INITIAL_STATE);
    let i = state.list_index;

    app.call_on_name("main-content", |view: &mut StackView| {
        let view = view
            .get_mut(cursive::views::LayerPosition::FromFront(0))
            .unwrap()
            .as_any_mut()
            .downcast_mut::<NamedView<Layer<ResizedView<LinearLayout>>>>()
            .unwrap();

        view.call_on_name(
            "command-list",
            |view: &mut Layer<ResizedView<LinearLayout>>| {
                clear_list_themes(view.get_inner_mut().get_inner_mut())
            },
        );

        view.call_on_name(
            &("list-item-".to_string() + &i.to_string()),
            |view: &mut ThemedView<Panel<Layer<ResizedView<LinearLayout>>>>| {
                view.set_theme(get_highlight_theme());
            },
        );
    });
}

fn update_header(config: MHBUtilConfig, app: &mut Cursive) {
    let state: State = app
        .user_data::<State>()
        .map(|s| s.clone())
        .unwrap_or(INITIAL_STATE);
    app.call_on_name("backlink-text", |view: &mut TextView| {
        view.set_content(
            String::from(": ")
                + &state
                    .path
                    .iter()
                    .map(|c| c.command_name())
                    .collect::<Vec<_>>()
                    .join(" ➡  "),
        )
    });

    app.call_on_name("main-content", |view: &mut StackView| {
        view.add_fullscreen_layer(command_selection(
            &state
                .path
                .last()
                .map(|x| x.sub_commands.clone())
                .flatten()
                .unwrap_or(config.commands),
        ));
    });

    update_colors(app);
}

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

fn get_list_theme() -> Theme {
    let mut theme = cursive::theme::load_default();
    theme.palette[cursive::theme::PaletteColor::Background] =
        cursive::theme::Color::Light(cursive::theme::BaseColor::Red);
    theme.palette[cursive::theme::PaletteColor::View] =
        cursive::theme::Color::Light(cursive::theme::BaseColor::Red);
    theme.palette[cursive::theme::PaletteColor::Shadow] =
        cursive::theme::Color::Dark(cursive::theme::BaseColor::Black);
    theme.borders = BorderStyle::None;
    theme
}

fn get_highlight_theme() -> Theme {
    let mut theme = Theme::default();
    theme.palette[cursive::theme::PaletteColor::Background] =
        cursive::theme::Color::Light(cursive::theme::BaseColor::White);
    theme.palette[cursive::theme::PaletteColor::View] =
        cursive::theme::Color::Light(cursive::theme::BaseColor::White);
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

fn get_subcommand<'a>(
    command: &'a MHBUtilCommand,
    matches: &ArgMatches,
) -> Option<&'a MHBUtilCommand> {
    match matches.subcommand() {
        Some((c, sub_matches)) => {
            let l = command.sub_commands.as_ref();
            match l {
                Some(subs) => {
                    let m = subs.iter().find(|x| x.name == c).unwrap();
                    get_subcommand(m, sub_matches)
                }
                _ => None,
            }
        }
        _ => Some(command),
    }
}

fn get_command(config: &MHBUtilConfig, matches: ArgMatches) -> Option<&MHBUtilCommand> {
    match matches.subcommand() {
        Some((command, sub_matches)) => {
            let m = config.commands.iter().find(|x| x.name == command).unwrap();
            get_subcommand(m, &sub_matches)
        }
        _ => None,
    }
}

fn run_command(command: &str) -> Result<(), Box<dyn Error>> {
    let s = String::from(command);
    let mut dir = env::current_exe()?;
    dir.pop();
    dir.push(s);

    if dir.exists() {
        std::process::Command::new("bash")
            .arg(dir.to_str().unwrap())
            .status()?;
        return Ok(());
    } else {
        std::process::Command::new("bash").arg(command).status()?;
        return Ok(());
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = load_config()?;
    let matches = build_args(config).get_matches();

    let command = get_command(config, matches);
    match command {
        Some(c) => return run_command(&c.command_to_execute.clone().unwrap()),
        _ => println!("No command. Running in interactive mode."),
    }

    let mut siv = cursive::default();

    siv.set_user_data(INITIAL_STATE);
    siv.set_theme(get_default_theme());

    let mut main = LinearLayout::vertical();

    let title = TextView::new("MHB Utilities");
    let backlink = TextView::new(": -").with_name("backlink-text");
    let mut header = LinearLayout::horizontal();
    header.add_child(title);
    header.add_child(backlink);
    let header = ThemedView::new(get_header_theme(), Layer::new(Panel::new(header)));
    let header = ResizedView::new(SizeConstraint::Full, SizeConstraint::Free, header);
    main.add_child(header);

    let mut stack = StackView::new();
    stack.add_fullscreen_layer(command_selection(&config.commands));
    main.add_child(stack.with_name("main-content"));

    let main = ResizedView::new(SizeConstraint::Full, SizeConstraint::Full, main);
    let main = Panel::new(main);
    siv.add_fullscreen_layer(main);

    let select_prev = |c: &mut Cursive| {
        let state: State = c
            .user_data::<State>()
            .map(|s| s.clone())
            .unwrap_or(INITIAL_STATE);
        let i = (state.list_index - 1).clamp(0, state.list_size - 1);
        let size = state.list_size;
        let path = state.path.to_owned();
        c.set_user_data(State {
            list_index: i,
            list_size: size,
            path,
            command: None,
        });

        update_colors(c);
    };

    let select_next = |c: &mut Cursive| {
        let state: State = c
            .user_data::<State>()
            .map(|s| s.clone())
            .unwrap_or(INITIAL_STATE);
        let i = (state.list_index + 1).clamp(0, state.list_size - 1);
        let size = state.list_size;
        let path = state.path.to_owned();
        c.set_user_data(State {
            list_index: i,
            list_size: size,
            path,
            command: None,
        });

        update_colors(c);
    };

    let select_command = |c: &mut Cursive| {
        let state: State = c
            .user_data::<State>()
            .map(|s| s.clone())
            .unwrap_or(INITIAL_STATE);
        let i = state.list_index;
        let mut path = state.path.to_owned();

        let command_list = path
            .last()
            .map(|x| x.sub_commands.as_ref())
            .flatten()
            .unwrap_or(&config.commands);

        let selected = command_list.get(i as usize);
        let can_select = if path.last().is_some() {
            path.last().unwrap().sub_commands.is_some()
                && selected
                    .map(|x| x.sub_commands.as_ref())
                    .flatten()
                    .is_some()
        } else {
            true
        };

        if can_select {
            let size = command_list.clone().len() as i64;
            path.push(selected.unwrap().clone());
            c.set_user_data(State {
                list_index: i,
                list_size: size,
                path,
                command: None,
            });

            update_header(config.clone(), c);
        } else {
            if selected.is_some() && selected.unwrap().command_to_execute.is_some() {
                c.quit();
                c.set_user_data(State {
                    list_index: i,
                    list_size: 0,
                    path: vec![],
                    command: Some(
                        selected
                            .unwrap()
                            .command_to_execute
                            .as_ref()
                            .unwrap()
                            .to_string(),
                    ),
                });
            }
        }
    };

    let previous_command = |c: &mut Cursive| {
        let state: State = c
            .user_data::<State>()
            .map(|s| s.clone())
            .unwrap_or(INITIAL_STATE);
        let i = state.list_index;
        let mut path = state.path.to_owned();
        path.pop();
        let size = path
            .last()
            .map(|x| x.sub_commands.clone().map(|x| x.len()))
            .flatten()
            .unwrap_or(config.commands.len()) as i64;
        c.set_user_data(State {
            list_index: i,
            list_size: size,
            path,
            command: None,
        });

        update_header(config.clone(), c);
    };

    siv.add_global_callback(event::Key::Up, move |s| select_prev(s));
    siv.add_global_callback(event::Key::Down, move |s| select_next(s));
    siv.add_global_callback(event::Key::Enter, move |s| select_command(s));
    siv.add_global_callback(event::Key::Backspace, move |s| previous_command(s));

    update_colors(&mut siv);
    siv.run();
    let state: State = siv
        .user_data::<State>()
        .map(|s| s.clone())
        .unwrap_or(INITIAL_STATE);

    if state.command.is_some() {
        run_command(&state.command.unwrap())?;
    }

    Ok(())
}
