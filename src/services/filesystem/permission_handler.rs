use anyhow::{Context, Result};
use std::path::Path;
use std::fs;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

// Permission bits (only meaningful on Unix)
pub mod bits {
    /// Owner read (0o400)
    pub const OWNER_READ: u32 = 0o400;
    /// Owner write (0o200)
    pub const OWNER_WRITE: u32 = 0o200;
    /// Owner execute (0o100)
    pub const OWNER_EXEC: u32 = 0o100;
    /// Group read (0o040)
    pub const GROUP_READ: u32 = 0o040;
    /// Group write (0o020)
    pub const GROUP_WRITE: u32 = 0o020;
    /// Group execute (0o010)
    pub const GROUP_EXEC: u32 = 0o010;
    /// Others read (0o004)
    pub const OTHERS_READ: u32 = 0o004;
    /// Others write (0o002)
    pub const OTHERS_WRITE: u32 = 0o002;
    /// Others execute (0o001)
    pub const OTHERS_EXEC: u32 = 0o001;
}

/// Set file permissions
/// On Unix: sets mode bits
/// On Windows: only supports making file readonly (if permissions & 0o200 == 0)
pub fn set_permissions(path: &Path, permissions: u32) -> Result<()> {
    #[cfg(unix)]
    {
        let metadata = fs::metadata(path).context("Failed to read metadata")?;
        let mut current = metadata.permissions();
        current.set_mode(permissions);
        fs::set_permissions(path, current).context("Failed to set permissions")?;
    }

    #[cfg(windows)]
    {
        // Windows: only support readonly flag
        let metadata = fs::metadata(path).context("Failed to read metadata")?;
        let mut perms = metadata.permissions();
        let readonly = (permissions & 0o200) == 0; // No write permission = readonly
        perms.set_readonly(readonly);
        fs::set_permissions(path, perms).context("Failed to set permissions")?;
    }

    Ok(())
}

/// Get file permissions
/// On Unix: returns mode bits
/// On Windows: returns simplified permissions (0o444 if readonly, 0o666 otherwise)
pub fn get_permissions(path: &Path) -> Result<u32> {
    let metadata = fs::metadata(path).context("Failed to read metadata")?;

    #[cfg(unix)]
    {
        Ok(metadata.permissions().mode())
    }

    #[cfg(windows)]
    {
        // Windows: convert readonly flag to Unix-like permissions
        Ok(if metadata.permissions().readonly() {
            0o444 // readonly
        } else {
            0o666 // read-write
        })
    }
}

/// Reset to default file permissions (0o644 on Unix, no-op on Windows)
pub fn reset_to_default(path: &Path) -> Result<()> {
    set_permissions(path, 0o644)
}

/// Add specific permission bits without changing others
/// On Unix: bitwise OR with current permissions
/// On Windows: only affects readonly flag
pub fn add_permissions(path: &Path, bits: u32) -> Result<()> {
    let current = get_permissions(path)?;
    set_permissions(path, current | bits)
}

/// Remove specific permission bits without changing others
/// On Unix: bitwise AND NOT with current permissions
/// On Windows: only affects readonly flag
pub fn remove_permissions(path: &Path, bits: u32) -> Result<()> {
    let current = get_permissions(path)?;
    set_permissions(path, current & !bits)
}

/// Copy permissions from one file to another
pub fn copy_permissions(from: &Path, to: &Path) -> Result<()> {
    let permissions = get_permissions(from)?;
    set_permissions(to, permissions)
}

/// Make file executable (Unix only, no-op on Windows)
#[cfg(unix)]
pub fn make_executable(path: &Path) -> Result<()> {
    add_permissions(path, bits::OWNER_EXEC)
}

/// Make file executable (no-op on Windows)
#[cfg(windows)]
pub fn make_executable(_path: &Path) -> Result<()> {
    Ok(()) // Windows determines executability by file extension
}

/// Make file readonly
pub fn make_readonly(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        set_permissions(path, 0o444)
    }

    #[cfg(windows)]
    {
        let metadata = fs::metadata(path)?;
        let mut perms = metadata.permissions();
        perms.set_readonly(true);
        fs::set_permissions(path, perms)?;
        Ok(())
    }
}

/// Make file writable
pub fn make_writable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        add_permissions(path, bits::OWNER_WRITE)
    }

    #[cfg(windows)]
    {
        let metadata = fs::metadata(path)?;
        let mut perms = metadata.permissions();
        perms.set_readonly(false);
        fs::set_permissions(path, perms)?;
        Ok(())
    }
}
