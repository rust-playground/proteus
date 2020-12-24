//! # Proteus
//!
//! Proteus is intended to make dynamic transformation of data using serde serializable, deserialize
//! using JSON and a JSON transformation syntax similar to Javascript JSON syntax.
//!
//! ```rust
//! use proteus::{actions, TransformBuilder};
//! use std::error::Error;
//!
//! // This example show the basic usage of transformations
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let input = r#"
//!         {
//!             "user_id":"111",
//!             "first_name":"Dean",
//!             "last_name":"Karn",
//!             "addresses": [
//!                 { "street":"26 Here Blvd", "postal":"123456", "country":"Canada", "primary":true },
//!                 { "street":"26 Lakeside Cottage Lane.", "postal":"654321", "country":"Canada" }
//!             ],
//!             "nested": {
//!                 "inner":{
//!                     "key":"value"
//!                 },
//!                 "my_arr":[null,"arr_value",null]
//!             }
//!         }"#;
//!     let trans = TransformBuilder::default()
//!         .add_actions(actions!(
//!             ("user_id", "id"),
//!             (
//!                 r#"join(" ", const("Mr."), first_name, last_name)"#,
//!                 "full-name"
//!             ),
//!             (
//!                 r#"join(", ", addresses[0].street, addresses[0].postal, addresses[0].country)"#,
//!                 "address"
//!             ),
//!             ("nested.inner.key", "prev_nested"),
//!             ("nested.my_arr", "my_arr"),
//!             (r#"const("arr_value_2")"#, "my_arr[]")
//!         )?)
//!         .build()?;
//!     let res = trans.apply_from_str(input)?;
//!     println!("{}", serde_json::to_string_pretty(&res)?);
//!     Ok(())
//! }
//! ```
//!
//! or direct from struct to struct
//!
//! ```rust
//! use proteus::{actions, TransformBuilder};
//! use serde::{Deserialize, Serialize};
//! use std::error::Error;
//!
//! #[derive(Serialize)]
//! struct KV {
//!     pub key: String,
//! }
//!
//! #[derive(Serialize)]
//! struct Nested {
//!     pub inner: KV,
//!     pub my_arr: Vec<Option<String>>,
//! }
//!
//! #[derive(Serialize)]
//! struct Address {
//!     pub street: String,
//!     pub postal: String,
//!     pub country: String,
//! }
//!
//! #[derive(Serialize)]
//! struct RawUserInfo {
//!     pub user_id: String,
//!     pub first_name: String,
//!     pub last_name: String,
//!     pub addresses: Vec<Address>,
//!     pub nested: Nested,
//! }
//!
//! #[derive(Serialize, Deserialize)]
//! struct User {
//!     pub id: String,
//!     #[serde(rename = "full-name")]
//!     pub full_name: String,
//!     pub address: String,
//!     pub prev_nested: String,
//!     pub my_arr: Vec<Option<String>>,
//! }
//!
//! // This example show the basic usage of transformations
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let input = RawUserInfo {
//!         user_id: "111".to_string(),
//!         first_name: "Dean".to_string(),
//!         last_name: "Karn".to_string(),
//!         addresses: vec![
//!             Address {
//!                 street: "26 Here Blvd".to_string(),
//!                 postal: "123456".to_string(),
//!                 country: "Canada".to_string(),
//!             },
//!             Address {
//!                 street: "26 Lakeside Cottage Lane.".to_string(),
//!                 postal: "654321".to_string(),
//!                 country: "Canada".to_string(),
//!             },
//!         ],
//!         nested: Nested {
//!             inner: KV {
//!                 key: "value".to_string(),
//!             },
//!             my_arr: vec![None, Some("arr_value".to_owned()), None],
//!         },
//!     };
//!     let trans = TransformBuilder::default()
//!         .add_actions(actions!(
//!             ("user_id", "id"),
//!             (
//!                 r#"join(" ", const("Mr."), first_name, last_name)"#,
//!                 "full-name"
//!             ),
//!             (
//!                 r#"join(", ", addresses[0].street, addresses[0].postal, addresses[0].country)"#,
//!                 "address"
//!             ),
//!             ("nested.inner.key", "prev_nested"),
//!             ("nested.my_arr", "my_arr"),
//!             (r#"const("arr_value_2")"#, "my_arr[]")
//!         )?)
//!         .build()?;
//!     let res: User = trans.apply_to(input)?;
//!     println!("{}", serde_json::to_string_pretty(&res)?);
//!     Ok(())
//! }
//! ```
//!
pub mod action;
pub mod actions;
pub mod errors;
pub mod parser;
pub mod transformer;

#[doc(inline)]
pub use parser::{Parsable, Parser, COMMA_SEP_RE, QUOTED_STR_RE};

#[doc(inline)]
pub use transformer::TransformBuilder;

#[doc(inline)]
pub use errors::Error;

/// This macros is shorthand for creating a set of actions to be added to [TransformBuilder](struct.TransformBuilder.html).
#[macro_export]
macro_rules! actions {
    ($($p:expr),*) => {
        {
            let mut parsables = Vec::new();
            $(
                parsables.push(proteus::Parsable::new($p.0, $p.1));
            )*
            proteus::Parser::parse_multi(&parsables)
        }
    };
}
