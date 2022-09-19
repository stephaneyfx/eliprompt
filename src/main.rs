// Copyright (C) 2020 Stephane Raux. Distributed under the zlib license.

#![deny(warnings)]

use clap::Parser;
use eliprompt::{Block, Config, Environment};
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
use thiserror::Error;

/// Generates shell prompt
#[derive(Clone, Debug, Parser)]
#[clap(author, version, about)]
enum Command {
    Prompt(PromptCommand),
    StartTimer(StartTimerCommand),
    StopTimer(StopTimerCommand),
    Install(InstallCommand),
    /// Prints default configuration
    PrintDefaultConfig,
}

/// Prints prompt
#[derive(Clone, Debug, Parser)]
struct PromptCommand {
    /// Working directory or current working directory if not specified.
    #[clap(long)]
    pwd: Option<PathBuf>,
    /// Application state as returned from a previous run
    #[clap(long, default_value_t)]
    state: State,
    /// Prints errors and duration of the prompt generation
    #[clap(long)]
    test: bool,
    /// Path to the configuration file
    #[clap(long = "config")]
    config_path: Option<PathBuf>,
    /// Uses alternative prompt
    #[clap(long)]
    alternative_prompt: bool,
    /// Shell to generate prompt for
    #[clap(long, default_value_t)]
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
#[derive(Clone, Debug, Parser)]
struct StartTimerCommand {
    /// Application state as returned from a previous run
    #[clap(long, default_value_t)]
    state: State,
}

/// Stops timer and prints new state to stdout
#[derive(Clone, Debug, Parser)]
struct StopTimerCommand {
    /// Application state as returned from a previous run
    #[clap(long)]
    state: State,
    /// Exit code of the timed command
    #[clap(long)]
    exit_code: i32,
}

/// Generates configuration for the given shell
///
/// The output should be `eval`'ed in the appropriate shell configuration file. For zsh, it is
/// `.zshrc`.
#[derive(Clone, Debug, Parser)]
struct InstallCommand {
    /// Shell to install prompt for
    #[clap(long)]
    shell: ShellType,
}

const APP_NAME: &str = env!("CARGO_PKG_NAME");

static DEFAULT_CONFIG_PATH: Lazy<Option<PathBuf>> = Lazy::new(|| {
    let mut path = dirs::config_dir()?;
    path.extend(&[APP_NAME, "config.json"]);
    Some(path)
});

fn run() -> Result<(), AppError> {
    let cmd = Command::parse();
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
        println!(
            "\nPrompt generation took {}",
            humantime::format_duration(elapsed)
        );
    }
    Ok(())
}

fn print_or_fallback<S: Shell>(shell: &mut S, cmd: &PromptCommand) -> Result<(), AppError> {
    let config = match (&cmd.config_path, &*DEFAULT_CONFIG_PATH) {
        (Some(path), _) => read_config(path),
        (_, Some(path)) => match read_config(path) {
            Ok(config) => Ok(config),
            Err(AppError::ReadingConfigFailed(e)) if e.kind() == io::ErrorKind::NotFound => {
                Ok(Config::default_pretty())
            }
            e => e,
        },
        _ => Ok(Config::default_pretty()),
    }?;
    match print_prompt(shell, &config, cmd) {
        Ok(()) => Ok(()),
        Err(e) if cmd.test => Err(e),
        Err(e) => {
            let _ = print_fallback_prompt(shell);
            Err(e)
        }
    }
}

fn print_prompt<S: Shell>(
    shell: &mut S,
    config: &Config,
    cmd: &PromptCommand,
) -> Result<(), AppError> {
    let (sender, receiver) = sync_channel(1);
    let blocks = thread::spawn({
        let config = config.clone();
        let cmd = cmd.clone();
        move || {
            let blocks = make_prompt(
                &config,
                cmd.pwd.as_deref(),
                cmd.alternative_prompt,
                &cmd.state,
            );
            drop(sender);
            blocks
        }
    });
    let blocks = match receiver.recv_timeout(config.timeout) {
        Ok(()) | Err(RecvTimeoutError::Disconnected) => blocks
            .join()
            .unwrap_or(Err(AppError::PromptGenerationPanicked)),
        Err(RecvTimeoutError::Timeout) => Err(AppError::PromptGenerationTimedOut),
    }?;
    show_prompt(shell, blocks)
}

