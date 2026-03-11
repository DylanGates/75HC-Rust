mod db;

use db::{Item, JsonDb};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = PathBuf::from("data.json");
    let db = JsonDb::new(db_path);

    // Create an item
    let it = db.create("Example item".to_string(), Some("a note".to_string()))?;
    println!("created: {:?}", it);

    // Read the item back
    let found = db.read(&it.id)?;
    println!("read: {:?}", found);

    // Update the item
    let updated = db.update(&it.id, Some("Updated name".to_string()), None)?;
    println!("updated: {:?}", updated);

    // List all items
    let all = db.list()?;
    println!("all items: {:?}", all);

    // Delete the created item
    let deleted = db.delete(&it.id)?;
    println!("deleted: {}", deleted);

    Ok(())
}
