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
            (r#"const("arr_value_2")"#, "my_arr[]"),
            (r#"len(nested)"#, "z_amount_of_nested_data"),
            (r#"sum(const(1.1), len(nested))"#, "zz_sum_nested")
        )?)
        .build()?;
    let res = trans.apply_from_str(input)?;
    println!("{}", serde_json::to_string_pretty(&res)?);
    Ok(())
}
