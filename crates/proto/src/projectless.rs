//! Working folders of chats without a project.
//!
//! Each projectless chat runs in its own folder under [`SCRATCH_ROOT`] on its
//! host. The cwd is stored home-relative because the creating device can't
//! know the host's home; the host expands it, and creates the folder, when
//! it runs.

use chrono::{DateTime, Utc};

/// Home-relative parent of every projectless chat's folder.
pub const SCRATCH_ROOT: &str = "~/.zeron/scratch";

/// The scratch cwd for a new projectless chat: `<date>-<id>`, the id cut to
/// its last 8 letters and digits (the random end of a UUID or ULID).
pub fn scratch_cwd(chat_id: &str, created_at: DateTime<Utc>) -> String {
    let id: Vec<char> = chat_id
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect();
    let id: String = id[id.len().saturating_sub(8)..].iter().collect();
    let date = created_at.format("%Y-%m-%d");
    if id.is_empty() {
        format!("{SCRATCH_ROOT}/{date}")
    } else {
        format!("{SCRATCH_ROOT}/{date}-{id}")
    }
}

/// A projectless chat's own folder under [`SCRATCH_ROOT`].
pub fn is_scratch_cwd(cwd: &str) -> bool {
    cwd.trim()
        .strip_prefix(SCRATCH_ROOT)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn scratch_cwd_names_the_folder_by_date_and_id_tail() {
        let at = Utc.with_ymd_and_hms(2026, 10, 21, 0, 0, 0).unwrap();
        assert_eq!(
            scratch_cwd("aaaaaaaa-1111-4111-8111-aaaaaaaaaaaa", at),
            "~/.zeron/scratch/2026-10-21-aaaaaaaa"
        );
        assert!(is_scratch_cwd("~/.zeron/scratch/2026-10-21-aaaaaaaa"));
    }
}
