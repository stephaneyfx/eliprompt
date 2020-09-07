// Copyright (C) 2020 Stephane Raux. Distributed under the MIT license.

//! Command-line application to generate a shell prompt.
//!
//! # Font
//! The default configuration uses symbols from [Nerd Fonts](https://www.nerdfonts.com/) and expects
//! one of them to be installed.
//!
//! # Shell support
//! Only zsh is supported. Please open an issue if support for another shell is desired.
//!
//! # Installation
//! ```sh
//! cargo install eliprompt
//! ```
//!
//! Make sure `eliprompt` is in your `PATH` and add the following to `.zshrc`:
//! ```sh
//! eval "$(eliprompt install --zsh)"
//! ```
//!
//! # Overview
//! The prompt is made of blocks. Each block contains the text to display as well as the style
//! (foreground and background colors).
//!
//! # Configuration
//! The configuration is stored in `~/.config/eliprompt/config.json`. It consists of a JSON object
//! of type `Config`. `Config` and the other JSON types involved are detailed below.
//!
//! ## `BlockProducer`
//! JSON object with a single field among:
//! - `Elapsed`:
//!   - Type: `Elapsed`
//! - `ExitCode`:
//!   - Type: `ExitCode`
//! - `GitHead`:
//!   - Type: `GitHead`
//! - `GitPath`:
//!   - Type: `GitPath`
//! - `WorkingDirectory`:
//!   - Type: `WorkingDirectory`
//! - `Or`:
//!   - Type: List of `BlockProducer`s
//!   - Producers are tried in order until one produces blocks.
//!
//! ## `Color`
//! String with a CSS color name (e.g. `"red"`) or a CSS sRGB color (e.g. `"#ff1000"`).
//!
//! ## `Config`
//! Root configuration object. JSON object with the following fields:
//! - `block_producers` [optional]:
//!   - Type: List of `BlockProducer`s
//!   - The sequence of all produced blocks is what makes up the prompt.
//! - `prompt` [optional]:
//!   - Type: `Symbol`
//!   - Text to display before the cursor where the next command will be entered. Separated from the
//! cursor by a space.
//! - `prompt_style` [optional]:
//!   - Type: `Style`
//!   - Style to display the prompt when the previous command exited with zero.
//! - `prompt_error_style` [optional]:
//!   - Type: `Style`
//!   - Style to display the prompt when the previous command exited with non-zero.
//! - `separator` [optional]:
//!   - Type: `Symbol`
//!   - Separator between block sequences from different producers.
//! - `separator_style` [optional]:
//!   - Type: `Style`
//! - `timeout` [optional]:
//!   - Type: `Duration`
//!   - Maximum duration to build the prompt. If it takes longer, a default prompt will be shown.
//!
//! ## `Duration`
//! String containing a duration with unit, e.g. `"3s"` for 3 seconds.
//!
//! ## `Elapsed`
//! Shows the duration of the previous command. JSON object with the following fields:
//! - `style` [optional]:
//!   - Type: `Style`
//! - `prefix` [optional]:
//!   - Type: `Symbol`
//!   - Text to display before the duration.
//! - `threshold` [optional]:
//!   - Type: `Duration`
//!   - The duration of a command is displayed if and only if it took longer than the threshold.
//!
//! ## `ExitCode`
//! Shows the exit code of the previous command if it was not zero. JSON object with the following
//! fields:
//! - `style` [optional]:
//!   - Type: `Style`
//! - `prefix` [optional]:
//!   - Type: `Symbol`
//!   - Text to display before the exit code.
//!
//! ## `GitHead`
//! Shows the current git branch. JSON object with the following fields:
//! - `style` [optional]:
//!   - Type: `Style`
//! - `prefix` [optional]:
//!   - Type: `Symbol`
//!   - Text to display before the git branch.
//!
//! ## `GitPath`
//! If the current working directory is in a git repository, it is shown relative to the root of the
//! repository. JSON object with the following fields:
//! - `style` [optional]:
//!   - Type: `Style`
//!
//! ## `Style`
//! JSON object with the following fields:
//! - `foreground` [optional]:
//!   - Type: `Color` or `null`
//! - `background` [optional]:
//!   - Type: `Color` or `null`
//!
//! ## `Symbol`
//! Text with optional fallback for terminals that do not handle fancy text. JSON object with the
//! following fields:
//! - `regular`:
//!   - Type: `String`
//! - `fallback`:
//!   - Type: `String` or `null`
//!   - Displayed in case of terminals that do not support fancy characters.
//!
//! ## `WorkingDirectory`
//! Shows the current working directory. JSON object with the following fields:
//! - `style` [optional]:
//!   - Type: `Style`
//! - `home_as_tilde` [optional]:
//!   - Type: `bool`
//!   - Indicates if the home directory should be displayed as a tilde.
//!
//! ## Example
//! ```json
//! {
//!     "block_producers": [
//!         {
//!             "Or": [
//!                 {
//!                     "GitPath": {
//!                         "style": {
//!                             "foreground": "limegreen"
//!                         }
//!                     }
//!                 },
//!                 {
//!                     "WorkingDirectory": {}
//!                 }
//!             ]
//!         },
//!         {
//!             "GitHead": {}
//!         }
//!     ],
//!     "prompt": {
//!         "regular": "\u2192",
//!         "fallback": ">"
//!     },
//!     "prompt_style": {
//!         "foreground": "dodgerblue"
//!     }
//! }
//! ```
//!
//! # Related projects
//! [starship](https://github.com/starship/starship) provides more blocks and supports more shells.

