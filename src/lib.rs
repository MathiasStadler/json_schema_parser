/// Copyright (c) 2023  Ward van der Veer

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
        Err(err_msg)     => panic!("Could not parse error {} from JSONSchema {}\n", err_msg, schema_text)
    }
}


/// convert JSONSchema in a string slice to a Rust struct
fn json_schema_to_struct(schema_text: &str) -> Result<String> {
    let schema_json_maybe: Result<Value> = serde_json::from_str(schema_text);
    let schema_json_value: Value = match schema_json_maybe {
        Ok(sj)        => sj,
        Err(err_msg)  => panic!("Could not parse JSON error {}\n", err_msg)
    };
    let schema_json_map: Map<String, Value> = match schema_json_value {
        Value::Object(obj)  => obj,
        _                   => panic!("Could not parse JSONSchema not JSON Object\n")
    };
    json_schema_map_to_struct(&schema_json_map)
}

/// convert JSONSchema in a serde JSON map to a Rust struct
fn json_schema_map_to_struct(schema_json_map: &Map<String, Value>) -> Result<String> {
    if schema_json_map.contains_key("title") == false || schema_json_map["title"].as_str() == None {
        panic!("Could not parse JSONSchema, no title\n");
    }
    if schema_json_map.contains_key("properties") == false {
        panic!("Could not parse JSONSchema, no properties\n");
    }
    let title: String = capitalise(schema_json_map["title"].as_str().unwrap());
    let mut rslt: String = format!("#[derive(Clone, Serialize, Deserialize)]\r\nstruct {} {{\n", title);
    let props_value: Value = schema_json_map["properties"].clone();
    if let Value::Object(props_map) = props_value {
        for props_map_item in props_map.iter() {
            let key_name = props_map_item.0.clone();
            let defn_value = props_map_item.1.clone();
            let field_text: String = get_field_text(&key_name, &defn_value);
            rslt = format!("{}{}", rslt, field_text);
        }
    }
    rslt = rslt + "}\n";
    if schema_json_map.contains_key("$defs") {
        rslt = format!("{}{}", rslt, process_defs(&schema_json_map["$defs"]));
    }
    return Ok(rslt);
}

/// process the &defs field
fn process_defs(defs_value: &Value) -> String {
    let mut rslt: String = "".to_string();
    if let Value::Object(defs_map) = defs_value {
        for defs_map_item in defs_map.iter() {
            let key_name = defs_map_item.0;
            let defn_value = defs_map_item.1;
            if let Value::Object(defn_map) = &defn_value {
                let mut defn_map_mut = defn_map.clone();
                defn_map_mut.insert("title".to_string(), Value::String(key_name.to_string()));
                let this_def: Result<String> = json_schema_map_to_struct(&defn_map_mut);
                if let Ok(this_ok) = this_def {
                    rslt = format!("{}\n\n{}", rslt, this_ok);
                } else {
                    panic!("Could not parse JSONSchema, invalid $def {}\n", key_name);
                }
            } else {
                panic!("Could not parse JSONSchema, invalid $def {}\n", key_name);
            }

        }
        return rslt;
    }
    panic!("Could not parse JSONSchema, invalid $defs");
}

/// convert a property to a Rust field declaration
fn get_field_text(key_name: &str, defn_value: &Value) -> String {
    if let Value::Object(defn_map) = defn_value {
        if defn_map.contains_key("type") == false {
            panic!("Could not parse JSONSchema, no type for {}\n", key_name);
        }
        let json_type_name = defn_map["type"].as_str().unwrap();
        let rust_type_name: String = match json_type_name {
            "array"      => {
                               let item_type_name: String = if let Value::Object(item_type_map) = &defn_map["items"] {
                                   if ! item_type_map.contains_key("type") && ! item_type_map.contains_key("$ref") {
                                       panic!("Could not parse JSONSchema, invalid array item type for {}\n", key_name);
                                   } 
                                   if item_type_map.contains_key("type") {
                                       // type
                                       if let Value::String(item_type) = &item_type_map["type"] {
                                           let item_type_string = get_simple_rust_type(&item_type);
                                           item_type_string
                                       } else {
                                           panic!("Could not parse JSONSchema, invalid array item type for {}\n", key_name);   
                                       }    
                                   } else {
                                       // $ref
                                       if let Value::String(ref_name) = &item_type_map["$ref"] {
                                           if &ref_name[0..8] == "#/$defs/" {
                                               let referenced_type = capitalise(&ref_name[8..]);
                                               referenced_type
                                           } else {
                                               panic!("Could not parse JSONSchema, unknown type {}\n", json_type_name);
                                           }
                                       } else {
                                          panic!("Could not parse JSONSchema, invalid array item type for {}\n", key_name); 
                                       }
                                   }
                               } else {
                                   panic!("Could not parse JSONSchema, no array item type for {}\n", key_name);
                               };
                               let full_name: String = format!("Vec<{}>", item_type_name);
                               full_name
                            },
            _            => {
                                let full_name: String = get_simple_rust_type(json_type_name);
                                full_name
                            }     
        };
        if defn_map.contains_key("required") {
            return format!("    {}: {},\n", key_name, rust_type_name);
        } else {
            return format!("    #[serde(default)]\n    {}: {},\n", key_name, rust_type_name);
        }
    } else {
        panic!("Could not parse JSONSchema, bad defintion for {}\n", key_name);
    }   
} 

/// convert JSONSchema types to Rust equivalents
fn get_simple_rust_type(json_type_name: &str) -> String {
    let rust_type_name: &str = match json_type_name {
        "boolean"    => "bool",
        "number"     => "string",
        "string"     => "String",
        "integer"    => "i32",
        "object"     => "serde_json::Value::Object",
        _            => panic!("Could not parse JSONSchema, unknown type {}\n", json_type_name)
    };
    return rust_type_name.to_string();
}

/// convert string to Capital Case
fn capitalise(src: &str) -> String {
    let lc: String = src.to_string();
    return lc[0..1].to_uppercase() + &lc[1..];
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_example_file1() {
        let file_path: String = "src/example.json".to_string();
        let contents: String = fs::read_to_string(file_path)
        .expect("Could not read example file\n");

        let ts = json_schema_to_struct(&contents);
        print!("{}\r\n", ts.unwrap());
        //assert_eq!(result, 4);
    }

    #[test]
    fn process_example_file2() {
        let file_path: String = "src/example2.json".to_string();
        let contents: String = fs::read_to_string(file_path)
        .expect("Could not read example file 2\n");

        let ts = json_schema_to_struct(&contents);
        print!("{}\r\n", ts.unwrap());
        //assert_eq!(result, 4);
    }


}
