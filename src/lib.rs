/// Copyright (c) 2023  Ward van der Veer

extern crate proc_macro;
extern crate serde_json;
use proc_macro::TokenStream;
use std::str::FromStr;
use std::collections::HashMap;

mod processor;

use processor::json_schema_to_struct;
use processor::json_schema_file_impl;


/// include JSON Schema directly in the Rust code
/// 
/// json_schema_here({ ...schema... });
/// 
#[proc_macro]
pub fn json_schema_here(schema_body: TokenStream) -> TokenStream {
    let schema_text: String = schema_body.to_string(); 
    let custom_name_map: HashMap<String, String> = HashMap::new();
    let custom_type_map: HashMap<String, String> = HashMap::new();
    let struct_text = json_schema_to_struct(&schema_text, &custom_name_map, &custom_type_map);
    let ts  = TokenStream::from_str(&struct_text.unwrap());
    match ts {
        Ok(rslt)      => return rslt,
        Err(err_msg)     => panic!("Could not parse error {} from JSON Schema {}\n", err_msg, schema_text)
    }
}

/// include JSON Schema from a file
/// supports custom names and types 
/// 
/// json_schema_file("<filename>", "<custom_type1>", "<custom_name1>", "<custom_name2>", "<custom_type2>", ...);
/// 
/// <filename> is relative path, e.g. "src/schema.json"
/// 
/// <custom_type> is "name=type", e.g. "flag=bool"
/// 
/// <custom_name is "old_name->new_name", e.g. "my field->my_field"
/// 
#[proc_macro]
pub fn json_schema_file(parameters: TokenStream) -> TokenStream {
    let mut parameter_number: i32 = 1;
    let mut file_path: String = "".to_string();
    let mut custom_name_map: HashMap<String, String> = HashMap::new();
    let mut custom_type_map: HashMap<String, String> = HashMap::new();
    for parameter in parameters.into_iter() {
        let syntax: String = parameter.to_string();
        if &syntax[0..1] != "\"" || &syntax[syntax.len()-1..syntax.len()] != "\"" {
            continue;
        }
        let param: String = syntax[1..syntax.len()-1].to_string();
        if parameter_number == 1 {
            file_path = param;
        } else if param.contains("=") {
            // type override
            let custom_type_parts: Vec<&str> = param.split("=").collect();
            if custom_type_parts.len() != 2 {
                panic!("Could not parse JSON Schema Invalid custom type: {}\n", param);
            }
            custom_type_map.insert(custom_type_parts[0].to_string(), custom_type_parts[1].to_string());
        } else if param.contains("->") {
            // name override
            let custom_name_parts: Vec<&str> = param.split("->").collect();
            if custom_name_parts.len() != 2 {
                panic!("Could not parse JSON Schema Invalid custom name: {}\n", param);
            }
            custom_name_map.insert(custom_name_parts[0].to_string(), custom_name_parts[1].to_string());
        }    
        parameter_number = parameter_number + 1;
    }
    let struct_text = json_schema_file_impl(file_path.clone(), &custom_name_map, &custom_type_map);
    let ts  = TokenStream::from_str(&struct_text);
    match ts {
        Ok(rslt)      => return rslt,
        Err(err_msg)     => panic!("Could not parse error {} from JSON Schema in {}\n", err_msg, file_path)
    }
}    
