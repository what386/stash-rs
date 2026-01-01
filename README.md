# Stash File Stack

**Stash** is a stack-based file management tool for the terminal.
It lets you temporarily move files and directories out of your working tree, then restore them later — without committing, archiving, or losing context.

Stash is designed to feel _obvious_: in many cases, you don’t need to specify whether you’re pushing or popping — Stash infers your intent based on context.

---

## **Table of Contents**

1. [Features](#features)
2. [Installation](#installation)
   1. [Install via Cargo](#install-via-cargo)
   2. [Build from Source](#build-from-source)

3. [Usage](#usage)
   1. [Basic Operations](#basic-operations)
   2. [How Operations Are Inferred](#how-operations-are-inferred)
   3. [Push (Stash Files)](#push-stash-files)
   4. [Pop (Restore Files)](#pop-restore-files)
   5. [Peek (Copy Without Removing)](#peek-copy-without-removing)
   6. [Delete Entry](#delete-entry)
   7. [List Entries](#list-entries)
   8. [Search Entries](#search-entries)
   9. [View Information](#view-information)
   10. [Clean Old Entries](#clean-old-entries)
   11. [Rename Entry](#rename-entry)
   12. [Export to Archive](#export-to-archive)
   13. [Dump All Entries](#dump-all-entries)

4. [Examples](#examples)
5. [License](#license)
6. [Contributing](#contributing)

---

## **Features**

- **Stack-based workflow** — Push files into a stash, pop them back later
- **Operation inference** — Automatically decides whether to stash or restore
- **History & metadata** — Track creation time, size, and contents
- **Automatic cleanup** — Remove old entries after a configurable time

---

## **Installation**

### **Install via Cargo**

```bash
cargo install stash-rs
```

Ensure Cargo’s bin directory is in your `PATH`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

---

### **Build from Source**

Requires [Rust and Cargo](https://www.rust-lang.org/tools/install):

```bash
git clone https://github.com/what386/stash-rs.git
cd stash-rs
cargo build --release
```

Executable:

```text
./target/release/stash-rs
```

---

## **Usage**

All commands support `--help`:

```bash
stash --help
```

---

### **Basic Operations**

Stash tries to “do the right thing” based on context.

```bash
# Stash a file (it exists locally)
stash file.txt

# Restore it later (file no longer exists locally)
stash file.txt

# Restore the most recent entry
stash
```

You can always force a specific operation using flags such as `--push` or `--pop`.

---

## **How Operations Are Inferred**

When you run:

```bash
stash [items...]
```

Stash determines what you mean using a small set of predictable rules.

### Inference Rules (Priority Order)

1. **No arguments**
   → Pop (restore) the most recent entry

2. **All arguments exist as paths in the current directory**
   → Push (stash) those files or directories

3. **Argument matches a stashed entry name**
   → Pop that entry

4. **Argument matches files inside a stashed entry**
   → Pop the entry containing those files

5. **Ambiguous or no match**
   → Prompt for clarification or require explicit flags

---

### Examples

```bash
# No args → restore most recent
stash
```

```bash
# File exists in the current directory → stash it
stash notes.txt
```

```bash
# File is not in the current directory, but entry name matches → restore
stash notes.txt
```

```bash
# Multiple paths in current directory → stash together in one entry

# Multiple path entries are the first item's filename..
stash readme.md license

# Or passed explicitly via the "--name <NAME>" flag
stash data/ schema.sql -n project-data
```

---

### Ambiguity Handling

If a name refers to **both** a local file and a stashed entry:

```bash
stash work.txt
```

Stash will prompt you to resolve the issue:

```text
Multiple matches found:
1. File "work.txt" in current directory (push)
2. Stashed entry "work.txt" (pop)

Select an option (1–2), or use --push / --pop to specify.
```

You can control this behavior via configuration or bypass it entirely using explicit flags.

---

### Forcing an Operation

Explicit flags always override inference:

```bash
stash --push work.txt     # Always stash
stash --pop work.txt      # Always restore
stash --peek work.txt     # Copy out without removing
```

---

### General idea

- **If it exists here → stash it**
- **If it exists in stash → restore it**
- **If it’s unclear → be explicit**

---

## **Push (Stash Files)**

```bash
stash <file1> [file2 ...] [options]
```

**Options:**

- `--name` / `-n` `<NAME>` – Name the entry
- `--copy` / `-c` – Copy instead of move
- `--link` / `-l` – Leave symlinks behind

**Examples:**

```bash
stash file.txt dir/
stash src/ -n "project-source-old"
stash document.pdf --copy
stash config/ --link
```

---

## **Pop (Restore Files)**

```bash
stash --pop [identifier] [options]
```

**Options:**

- `--copy` / `-c` – Copy instead of move
- `--force` / `-f` – Overwrite existing files
- `--restore` – Restore to original paths

**Examples:**

```bash
stash
stash "project-backup"
stash --restore
stash --copy
```

---

## **Peek (Copy Without Removing)**

```bash
stash --peek [identifier]
```

**Examples:**

```bash
stash --peek
stash --peek work-files
```

---

## **List Entries**

```bash
stash --list
```

Displays:

- Name
- UUID
- Creation date
- Size
- Item count

---

## **Search Entries**

```bash
stash --search <pattern>
```

```bash
stash --search project
```

---

## **View Information**

```bash
stash --info [identifier]
```

```bash
stash --info
stash --info backup-2024
```

---

## **Clean Old Entries**

```bash
stash --clean [days]
```

```bash
stash --clean
stash --clean 7
```

---

## **Rename Entry**

```bash
stash --rename <old:new>
```

```bash
stash --rename "temp:production-backup"
```

---

## **Export to Archive**

```bash
stash --tar <output-file>
```

```bash
stash --tar backup.tar
```

---

## **Dump All Entries**

```bash
stash --dump [--delete]
```

```bash
stash --dump
stash --dump --delete
```

---

## **Examples**

### Temporary Cleanup

```bash
stash old-code/ notes.txt
# work on something else
stash
```

### Context Switching

```bash
stash src/ --name feature-x
stash src/ --name bugfix-y
stash feature-x --restore
```

### Quick Backups

```bash
# create config-backup, move to stash
stash config/ --copy --name config-backup

# restore config-backup from stash
stash config-backup
```

---

## **Contributing**

Contributions are welcome!
Issues and pull requests can be submitted on [GitHub](https://github.com/what386/stash-rs).
