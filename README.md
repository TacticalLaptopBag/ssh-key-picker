# ssh-key-picker

This is part of a little experiment of mine where I rewrite some of my Python tools in Lua,
just to get a feel for the language and to see if I want to fully switch to Lua
for writing simple tools.

`ssh-key-picker` is a simple command line tool that lets you have exactly 1 SSH key at a time.
This is simply accomplished by placing disabled keys into a folder named `disabled-keys` in `~/.ssh`.

Theoretically, this should work fine on any OS.

## Usage
Running just `ssh-key-picker` will show a list of all your keys.
You can then pick from the list to enable that key.

However, you can use the `--key` or `-k` option to specify the name of the key you want to enable.
When using `-k`, you don't need to specify the full name, just part of it.
e.g. running `ssh-key-picker -k rsa` will enable a key named `id_rsa`.

## Install

### Linux
First, run `./depends.sh` to get dependencies.
If you are not running Debian/Ubuntu or any of its derivatives,
you will need to check `depends.txt` for required packages.

Finally, just run `sudo ./install.sh` to install to the system,
or just `./install.sh` to install to your user folder.
You can specify a custom install path by setting `INSTALL_DIR`.
Note that a `lib` folder will be created in the parent of `INSTALL_DIR`.
