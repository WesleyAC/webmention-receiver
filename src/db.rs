use rusqlite::Connection;

// It's not terribly efficient to open a new sqlite connection in every request that touches the
// database. Fuck it, though, it's easy and it's way faster than anyone needs.
pub fn open() -> Result<Connection, rusqlite::Error> {
    // TODO: set some good pragmas
    rusqlite::Connection::open("./webmention-receiver.sqlite3")
}

pub fn run_migrations(db: &Connection) -> Result<(), rusqlite::Error> {
    let mut version: usize;
    let migrations = vec![include_str!("migrations/0000_init.sql")];
    while {
        version = db.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        version < migrations.len()
    } {
        log::info!("Backing up before running migration {}.", version);
        let mut dst = Connection::open(format!(
            "./webmention-receiver.migrationbackup.{}.{}.sqlite3",
            chrono::Utc::now().format("%s-%f"),
            version
        ))?;
        let backup = rusqlite::backup::Backup::new(&db, &mut dst)?;
        assert!(backup.step(-1)? == rusqlite::backup::StepResult::Done);
        log::info!("Running migration {}.", version);
        db.execute_batch(migrations[version]).unwrap();
    }
    if version > migrations.len() + 1 {
        panic!("unknown schema version!");
    }
    Ok(())
}
