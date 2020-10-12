// Copyright (C) 2020 Stephane Raux. Distributed under the zlib license.

#![deny(warnings)]

use eliprompt::{Block, BlockProducer, Config, Environment, Style};
use moniclock::Clock;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    env,
    error::Error,
    fmt::{self, Display},
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    str::FromStr,
    sync::mpsc::{sync_channel, RecvTimeoutError},
    thread,
    time::{Duration, Instant},
};
use structopt::StructOpt;
use thiserror::Error;

/// Generates shell prompt
#[derive(Debug, StructOpt)]
#[structopt(author)]
enum Command {
    Prompt(PromptCommand),
    StartTimer(StartTimerCommand),
    StopTimer(StopTimerCommand),
    Install(InstallCommand),
    /// Prints default configuration
    PrintDefaultConfig,
}

/// Prints prompt
#[derive(Debug, StructOpt)]
struct PromptCommand {
    /// Working directory or current working directory if not specified.
    #[structopt(long)]
    pwd: Option<PathBuf>,
    /// Application state as returned from a previous run
    #[structopt(long, default_value)]
    state: State,
    /// Prints errors and duration of the prompt generation
    #[structopt(long)]
    test: bool,
    /// Path to the configuration file
    #[structopt(long = "config")]
    config_path: Option<PathBuf>,
    /// Uses symbol fallback
    #[structopt(long)]
    symbol_fallback: bool,
    /// Shell to generate prompt for
    #[structopt(long, default_value)]
    shell: ShellType,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
enum ShellType {
    Generic,
    Zsh,
}

impl Default for ShellType {
    fn default() -> Self {
        ShellType::Generic
    }
}

/// Starts timer and prints new state to stdout
#[derive(Debug, StructOpt)]
struct StartTimerCommand {
    /// Application state as returned from a previous run
    #[structopt(long, default_value)]
    state: State,
}

/// Stops timer and prints new state to stdout
#[derive(Debug, StructOpt)]
struct StopTimerCommand {
    /// Application state as returned from a previous run
    #[structopt(long)]
    state: State,
    /// Exit code of the timed command
    #[structopt(long)]
    exit_code: i32,
}

/// Generates configuration for the given shell
///
/// The output should be `eval`'ed in the appropriate shell configuration file. For zsh, it is
/// `.zshrc`.
#[derive(Debug, StructOpt)]
struct InstallCommand {
    /// Shell to install prompt for
    #[structopt(long)]
    shell: ShellType,
}

const APP_NAME: &str = env!("CARGO_PKG_NAME");

static DEFAULT_CONFIG_PATH: Lazy<Option<PathBuf>> = Lazy::new(|| {
    let mut path = dirs::config_dir()?;
    path.extend(&[APP_NAME, "config.json"]);
    Some(path)
});

fn run() -> Result<(), AppError> {
    let cmd = Command::from_args();
    match cmd {
        Command::Prompt(cmd) => generate_prompt(cmd)?,
        Command::StartTimer(cmd) => start_timer(cmd),
        Command::StopTimer(cmd) => stop_timer(cmd),
        Command::Install(cmd) => install(cmd)?,
        Command::PrintDefaultConfig => print_default_config(),
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

fn generate_prompt(cmd: PromptCommand) -> Result<(), AppError> {
    let t0 = Instant::now();
    let mut buffer = Vec::<u8>::new();
    match cmd.shell {
        ShellType::Generic => print_or_fallback(&mut GenericShell(&mut buffer), &cmd)?,
        ShellType::Zsh => print_or_fallback(&mut Zsh(&mut buffer), &cmd)?,
    }
    println!();
    io::stdout().write_all(&buffer).map_err(AppError::Print)?;
    let elapsed = t0.elapsed();
    if cmd.test {
        println!("\nPrompt generation took {}", humantime::format_duration(elapsed));
    }
    Ok(())
}

fn print_or_fallback<S: Shell>(shell: &mut S, cmd: &PromptCommand) -> Result<(), AppError> {
    match print_prompt(shell, cmd) {
        Ok(()) => Ok(()),
        Err(e) if cmd.test => Err(e),
        Err(e) => {
            let _ = print_fallback_prompt(shell);
            Err(e)
        }
    }
}

fn print_prompt<S: Shell>(shell: &mut S, cmd: &PromptCommand) -> Result<(), AppError> {
    let config = match (&cmd.config_path, &*DEFAULT_CONFIG_PATH) {
        (Some(path), _) => read_config(path),
        (_, Some(path)) if path.exists() => read_config(path),
        _ => Ok(default_pretty_config()),
    }?;
    let symbol_fallback = cmd.symbol_fallback;
    let timeout = config.timeout();
    let state = cmd.state.clone();
    let working_dir = cmd.pwd.clone();
    let (sender, receiver) = sync_channel(1);
    let blocks = thread::spawn(move || {
        let blocks = make_prompt(
            &config,
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
    working_dir: Option<&Path>,
    symbol_fallback: bool,
    state: State,
) -> Result<Vec<Block>, AppError> {
    let exit_code = state.prev_exit_code;
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

fn start_timer(_: StartTimerCommand) {
    let state = State {
        prev_cmd_duration: CmdDuration::StartedAt(Clock::new().elapsed()),
        prev_exit_code: 0,
    };
    print_state(&state);
}

fn stop_timer(cmd: StopTimerCommand) {
    let duration = match cmd.state.prev_cmd_duration {
        CmdDuration::StartedAt(start) => {
            let end = start.max(Clock::new().elapsed());
            CmdDuration::Elapsed(end - start)
        }
        CmdDuration::Unknown
        | CmdDuration::Elapsed(_) => CmdDuration::Unknown,
    };
    let state = State {
        prev_exit_code: cmd.exit_code,
        prev_cmd_duration: duration,
    };
    print_state(&state);
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

fn install(cmd: InstallCommand) -> Result<(), AppError> {
    let program_path = env::current_exe().map_err(AppError::GettingProgramPathFailed)?;
    let program_path = program_path.to_str().ok_or(AppError::ProgramPathNotUnicode)?;
    match cmd.shell {
        ShellType::Generic => Err(AppError::CannotInstallGenericShell),
        ShellType::Zsh => install_zsh(program_path),
    }
}

fn install_zsh(program_path: &str) -> Result<(), AppError> {
    let config = r####"
eliprompt_precmd() {
    prev_status=$?
    ELIPROMPT_STATE=$(ELIPROMPT_EXE stop-timer --state "$ELIPROMPT_STATE" --exit-code $prev_status)
    PROMPT=$(ELIPROMPT_EXE prompt --state "$ELIPROMPT_STATE" --shell zsh)
}

eliprompt_preexec() {
    ELIPROMPT_STATE=$(ELIPROMPT_EXE start-timer --state "$ELIPROMPT_STATE")
}

[[ -v precmd_functions ]] || precmd_functions=()
[[ ${precmd_functions[(ie)eliprompt_precmd]} -le ${#precmd_functions} ]] || precmd_functions+=(eliprompt_precmd)

[[ -v preexec_functions ]] || preexec_functions=()
[[ ${preexec_functions[(ie)eliprompt_preexec]} -le ${#preexec_functions} ]] || preexec_functions+=(eliprompt_preexec)
"####;
    let config = config.replace("ELIPROMPT_EXE", program_path);
    println!("{}", config);
    Ok(())
}

#[derive(Debug, Error)]
enum AppError {
    #[error("Configuration file is invalid")]
    BadConfig(#[source] serde_json::Error),
    #[error("Failed to read configuration file")]
    ReadingConfigFailed(#[source] io::Error),
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
    #[error("Failed to get the path to this program")]
    GettingProgramPathFailed(#[source] io::Error),
    #[error("The path to this program is not Unicode")]
    ProgramPathNotUnicode,
    #[error("Installation is not possible for generic shell")]
    CannotInstallGenericShell,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct State {
    prev_exit_code: i32,
    prev_cmd_duration: CmdDuration,
}

impl Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state_str =
            bs58::encode(serde_json::to_string(self).expect("Serializing state cannot fail"))
                .with_prepared_alphabet(bs58::Alphabet::BITCOIN)
                .into_string();
        f.write_str(&state_str)
    }
}

impl FromStr for State {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let state_str = s.trim();
        if state_str.is_empty() { return Ok(Default::default()) }
        let state_bytes = bs58::decode(state_str)
            .with_prepared_alphabet(bs58::Alphabet::BITCOIN)
            .into_vec()
            .map_err(AppError::DecodingStateFailed)?;
        Ok(serde_json::from_slice(&state_bytes).map_err(AppError::ParsingStateFailed)?)
    }
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
