use crate::schema::Schema;

#[test]
fn deserialize_schema() {
    let schema: Schema<'_> =
        toml::from_slice(include_bytes!("test_character_schema.toml")).expect("Couldn't deserialize test schema!");
    println!("{:?}", schema);
}
