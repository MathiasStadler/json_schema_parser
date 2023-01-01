extern crate proc_macro;
use proc_macro::TokenStream;
use std::str::FromStr;
use serde_json::{Result, Value, Map};
use std::fs;

#[proc_macro]
pub fn json_schema_here(schema_body: TokenStream) -> TokenStream {
    let schema_text: String = schema_body.to_string(); 
    let struct_text = json_schema_to_struct(&schema_text);
    let ts  = TokenStream::from_str(&struct_text.unwrap());
    match ts {
        Ok(rslt)      => return rslt,
        Err(err_msg)     => panic!("Could not parse error {} from JSONSchema {}", err_msg, schema_text)
    }
}

fn json_schema_to_struct(schema_text: &str) -> Result<String> {
    let schema_json_maybe: Result<Value> = serde_json::from_str(schema_text);
    let schema_json_value: Value = match schema_json_maybe {
        Ok(sj)        => sj,
        Err(err_msg)  => panic!("Could not parse JSON error {}", err_msg)
    };
    let schema_json_map: Map<String, Value> = match schema_json_value {
        Value::Object(obj)  => obj,
        _                   => panic!("Could not parse JSONSchema not JSON Object")
    };
    if schema_json_map.contains_key("title") == false || schema_json_map["title"].as_str() == None {
        panic!("Could not parse JSONSchema, no title");
    }
    if schema_json_map.contains_key("properties") == false {
        panic!("Could not parse JSONSchema, no properties");
    }
    let mut title: String = schema_json_map["title"].as_str().unwrap().to_string();
    title = title[0..1].to_uppercase() + &title[1..];
    let mut rslt: String = format!("struct {} {{\n", title);
    let props_value: Value = schema_json_map["properties"].clone();
    if let Value::Object(props_map) = props_value {
        for key in props_map.keys() {
            let key_name = key.clone();
            let defn_value = props_map.get(key);
            let mut type_name: String = "String".to_string();
            if let Value::Object(defn_map) = defn_value.unwrap() {
                type_name = defn_map["type"].as_str().unwrap().to_string();
                match type_name.as_str() {
                    "integer"    => type_name = "i64".to_string(),
                    "string"     => type_name = "String".to_string(),
                    _            => type_name = type_name[0..1].to_lowercase() + &type_name[1..]
                }
            }
            rslt = format!("{}    {}: {},\r\n", rslt, key_name, type_name);
        }
    }
    rslt = rslt + "}\r\n";
    return Ok(rslt);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_example_file() {
        let file_path: String = "src/example.json".to_string();
        let contents: String = fs::read_to_string(file_path)
        .expect("Should have been able to read the file");

        let ts = json_schema_to_struct(&contents);
        print!("{}\r\n", ts.unwrap());
        //assert_eq!(result, 4);
    }

}
