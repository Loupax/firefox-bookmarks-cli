# ff_bookmarks

A lightweight, safe, and blazing-fast CLI tool written in Rust to query your Firefox bookmarks.

It automatically detects your Firefox profile, safely copies the locked SQLite database to a temporary location to avoid conflicts, and lists your bookmarks.

## Y tho?
I like to start most of my workflows using my keyboard, so I wanted an easy way to navigate my bookmarks without using the mouse. Once this tool is running I basically add the following script to a hotkey and open what I want without leaving my hands fro the keyboard.
```bash
#!/bin/bash

choice=$(ff_bookmarks --profile default-release | tofi --prompt-text "> ")

if [ -z "$choice" ]; then
    exit 0
fi

url=$(echo "$choice" | awk -F '\t' '{print $2}')

# Open Firefox
#    We use setsid to detach Firefox from this script, preventing it 
#    from closing if the script terminal closes.
setsid -f firefox "$url" >/dev/null 2>&1
```

## Features

* **Zero Config:** Automatically finds the default Firefox profile on Windows, macOS, and Linux.
* **Safe:** Never reads the live database directly; works on a temporary copy to prevent "database locked" errors.
* **Profile Support:** Query specific profiles (e.g., `dev-edition`) via arguments.
* **Shell Integration:** Includes a Bash completion script for tab-completing profile names.

## Prerequisites

* Rust and Cargo installed (`rustup`).
* Firefox installed.

## Build & Install

1.  **Clone or create the project:**
    ```bash
    git clone <your-repo-url>
    cd firefox_bookmarks
    ```

2.  **Ensure Cargo.toml is configured:**
    Make sure your `Cargo.toml` has the correct name, or `cargo install` will use the folder name.
    ```toml
    [package]
    name = "ff_bookmarks"
    # ...
    ```

3.  **Build & Install:**
    This command compiles the project and moves the binary to `~/.cargo/bin` (ensure this path is in your `$PATH`).
    ```bash
    cargo install --path .
    ```

## Usage

**List bookmarks from the default profile:**
```bash
ff_bookmarks
```

**List bookmarks from a specific profile:**
```bash
ff_bookmarks --profile dev-edition
```

**List available profile names:**
```bash
ff_bookmarks --list-profiles
```

---

## Setting up Autocompletion (Manual Method)

To enable Tab-completion for profile names (e.g., `ff_bookmarks --profile <Tab>`), follow these steps.

### 1. Create the Completion Script
Create a file named `completion.bash` in a safe location (e.g., `~/.config/ff_bookmarks/` or your project root) and paste the following content:

```bash
_bookmarks_completion() {
    local cur prev suggestions

    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    local commands="--help --profile --list-profiles"

    if [[ "${prev}" == "--profile" ]]; then
        # Note: Ensure 'ff_bookmarks' is in your PATH
        local real_profiles=$(ff_bookmarks --list-profiles)
        suggestions=$(compgen -W "${real_profiles}" -- "${cur}")
    else
        suggestions=$(compgen -W "${commands}" -- "${cur}")
    fi

    COMPREPLY=( $suggestions )
}

complete -F _bookmarks_completion ff_bookmarks
```

### 2. Register the Script
Open your `.bashrc` (Linux) or `.bash_profile` (macOS) in a text editor:

```bash
nvim ~/.bashrc
```

Add the following line to the bottom of the file (update the path to where you saved `completion.bash`):

```bash
source /path/to/your/completion.bash
```

### 3. Reload Shell
Apply the changes by restarting your terminal or running:

```bash
source ~/.bashrc
```
