# rcpy

**`rcpy`** is a fast, Rust-based recursive copy tool that features a progress bar, multi-threaded performance, and optional `--dry-run` simulation mode.

### Why `rcpy`?

Unlike traditional `cp`, `rcpy` is:
- ✅ **Multi-threaded** by default (with a single-thread fallback)
- ✅ Shows a **progress bar**
- ✅ Includes an optional **dry-run mode** to simulate operations
- ✅ Supports **file exclusion by extension**
- ✅ Prints summaries after copy completes

---

## Features

- ✅ Recursive and non-recursive modes
- ✅ Multi-threaded (default) or single-threaded
- ✅ `--dry-run` support to simulate without writing files
- ✅ Exclude files by extension with `--exclude`
- ✅ Show only files, only dirs, or both via output controls
- ✅ Summary of copied files/directories + duration

---

## Installation

### From source
Assuming you have rust installed:

```bash
git clone https://github.com/vaeleborne/rcpy.git
cd rcpy
cargo build --release
```

Then Move the binary:

```bash
cp target/release/rcpy ~/.local/bin/
```

Or if on windows: Hit windows key, search environment variables, then in PATH browse to the directory.

## Usage

```bash
rcpy <source> <destination> [OPTIONS]
```

## Options

| Flag              | Description                                      |
|-------------------|--------------------------------------------------|
| `-s`, `--single-thread` | Use a single-threaded copy strategy       |
| `-v`, `--verbose`       | Show both file and directory operations   |
| `--only-files`          | Only output file copy messages            |
| `--only-dirs`           | Only output directory creation messages   |
| `-d`, `--dry-run`       | Simulate copy without writing any files  |
| `--exclude <EXT>`       | Exclude files by extension (e.g. `tmp`)  |
| `--no-recursive`        | Copy only top-level files and folders    |


## Examples

Copy recursively (default):
```bash
rcpy ./project ./backup
```
Simulate copy recursively without writing anything (Long and Short):
```bash
rcpy ./project ./backup --dry-run
rcpy ./project ./backup -d
```

Copy only top-level items:
```bash
rcpy ./project ./backup --no-recursive
```

Copy using a single thread and exclude .psd and .tmp files:
```bash
rcpy ./assets ./output -s --exclude psd --exclude tmp
```

Only show copied files (not directories):
```bash
rcpy ./src ./dst --only-files
```

## Future Plans
--interactive mode (confirm each file)

--log output to a file

--threads N to control parallelism

--update and --skip-existing flags

 Windows .exe installer and shell integration

 .rcpyignore file support (like .gitignore)

## License
MIT License © 2025 Dylan Morgan
