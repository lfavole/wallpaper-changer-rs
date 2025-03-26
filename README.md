# wallpaper-changer-rs

[![Rust](https://img.shields.io/badge/language-Rust-blue?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Build status](https://img.shields.io/github/actions/workflow/status/lfavole/wallpaper-changer-rs/build.yml?branch=main)](https://github.com/lfavole/wallpaper-changer-rs/actions)
[![License](https://img.shields.io/badge/license-unlicense-green)](LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/lfavole/wallpaper-changer-rs)](https://github.com/lfavole/wallpaper-changer-rs/stargazers)
[![Last commit](https://img.shields.io/github/last-commit/lfavole/wallpaper-changer-rs)](https://github.com/lfavole/wallpaper-changer-rs/commits/main)

## Overview

**wallpaper-changer-rs** is a lightweight wallpaper changer written in Rust. It can use local pictures or fetch pictures from Unsplash to automatically change your desktop wallpaper.

## Features

- Change wallpaper using local images.
- Fetch and set wallpapers from Unsplash.
- Lightweight and fast.
- Simple configuration.

## Installation

Windows:
```bat
REM Download the program in C:\Users\...\AppData\Local\wallpaper-changer-rs
mkdir C:\Users\%USERNAME%\AppData\Local\wallpaper-changer-rs
curl https://lfavole.github.io/wallpaper-changer-rs/wallpaper_changer --output-dir C:\Users\%USERNAME%\AppData\Local\wallpaper-changer-rs\wallpaper_changer.exe

REM Register as a scheduled task
C:\Users\%USERNAME%\AppData\Local\wallpaper-changer-rs\wallpaper_changer.exe register
```

Linux:
```sh
# Download the program in ~/.local/share/wallpaper-changer-rs
mkdir -p ~/.local/share/wallpaper-changer-rs
curl https://lfavole.github.io/wallpaper-changer-rs/wallpaper_changer --output-dir ~/.local/share/wallpaper-changer-rs
chmod +x ~/.local/share/wallpaper-changer-rs/wallpaper_changer

# Register as a scheduled task
~/.local/share/wallpaper-changer-rs/wallpaper_changer register
```

## Development

Make sure you have [Rust](https://www.rust-lang.org/tools/install) installed on your system.

```sh
# Clone the repository
git clone https://github.com/lfavole/wallpaper-changer-rs.git
cd wallpaper-changer-rs

# Build the project
cargo build --release

# Register as a scheduled task (if you want)
./target/release/wallpaper_changer register
```

## Usage

### Configuration

Edit the `config.toml` file to configure the wallpaper changer. You can set the path to your local images or configure Unsplash settings.

### Commands

- Change wallpaper:
    ```sh
    ./wallpaper-changer-rs
    ```

- Register itself as a scheduled task:
    ```sh
    ./wallpaper-changer-rs register
    ```

> [!WARNING]
> On Windows, remember to go in the Task Scheduler (`taskschd.msc`), find the task (wallpaper-changer-rs),
> go in the "Conditions" tab and uncheck "Start the task only if the computer is on AC power".
>
> Otherwise the task will not run if the computer is not plugged in.

- Unregister itself as a scheduled task:
    ```sh
    ./wallpaper-changer-rs register
    ```