#![deny(warnings)]

use clap::{App, AppSettings, Arg, ArgGroup, ArgMatches, SubCommand};
use eliprompt::{Block, BlockProducer, Config, Environment, Style};
use moniclock::Clock;
use serde::{Deserialize, Serialize};
use std::{
    env,
    error::Error,
    fmt::Display,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::mpsc::{sync_channel, RecvTimeoutError},
    thread,
    time::{Duration, Instant},
};
use thiserror::Error;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

fn run() -> Result<(), AppError> {
    let mut default_config_path = dirs::config_dir().ok_or(AppError::GettingConfigDirFailed)?;
    default_config_path.extend(&[APP_NAME, "config.json"]);
    let config_help = format!(
        "Path to the configuration file (default: {})",
        default_config_path.display(),
    );
    let state_arg = Arg::with_name("state")
        .help("Application state as returned from a previous run")
        .long("state")
        .takes_value(true);
    let prev_exit_code_arg = Arg::with_name("prev-exit-code")
        .help("Exit code of the previous command")
        .long("prev-exit-code")
        .takes_value(true);
    let args = App::new(APP_NAME)
        .version(APP_VERSION)
        .author(APP_AUTHORS)
        .about("Generates shell prompts")
        .setting(AppSettings::SubcommandRequired)
        .subcommand(
            SubCommand::with_name("prompt")
                .about("Prints the prompt")
                .arg(prev_exit_code_arg.clone())
                .arg(
                    Arg::with_name("pwd")
                        .help("Working directory")
                        .long("pwd")
                        .takes_value(true)
                )
                .arg(state_arg.clone())
                .arg(
                    Arg::with_name("test")
                        .help("Prints errors and duration of the prompt generation")
                        .long("test")
                )
                .arg(
                    Arg::with_name("config")
                        .help(&config_help)
                        .long("config")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("symbol-fallback")
                        .help("Uses symbol fallback")
                        .long("symbol-fallback")
                )
                .arg(
                    Arg::with_name("zsh")
                        .help("Generates a prompt for zsh")
                        .long("zsh")
                )
        )
        .subcommand(
            SubCommand::with_name("store")
                .about("Stores state used to render the prompt. The new state is printed to \
                    stdout.")
                .arg(state_arg.clone().required(true))
                .arg(
                    Arg::with_name("start-timer")
                        .help("Starts timer")
                        .long("start-timer")
                )
                .arg(
                    Arg::with_name("stop-timer")
                        .help("Stops timer")
                        .long("stop-timer")
                        .conflicts_with("start-timer")
                )
                .arg(prev_exit_code_arg)
        )
        .subcommand(
            SubCommand::with_name("install")
                .about("Generates prompt configuration for the given shell. The output should be \
                    `eval`ed in the appropriate shell configuration file.")
                .arg(
                    Arg::with_name("zsh")
                        .help("Generates for zsh. `eval` the output in `.zshrc`")
                        .long("zsh")
                )
                .group(
                    ArgGroup::with_name("shell")
                        .args(&["zsh"])
                        .required(true)
                )
        )
        .subcommand(
            SubCommand::with_name("print-default-config")
                .about("Prints the default configuration.")
        )
        .get_matches();
    match args.subcommand() {
        ("prompt", args) => process_prompt(args, &default_config_path)?,
        ("store", Some(args)) => process_store(args)?,
        ("install", Some(args)) => process_install(args)?,
        ("print-default-config", _) => print_default_config(),
        _ => unreachable!(),
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        print_error(&e);
        std::process::exit(1);
    }
}

