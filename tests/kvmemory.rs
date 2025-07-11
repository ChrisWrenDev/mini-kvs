use kvs::{KvMemory, Result, StoreTrait};

// Should get previously stored value
#[test]
fn get_stored_value() -> Result<()> {
    let mut store = KvMemory::new();

    store.set("key1".to_owned(), "value1".to_owned())?;
    store.set("key2".to_owned(), "value2".to_owned())?;

    assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
    assert_eq!(store.get("key2".to_owned())?, Some("value2".to_owned()));

    Ok(())
}

// Should overwrite existent value
#[test]
fn overwrite_value() -> Result<()> {
    let mut store = KvMemory::new();

    store.set("key1".to_owned(), "value1".to_owned())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));

    store.set("key1".to_owned(), "value2".to_owned())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value2".to_owned()));

    Ok(())
}

// Should get `None` when getting a non-existent key
#[test]
fn get_non_existent_value() -> Result<()> {
    let mut store = KvMemory::new();

    store.set("key1".to_owned(), "value1".to_owned())?;
    assert_eq!(store.get("key2".to_owned())?, None);

    Ok(())
}

#[test]
fn remove_key() -> Result<()> {
    let mut store = KvMemory::new();

    store.set("key1".to_owned(), "value1".to_owned())?;
    store.remove("key1".to_owned())?;
    assert_eq!(store.get("key1".to_owned())?, None);

    Ok(())
}