fn show_prompt<S: Shell>(shell: &mut S, blocks: Vec<Block>) -> Result<(), AppError> {
    let style = blocks
        .into_iter()
        .try_fold(ansi_term::Style::new(), |style, block| {
            let s = block.render();
            let style_diff = style.infix(*s.style_ref());
            shell.write_color_escape(&style_diff)?;
            write!(shell, "{}", &*s)?;
            Ok(*s.style_ref())
        })
        .map_err(AppError::Print)?;
    shell
        .write_color_escape(style.suffix())
        .map_err(AppError::Print)?;
    Ok(())
}

fn make_prompt(
    config: &Config,
    working_dir: Option<&Path>,
    alternative_prompt: bool,
    state: &State,
) -> Result<Vec<Block>, AppError> {
    let exit_code = state.prev_exit_code;
    let environment = match working_dir {
        Some(p) => Environment::new(p),
        None => Environment::current(),
    }?;
    let environment = environment.with_prev_exit_code(exit_code);
    let environment = match state.prev_cmd_duration {
        CmdDuration::Elapsed(d) => environment.with_prev_cmd_duration(d),
        _ => environment,
    };
    let environment = environment.force_alternative_prompt(alternative_prompt);
    Ok(config.produce(&environment))
}

fn print_fallback_prompt<S: Shell>(shell: &mut S) -> Result<(), AppError> {
    let blocks = eliprompt::fallback_prompt().produce(&Environment::current()?);
    show_prompt(shell, blocks)
}

fn start_timer(cmd: StartTimerCommand) {
    let state = State {
        prev_cmd_duration: CmdDuration::StartedAt(Clock::new().elapsed()),
        prev_exit_code: cmd.state.prev_exit_code,
    };
    print_state(&state);
}

fn stop_timer(cmd: StopTimerCommand) {
    let duration = match cmd.state.prev_cmd_duration {
        CmdDuration::StartedAt(start) => {
            let end = start.max(Clock::new().elapsed());
            CmdDuration::Elapsed(end - start)
        }
        CmdDuration::Unknown | CmdDuration::Elapsed(_) => CmdDuration::Unknown,
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
            .with_alphabet(bs58::Alphabet::BITCOIN)
            .into_string();
    println!("{}", state_str);
}

fn read_config(path: &Path) -> Result<Config, AppError> {
    serde_json::from_slice(&fs::read(path).map_err(AppError::ReadingConfigFailed)?)
        .map_err(AppError::BadConfig)
}

fn install(cmd: InstallCommand) -> Result<(), AppError> {
    let program = "eliprompt";
    match cmd.shell {
        ShellType::Generic => Err(AppError::CannotInstallGenericShell),
        ShellType::Zsh => install_zsh(program),
    }
}

fn install_zsh(program: &str) -> Result<(), AppError> {
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
    let config = config.replace("ELIPROMPT_EXE", program);
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
                .with_alphabet(bs58::Alphabet::BITCOIN)
                .into_string();
        f.write_str(&state_str)
    }
}

impl FromStr for State {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let state_str = s.trim();
        if state_str.is_empty() {
            return Ok(Default::default());
        }
        let state_bytes = bs58::decode(state_str)
            .with_alphabet(bs58::Alphabet::BITCOIN)
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
        buf.iter().copied().try_fold(0, |len, b| {
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

fn print_default_config() {
    println!(
        "{}",
        serde_json::to_string_pretty(&Config::default_pretty()).unwrap()
    );
}