fn print_error(mut e: &dyn Error) {
    eprintln!("Error: {}", e);
    while let Some(cause) = e.source() {
        eprintln!("Because: {}", cause);
        e = cause;
    }
}

fn process_prompt(
    args: Option<&ArgMatches<'_>>,
    default_config_path: &Path,
) -> Result<(), AppError> {
    let t0 = Instant::now();
    let is_test = args.map_or(false, |args| args.is_present("test"));
    let mut buffer = Vec::<u8>::new();
    if args.map_or(false, |args| args.is_present("zsh")) {
        print_or_fallback(&mut Zsh(&mut buffer), args, default_config_path, is_test)?;
    } else {
        print_or_fallback(&mut GenericShell(&mut buffer), args, default_config_path, is_test)?;
    }
    println!();
    io::stdout().write_all(&buffer).map_err(AppError::Print)?;
    let elapsed = t0.elapsed();
    if is_test {
        println!("\nPrompt generation took {}", humantime::format_duration(elapsed));
    }
    Ok(())
}

fn print_or_fallback<S: Shell>(
    shell: &mut S,
    args: Option<&ArgMatches<'_>>,
    default_config_path: &Path,
    is_test: bool,
) -> Result<(), AppError> {
    match print_prompt(shell, args, default_config_path) {
        Ok(()) => Ok(()),
        Err(e) if is_test => Err(e),
        Err(e) => {
            let _ = print_fallback_prompt(shell);
            Err(e)
        }
    }
}

fn print_prompt<S: Shell>(
    shell: &mut S,
    args: Option<&ArgMatches<'_>>,
    default_config_path: &Path,
) -> Result<(), AppError> {
    let exit_code = args
        .and_then(|args| args.value_of("prev-exit-code"))
        .map(|code| code.parse().map_err(|_| AppError::BadExitCode(code.into())))
        .transpose()?;
    let working_dir = args.and_then(|args| args.value_of("pwd")).map(PathBuf::from);
    let state = args
        .and_then(|args| args.value_of("state"))
        .map(|state| read_state(&state))
        .transpose()?
        .unwrap_or_default();
    let config = match args.and_then(|args| args.value_of("config")) {
        Some(path) => read_config(Path::new(path)),
        None if default_config_path.exists() => read_config(default_config_path),
        None => Ok(default_pretty_config()),
    }?;
    let symbol_fallback = args.map_or(false, |args| args.is_present("symbol-fallback"));
    let timeout = config.timeout();
    let (sender, receiver) = sync_channel(1);
    let blocks = thread::spawn(move || {
        let blocks = make_prompt(
            &config,
            exit_code,
            working_dir.as_deref(),
            symbol_fallback,
            state,
        );
        drop(sender);
        blocks
    });
    let blocks = match receiver.recv_timeout(timeout) {
        Ok(()) | Err(RecvTimeoutError::Disconnected) => {
            blocks.join().unwrap_or(Err(AppError::PromptGenerationPanicked))
        }
        Err(RecvTimeoutError::Timeout) => Err(AppError::PromptGenerationTimedOut),
    }?;
    show_prompt(shell, blocks)
}

fn show_prompt<S: Shell>(shell: &mut S, blocks: Vec<Block>) -> Result<(), AppError> {
    let style = blocks
        .iter()
        .try_fold(ansi_term::Style::new(), |style, block| {
            let s = block.render();
            let style_diff = style.infix(*s.style_ref());
            shell.write_color_escape(&style_diff)?;
            write!(shell, "{}", &*s)?;
            Ok(*s.style_ref())
        })
        .map_err(AppError::Print)?;
    shell.write_color_escape(style.suffix()).map_err(AppError::Print)?;
    Ok(())
}

