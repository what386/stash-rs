# Stash File Stack

**Stash** is a stack-based file management tool for the terminal.
It lets you temporarily move files and directories out of your working tree, then restore them later.

Stash is designed to feel _obvious_: in many cases, you don’t need to specify whether you’re pushing or popping — Stash infers your intent based on context.

---

## **Table of Contents**

1. [Features](#features)
2. [Installation](#installation)
3. [Usage](#usage)
   1. [Basic Operations](#basic-operations)
   2. [How Operations Are Inferred](#how-operations-are-inferred)
   3. [Push (Stash Files)](#push-stash-files)
   4. [Pop (Restore Files)](#pop-restore-files)
   5. [List Entries](#list-entries)
   6. [Search Entries](#search-entries)
   7. [View Information](#view-information)
   8. [Clean Old Entries](#clean-old-entries)
   9. [Rename Entry](#rename-entry)
   10. [Export to Archive](#export-to-archive)
   11. [Dump All Entries](#dump-all-entries)

---

## **Features**

- **Stack-based workflow** — Push files into a stash, pop them back later
- **Operation inference** — Automatically decides whether to stash or restore
- **History & metadata** — Track creation time, size, and contents
- **Automatic cleanup** — Remove old entries after a configurable time

---

## **Usage**

All commands support `--help`:

```bash
stash --help
```

---

## **Basic Operations**

```bash
# Stash a file (it exists locally)
stash file.txt

# Restore it later (file no longer exists locally)
stash file.txt

# Restore the most recent entry
stash
```

---

## **How Operations Are Inferred**

When you run:

```bash
stash [items...]
```

Stash determines what you mean using a small set of predictable rules:

1. **No arguments**
   → Pop (restore) the most recent entry

2. **All arguments exist locally**
   → Push (stash) those files or directories

3. **Argument does not exist locally**
   → Treated as a stash entry identifier and restored

4. **Ambiguous input**
   → Requires explicit flags or clarification

---

## **Push (Stash Files)**

```bash
stash <file1> [file2 ...] [options]
```

**Options:**

- `--name`, `-n <NAME>`
  Assign a custom name to the stash entry

- `--copy`, `-c`
  Copy files instead of moving them

**Examples:**

```bash
stash file.txt dir/
stash src/ -n project-source
stash document.pdf --copy
```

---

## **Pop (Restore Files)**

```bash
stash [identifier] [options]
```

**Options:**

- `--copy`, `-c`
  Copy files instead of moving them

- `--force`, `-f`
  Overwrite existing files when restoring

- `--restore`, `-r`
  Restore files to their original paths

**Examples:**

```bash
stash
stash backup-2024
stash --restore
stash --force
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
stash --rn <old:new>
```

```bash
stash --rename temp:production-backup
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

Restore **all entries** in stash order:

```bash
stash --dump
```

This restores every entry to the current directory using safe defaults.

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
stash config/ --copy --name config-backup
stash config-backup
```

---

## **Contributing**

Contributions are welcome!
Issues and pull requests can be submitted on GitHub.
