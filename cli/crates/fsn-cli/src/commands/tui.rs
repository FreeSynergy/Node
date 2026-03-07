// `fsn tui` — start the terminal UI dashboard.

use std::path::Path;
use anyhow::Result;

pub async fn run(root: &Path) -> Result<()> {
    fsn_tui::run(root)
}