fn make_prompt(
    config: &Config,
    exit_code: Option<i32>,
    working_dir: Option<&Path>,
    symbol_fallback: bool,
    state: State,
) -> Result<Vec<Block>, AppError> {
    let exit_code = exit_code.unwrap_or(state.prev_exit_code);
    let environment = match working_dir {
        Some(p) => Environment::new(p),
        None => Environment::current(),
    }?;
    let environment = environment.with_prev_exit_code(exit_code);
    let environment = if symbol_fallback {
        environment.with_regular_symbols(false)
    } else {
        environment
    };
    let environment = match state.prev_cmd_duration {
        CmdDuration::Elapsed(d) => environment.with_prev_cmd_duration(d),
        _ => environment,
    };
    let blocks = config.produce(&environment)?;
    Ok(blocks)
}

fn print_fallback_prompt<S: Shell>(shell: &mut S) -> Result<(), AppError> {
    let blocks = Config::new().produce(&Environment::current()?)?;
    show_prompt(shell, blocks)
}

fn process_store(args: &ArgMatches<'_>) -> Result<(), AppError> {
    let state_str = args.value_of("state").expect("`state` is a mandatory argument").trim();
    let mut state = read_state(state_str)?;
    if args.is_present("start-timer") {
        start_timer(&mut state);
    } else if args.is_present("stop-timer") {
        stop_timer(&mut state);
    }
    if let Some(code) = args.value_of("prev-exit-code") {
        state.prev_exit_code = code.parse().map_err(|_| AppError::BadExitCode(code.into()))?;
    }
    print_state(&state);
    Ok(())
}

fn start_timer(state: &mut State) {
    state.prev_cmd_duration = CmdDuration::StartedAt(Clock::new().elapsed());
}

fn stop_timer(state: &mut State) {
    let start = match state.prev_cmd_duration {
        CmdDuration::StartedAt(start) => start,
        CmdDuration::Unknown
        | CmdDuration::Elapsed(_) => {
            state.prev_cmd_duration = CmdDuration::Unknown;
            return;
        }
    };
    let end = start.max(Clock::new().elapsed());
    state.prev_cmd_duration = CmdDuration::Elapsed(end - start);
}

fn read_state(state_str: &str) -> Result<State, AppError> {
    let state_str = state_str.trim();
    if state_str.is_empty() { return Ok(Default::default()) }
    let state_bytes = bs58::decode(state_str)
        .with_prepared_alphabet(bs58::Alphabet::BITCOIN)
        .into_vec()
        .map_err(AppError::DecodingStateFailed)?;
    Ok(serde_json::from_slice(&state_bytes).map_err(AppError::ParsingStateFailed)?)
}

fn print_state(state: &State) {
    let state_str =
        bs58::encode(serde_json::to_string(&state).expect("Serializing state cannot fail"))
            .with_prepared_alphabet(bs58::Alphabet::BITCOIN)
            .into_string();
    println!("{}", state_str);
}

fn read_config(path: &Path) -> Result<Config, AppError> {
    serde_json::from_slice(&fs::read(path).map_err(AppError::ReadingConfigFailed)?)
        .map_err(AppError::BadConfig)
}

fn process_install(args: &ArgMatches<'_>) -> Result<(), AppError> {
    let program_path = env::current_exe().map_err(AppError::GettingProgramPathFailed)?;
    let program_path = program_path.to_str().ok_or(AppError::ProgramPathNotUnicode)?;
    if args.is_present("zsh") {
        install_zsh(program_path);
    }
    Ok(())
}

