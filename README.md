[![Crate](https://img.shields.io/crates/v/eliprompt.svg?style=flat-square&logo=rust)](https://crates.io/crates/eliprompt)
[![Docs](https://img.shields.io/badge/docs-eliprompt-blue?style=flat-square)](https://docs.rs/eliprompt)
[![MIT license](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](https://opensource.org/licenses/MIT)

<!-- cargo-sync-readme start -->

Command-line application to generate a shell prompt.

# Shell support
Only zsh is supported. Please open an issue if support for another shell is desired.

# Installation
```sh
cargo install eliprompt
```

Make sure `eliprompt` is in your `PATH` and add the following to `.zshrc`:
```sh
eval "$(eliprompt install --zsh)"
```

# Overview
The prompt is made of blocks. Each block contains the text to display as well as the style
(foreground and background colors).

# Configuration
The configuration is stored in `~/.config/eliprompt/config.json`. It consists of a JSON object
of type `Config`. `Config` and the other JSON types involved are detailed below.

## `BlockProducer`
JSON object with a single field among:
- `Elapsed`:
  - Type: `Elapsed`
- `ExitCode`:
  - Type: `ExitCode`
- `GitHead`:
  - Type: `GitHead`
- `GitPath`:
  - Type: `GitPath`
- `WorkingDirectory`:
  - Type: `WorkingDirectory`
- `Or`:
  - Type: List of `BlockProducer`s
  - Producers are tried in order until one produces blocks.

## `Color`
String with a CSS color name (e.g. `"red"`) or a CSS sRGB color (e.g. `"#ff1000"`).

## `Config`
Root configuration object. JSON object with the following fields:
- `block_producers` [optional]:
  - Type: List of `BlockProducer`s
  - The sequence of all produced blocks is what makes up the prompt.
- `prompt` [optional]:
  - Type: `Symbol`
  - Text to display before the cursor where the next command will be entered. Separated from the
cursor by a space.
- `prompt_style` [optional]:
  - Type: `Style`
  - Style to display the prompt when the previous command exited with zero.
- `prompt_error_style` [optional]:
  - Type: `Style`
  - Style to display the prompt when the previous command exited with non-zero.
- `separator` [optional]:
  - Type: `Symbol`
  - Separator between block sequences from different producers.
- `separator_style` [optional]:
  - Type: `Style`
- `timeout` [optional]:
  - Type: `Duration`
  - Maximum duration to build the prompt. If it takes longer, a default prompt will be shown.

## `Duration`
String containing a duration with unit, e.g. `"3s"` for 3 seconds.

## `Elapsed`
Shows the duration of the previous command. JSON object with the following fields:
- `style` [optional]:
  - Type: `Style`
- `prefix` [optional]:
  - Type: `Symbol`
  - Text to display before the duration.
- `threshold` [optional]:
  - Type: `Duration`
  - The duration of a command is displayed if and only if it took longer than the threshold.

## `ExitCode`
Shows the exit code of the previous command if it was not zero. JSON object with the following
fields:
- `style` [optional]:
  - Type: `Style`
- `prefix` [optional]:
  - Type: `Symbol`
  - Text to display before the exit code.

## `GitHead`
Shows the current git branch. JSON object with the following fields:
- `style` [optional]:
  - Type: `Style`
- `prefix` [optional]:
  - Type: `Symbol`
  - Text to display before the git branch.

## `GitPath`
If the current working directory is in a git repository, it is shown relative to the root of the
repository. JSON object with the following fields:
- `style` [optional]:
  - Type: `Style`

## `Style`
JSON object with the following fields:
- `foreground` [optional]:
  - Type: `Color` or `null`
- `background` [optional]:
  - Type: `Color` or `null`

## `Symbol`
Text with optional fallback for terminals that do not handle fancy text. JSON object with the
following fields:
- `regular`:
  - Type: `String`
- `fallback`:
  - Type: `String` or `null`
  - Displayed in case of terminals that do not support fancy characters.

## `WorkingDirectory`
Shows the current working directory. JSON object with the following fields:
- `style` [optional]:
  - Type: `Style`
- `home_as_tilde` [optional]:
  - Type: `bool`
  - Indicates if the home directory should be displayed as a tilde.

## Example
```json
{
    "block_producers": [
        {
            "Or": [
                {
                    "GitPath": {
                        "style": {
                            "foreground": "limegreen"
                        }
                    }
                },
                {
                    "WorkingDirectory": {}
                }
            ]
        },
        {
            "GitHead": {}
        }
    ],
    "prompt": {
        "regular": "\u2192",
        "fallback": ">"
    },
    "prompt_style": {
        "foreground": "dodgerblue"
    }
}
```

# Related projects
[starship](https://github.com/starship/starship) provides more blocks and supports more shells.

<!-- cargo-sync-readme end -->
