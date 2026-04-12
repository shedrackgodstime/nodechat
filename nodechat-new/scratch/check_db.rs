use rusqlite::Connection;
use nodechat_new::storage::queries;

fn main() -> anyhow::Result<()> {
    let path = nodechat_new::backend::db_path();
    println!("Database path: {:?}", path);
    if !path.exists() {
        println!("Database file DOES NOT EXIST at path.");
    } else {
        let conn = Connection::open(&path)?;
        match queries::get_local_identity(&conn)? {
            Some(id) => println!("Found identity: {}", id.display_name),
            None => println!("No identity found in database."),
        }
    }
    Ok(())
}
