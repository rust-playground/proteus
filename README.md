# Proteus &emsp; [![Build Status]][travis] [![Latest Version]][crates.io]

[Build Status]: https://api.travis-ci.org/rust-playground/proteus.svg?branch=master
[travis]: https://travis-ci.org/rust-playground/proteus
[Latest Version]: https://img.shields.io/crates/v/proteus.svg
[crates.io]: https://crates.io/crates/proteus

**This library is intended to make dynamic transformation of data using serde serializable, deserialize using JSON and a 
JSON transformation syntax similar to Javascript JSON syntax. It supports registering custom Actions for use in syntax**

---

```toml
[dependencies]
proteus = "0.1"
```

## Getter/Setter Syntax
The Getter and Setter syntax is custom to support custom/dynamic Actions and nearly identical with the Setter having 
additional options. If other parsing syntax is desired it can be used to build the Transformation in the same way that
is done internally.

The transformation syntax is very similar to access JSON data in Javascript.
To handle special characters such as ``(blank), `[`, `]`, `"` and `.` you can use the explicit
key syntax `["example[].blah"]` which would represent the key in the following JSON:
```json
{
  "example[].blah" : "my value"
}
```

**IMPORTANT:** order of operations is important.

#### Getter
| syntax | description |
---------|-------------|
| | this will grab the top-level value which could be any valid type: Object, array, ... |
| id | Gets a JSON Object's name. eg. key in HashMap |
| [0] | Gets a JSON Arrays index at the specified index. |
| profile.first_name | Combine Object names with dot notation. |
| profile.address[0].street | Combinations using dot notation and indexes is also supported. |

#### Setter
| syntax | description |
---------|-------------|
| | this will set the top-level value in the destination |
| id | By itself any text is considered to be a JSON Object's name. |
| [] | This appends the source **data** to an array, creating it if it doesn't exist and is only valid at the end of set syntax eg. profile.address[] |
| [\+] | The source Array should append all of it's values into the destination Array and is only valid at the end of set syntax eg. profile.address[] |
| [\-] | The source Array values should replace the destination Array's values at the overlapping indexes and is only valid at the end of set syntax eg. profile.address[] |
| {} | This merges the supplied Object overtop of the existing and is only valid at the end of set syntax eg. profile{} |
| profile.first_name | Combine Object names with dot notation. |
| profile.address[0].street | Combinations using dot notation and indexes is also supported. |

## Example usages
```rust
use proteus::{actions, TransformBuilder};
use std::error::Error;

// This example show the basic usage of transformations
fn main() -> Result<(), Box<dyn Error>> {
    let input = r#"
        {
            "user_id":"111",
            "first_name":"Dean",
            "last_name":"Karn",
            "addresses": [
                { "street":"26 Here Blvd", "postal":"123456", "country":"Canada", "primary":true },
                { "street":"26 Lakeside Cottage Lane.", "postal":"654321", "country":"Canada" }
            ],
            "nested": {
                "inner":{
                    "key":"value"
                },
                "my_arr":[null,"arr_value",null]
            }
        }"#;
    let trans = TransformBuilder::default()
        .add_actions(actions!(
            ("user_id", "id"),
            (
                r#"join(" ", const("Mr."), first_name, last_name)"#,
                "full-name"
            ),
            (
                r#"join(", ", addresses[0].street, addresses[0].postal, addresses[0].country)"#,
                "address"
            ),
            ("nested.inner.key", "prev_nested"),
            ("nested.my_arr", "my_arr"),
            (r#"const("arr_value_2")"#, "my_arr[]")
        )?)
        .build()?;
    let res = trans.apply_from_str(input)?;
    println!("{}", serde_json::to_string_pretty(&res)?);
    Ok(())
}
```

or when you want to do struct to struct transformations

```rust
use proteus::{actions, TransformBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize)]
struct KV {
    pub key: String,
}

#[derive(Serialize)]
struct Nested {
    pub inner: KV,
    pub my_arr: Vec<Option<String>>,
}

#[derive(Serialize)]
struct Address {
    pub street: String,
    pub postal: String,
    pub country: String,
}

#[derive(Serialize)]
struct RawUserInfo {
    pub user_id: String,
    pub first_name: String,
    pub last_name: String,
    pub addresses: Vec<Address>,
    pub nested: Nested,
}

#[derive(Serialize, Deserialize)]
struct User {
    pub id: String,
    #[serde(rename = "full-name")]
    pub full_name: String,
    pub address: String,
    pub prev_nested: String,
    pub my_arr: Vec<Option<String>>,
}

// This example show the basic usage of transformations
fn main() -> Result<(), Box<dyn Error>> {
    let input = RawUserInfo {
        user_id: "111".to_string(),
        first_name: "Dean".to_string(),
        last_name: "Karn".to_string(),
        addresses: vec![
            Address {
                street: "26 Here Blvd".to_string(),
                postal: "123456".to_string(),
                country: "Canada".to_string(),
            },
            Address {
                street: "26 Lakeside Cottage Lane.".to_string(),
                postal: "654321".to_string(),
                country: "Canada".to_string(),
            },
        ],
        nested: Nested {
            inner: KV {
                key: "value".to_string(),
            },
            my_arr: vec![None, Some("arr_value".to_owned()), None],
        },
    };
    let trans = TransformBuilder::default()
        .add_actions(actions!(
            ("user_id", "id"),
            (
                r#"join(" ", const("Mr."), first_name, last_name)"#,
                "full-name"
            ),
            (
                r#"join(", ", addresses[0].street, addresses[0].postal, addresses[0].country)"#,
                "address"
            ),
            ("nested.inner.key", "prev_nested"),
            ("nested.my_arr", "my_arr"),
            (r#"const("arr_value_2")"#, "my_arr[]")
        )?)
        .build()?;
    let res: User = trans.apply_to(input)?;
    println!("{}", serde_json::to_string_pretty(&res)?);
    Ok(())
}
```

#### Actions
The following are the supported actions.

|action|description|
|------|-----------|
|const("Mr.")|Is used to define a constant value.|
|join(",", const("Mr."), first_name, last_name)|Joins one or more using the provided separator|
|len(array_field)|Returns the length of a string, array or an object(by number of keys).|
|strip_start("v", key)|Strips the provided prefix from string values.|
|strip_end("v", key)|Strips the provided suffix from string values.|
|sum(cost, taxes, const(1))|Sums one or more provided values.|
|trim(key)|Trim the start and end whitespace from strings.|
|trim_start(key)|Trim the start whitespace from strings.|
|trim_end(key)|Trim the end whitespace from strings.|


#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Proteus by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>