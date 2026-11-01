//! SQLite in the app's folder.

use reva_desktop_pkg::paths::real_path;
use reva_sqlite::Sqlite;
use std::path::PathBuf;

/// SQLite for the folder `root`.  A database name is a VM path (a name
/// without a leading `/` is taken from `/`), mapped into the folder as the
/// file words map paths; one that would leave the folder does not open.
pub fn new_sqlite(root: PathBuf) -> Sqlite {
    Sqlite::new(Box::new(move |name| {
        let vm = if name.starts_with('/') {
            name.to_string()
        } else {
            format!("/{name}")
        };
        real_path(&root, &vm)
    }))
}
