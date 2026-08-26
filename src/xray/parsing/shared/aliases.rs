use std::collections::HashMap;

use serde::Deserialize;

pub type Address = String;
pub type Cidr = String;
pub type DomainMatcher = String;
pub type DurationString = String;
pub type StringMap = HashMap<String, String>;
pub type StringArrayMap = HashMap<String, Vec<String>>;

#[derive(Deserialize)]
#[serde(untagged)]
enum StringOrList {
    One(String),
    Many(Vec<String>),
}

pub fn deserialize_optional_string_list<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(
        Option::<StringOrList>::deserialize(deserializer)?.map(|value| match value {
            StringOrList::One(value) => value.split(',').map(str::to_string).collect(),
            StringOrList::Many(values) => values,
        }),
    )
}
