# ssh-key-picker

A simple command-line tool that lets you have exactly 1 SSH key at a time.
This is accomplished by placing all other keys inside `~/.ssh/disabled`.

Theoretically, this is cross-platform, though it's only been tested on Linux.

- [ssh-key-picker](#ssh-key-picker)
  - [Install](#install)
  - [Usage](#usage)
  - [File Paths](#file-paths)

## Install

Install with Cargo:
```bash
cargo install ssh-key-picker
```
Ensure `~/.cargo/bin/` is in your path by adding this to your `~/.bashrc` file:
```bash
export PATH=$PATH:$HOME/.cargo/bin
```


## Usage

Just run `ssh-key-picker` and it will automatically detect all of your keys in
`~/.ssh/`.
It will prompt you for a name for each key.
Once done, it will show you a list of keys, and ask which one you want to select.
You can select by either index or a partial string match:
```bash
$ ssh-key-picker
Available keys:
  1: Personal
  2: Professional
Select key (number/name): 1
Activated key Personal as id_ed25519

$ ssh-key-picker
Available keys:
  1: Personal
  2: Professional
Select key (number/name): pro
Activated key Professional as id_ed25519
```

If you already know the name of the key you want to activate,
you can just provide it as an argument:
```bash
$ ssh-key-picker pers
Disabled previously active key: Professional
Activated key Personal as id_ed25519
```


## File Paths

`ssh-key-picker` keeps track of your keys in a file, the location of which varies depending on your OS:
| OS | Location |
| -- | -------- |
| Windows | `%APPDATA%\ssh-key-picker\keys.json` |
| macOS | `$HOME/Library/Application Support/ssh-key-picker/keys.json` |
| Linux | `$XDG_DATA_HOME/ssh-key-picker/keys.json` OR `$HOME/.local/share/ssh-key-picker/keys.json` |