fn install_zsh(program_path: &str) {
    let config = r####"
eliprompt_precmd() {
    prev_status=$?
    ELIPROMPT_STATE=$(ELIPROMPT_EXE store --state "$ELIPROMPT_STATE" --stop-timer --prev-exit-code $prev_status)
    PROMPT=$(ELIPROMPT_EXE prompt --state "$ELIPROMPT_STATE" --zsh)
}

eliprompt_preexec() {
    ELIPROMPT_STATE=$(ELIPROMPT_EXE store --state "$ELIPROMPT_STATE" --start-timer --prev-exit-code 0)
}

[[ -v precmd_functions ]] || precmd_functions=()
[[ ${precmd_functions[(ie)eliprompt_precmd]} -le ${#precmd_functions} ]] || precmd_functions+=(eliprompt_precmd)

[[ -v preexec_functions ]] || preexec_functions=()
[[ ${preexec_functions[(ie)eliprompt_preexec]} -le ${#preexec_functions} ]] || preexec_functions+=(eliprompt_preexec)
"####;
    let config = config.replace("ELIPROMPT_EXE", program_path);
    println!("{}", config);
}

#[derive(Debug, Error)]
enum AppError {
    #[error("Configuration file is invalid")]
    BadConfig(#[source] serde_json::Error),
    #[error("Failed to read configuration file")]
    ReadingConfigFailed(#[source] io::Error),
    #[error("Invalid exit code {0}")]
    BadExitCode(String),
    #[error("Failed to print prompt")]
    Print(#[source] io::Error),
    #[error("Error while building prompt")]
    Prompt(#[from] eliprompt::Error),
    #[error("Prompt generation panicked")]
    PromptGenerationPanicked,
    #[error("Prompt generation timed out")]
    PromptGenerationTimedOut,
    #[error("Failed to decode state")]
    DecodingStateFailed(#[source] bs58::decode::Error),
    #[error("Failed to parse state")]
    ParsingStateFailed(#[source] serde_json::Error),
    #[error("Failed to get the app configuration directory")]
    GettingConfigDirFailed,
    #[error("Failed to get the path to this program")]
    GettingProgramPathFailed(#[source] io::Error),
    #[error("The path to this program is not Unicode")]
    ProgramPathNotUnicode,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct State {
    prev_exit_code: i32,
    prev_cmd_duration: CmdDuration,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum CmdDuration {
    Unknown,
    StartedAt(Duration),
    Elapsed(Duration),
}

impl Default for CmdDuration {
    fn default() -> Self {
        Self::Unknown
    }
}

trait Shell: Write {
    fn write_color_escape<T: Display>(&mut self, x: T) -> io::Result<()>;
}

struct Zsh<W>(W);

impl<W: Write> Write for Zsh<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        buf
            .iter()
            .copied()
            .try_fold(0, |len, b| {
                match b {
                    b'%' => self.0.write_all(b"%%")?,
                    _ => self.0.write_all(&[b])?,
                }
                Ok(len + 1)
            })
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl<W: Write> Shell for Zsh<W> {
    fn write_color_escape<T: Display>(&mut self, x: T) -> io::Result<()> {
        write!(self.0, "%{{{}%}}", x)
    }
}

struct GenericShell<W>(W);

impl<W: Write> Write for GenericShell<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl<W: Write> Shell for GenericShell<W> {
    fn write_color_escape<T: Display>(&mut self, x: T) -> io::Result<()> {
        write!(self.0, "{}", x)
    }
}

fn default_pretty_config() -> Config {
    let producers = vec![
        BlockProducer::Or(vec![
            BlockProducer::GitPath(
                eliprompt::block::GitPath::new()
                    .with_style(Style::new().with_fg("limegreen".parse().unwrap())),
            ),
            BlockProducer::WorkingDirectory(
                eliprompt::block::WorkingDirectory::new()
                    .with_style(Style::new().with_fg("forestgreen".parse().unwrap())),
            ),
        ]),
        BlockProducer::GitHead(
            eliprompt::block::GitHead::new()
                .with_style(Style::new().with_fg("plum".parse().unwrap())),
        ),
        BlockProducer::Elapsed(
            eliprompt::block::Elapsed::new()
                .with_style(Style::new().with_fg("gold".parse().unwrap())),
        ),
        BlockProducer::ExitCode(
            eliprompt::block::ExitCode::new()
                .with_style(Style::new().with_fg("crimson".parse().unwrap())),
        ),
    ];
    Config::from_producers(producers)
        .with_prompt_style(Style::new().with_fg("dodgerblue".parse().unwrap()))
        .with_prompt_error_style(Style::new().with_fg("crimson".parse().unwrap()))
}

fn print_default_config() {
    println!("{}", serde_json::to_string_pretty(&default_pretty_config()).unwrap());
}
