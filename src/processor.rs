/// Copyright (c) 2023  Ward van der Veer

extern crate serde_json;
use std::fs;
use serde_json::{Result, Value, Map};
use std::collections::HashMap;

/// implementation of json_schema_file macro code
pub fn json_schema_file_impl(file_path: String, custom_name_map: &HashMap<String, String>, custom_type_map: &HashMap<String, String>) -> String {
    let schema_text: String = fs::read_to_string(&file_path)
        .expect(format!("Could not read JSON Schema file: {}\n", &file_path).as_str());
    let struct_text = json_schema_to_struct(&schema_text, custom_name_map, custom_type_map);
    match struct_text {
        Ok(rslt)      => return rslt,
        Err(err_msg)   => panic!("Could not parse error {} from JSON Schema {}\n", err_msg, schema_text)
    }
}

/// convert JSON Schema in a string slice to a Rust struct
pub fn json_schema_to_struct(schema_text: &str, custom_name_map: &HashMap<String, String>, custom_type_map: &HashMap<String, String>) -> Result<String> {
    let schema_json_maybe: Result<Value> = serde_json::from_str(schema_text);
    let schema_json_value: Value = match schema_json_maybe {
        Ok(sj)        => sj,
        Err(err_msg)  => panic!("Could not parse JSON error {}\n", err_msg)
    };
    let schema_json_map: Map<String, Value> = match schema_json_value {
        Value::Object(obj)  => obj,
        _                   => panic!("Could not parse JSON Schema not JSON Object\n")
    };
    json_schema_map_to_struct(&schema_json_map, custom_name_map, custom_type_map)
}

/// convert JSON Schema in a serde JSON map to a Rust struct
fn json_schema_map_to_struct(schema_json_map_raw: &Map<String, Value>, custom_name_map: &HashMap<String, String>, custom_type_map: &HashMap<String, String>) -> Result<String> {
    let title: String = if schema_json_map_raw.contains_key("title") == false || schema_json_map_raw["title"].as_str() == None {
        if custom_name_map.contains_key("") {
            custom_name_map.get("").unwrap().to_string()
        } else {
            panic!("Could not parse JSON Schema, no title\n");
        }
    } else {
        format_struct_name(schema_json_map_raw["title"].as_str().unwrap(), custom_name_map)
    };
    let schema_json_map = process_embedded_objects_into_defs(&title, schema_json_map_raw);
    if schema_json_map.contains_key("properties") == false {
        panic!("Could not parse JSON Schema, no properties\n");
    }
    let mut rslt: String = format!("#[derive(Clone, Serialize, Deserialize, Default)]\r\npub struct {} {{\n", title);
    let props_value: Value = schema_json_map["properties"].clone();
    if let Value::Object(props_map) = props_value {
        for props_map_item in props_map.iter() {
            let key_name = props_map_item.0.clone();
            let defn_value = props_map_item.1.clone();
            let field_text: String = get_field_text(&key_name, &defn_value, custom_name_map, custom_type_map);
            rslt = format!("{}{}", rslt, field_text);
        }
    }
    rslt = rslt + "}\n";
    if schema_json_map.contains_key("$defs") {
        rslt = format!("{}{}", rslt, process_defs(&schema_json_map["$defs"], custom_name_map, custom_type_map));
    }
    return Ok(rslt);
}

/// move embedded objects into the $defs
fn process_embedded_objects_into_defs(struct_name: &str, schema_json_map: &Map<String, Value>) -> Map<String, Value> {
    let mut new_defs: HashMap<String, Map<String, Value>> = HashMap::new();
    let mut revised_schema_json_map: Map<String, Value> = extract_embedded_objects(struct_name, schema_json_map, &mut new_defs, true);
    if ! new_defs.is_empty() {
        let mut defs_map: Map<String,Value> = Map::new();
        if revised_schema_json_map.contains_key("$defs") {
            if let Value::Object(old_defs_map) = &revised_schema_json_map["$defs"] {
                for (old_def_name, old_def_obj) in old_defs_map {
                    defs_map.insert(old_def_name.to_string(), old_def_obj.clone());
                }
            }
        } 
        for (new_def_name, new_def_obj) in new_defs {
            defs_map.insert(new_def_name, Value::Object(new_def_obj));
        }
        revised_schema_json_map.insert("$defs".to_string(), Value::Object(defs_map));  
    }
    return revised_schema_json_map;
} 

/// extract embedded objects 
fn extract_embedded_objects(name_to_field: &str, schema_json_map_section: &Map<String, Value>, new_defs: & mut HashMap<String, Map<String, Value>>, is_root: bool) -> Map<String, Value> {
    if schema_json_map_section.contains_key("type") == false {
        // may be a $ref
        return schema_json_map_section.clone();
    }

    let section_type: &str = schema_json_map_section["type"].as_str().unwrap();
      
    if section_type != "object" && section_type != "array" {
        // nothing to change
        return schema_json_map_section.clone();
    }
    if section_type == "array" {
        let array_name = format!("{}_item", name_to_field);
        if let Value::Object(items_type) = &schema_json_map_section["items"] {
            let mut new_schema_json_map_section = schema_json_map_section.clone();
            new_schema_json_map_section["items"] = Value::Object(extract_embedded_objects(&array_name, &items_type, new_defs, false));
            return new_schema_json_map_section;
        } else {
            panic!("Can't find item type for {}", array_name);
        }
    }
    let props_value: Value = schema_json_map_section["properties"].clone();
    let mut new_schema_json_map_section = schema_json_map_section.clone();
    if let Value::Object(props_map) = props_value {
        for props_map_item in props_map.iter() {
            let key_name = props_map_item.0.clone();
            let defn_value = props_map_item.1.clone();
            let obj_name = format!("{}_{}", name_to_field, key_name);
            if let Value::Object(obj_type) = defn_value {
                new_schema_json_map_section["properties"][&key_name] = Value::Object(extract_embedded_objects(&obj_name, &obj_type, new_defs, false));
            } else {
                panic!("Can find item type for {}", obj_name);
            }
        }
    }
    if !is_root {
        new_defs.insert(name_to_field.to_string(), new_schema_json_map_section.clone());
        new_schema_json_map_section = Map::new();
        new_schema_json_map_section.insert("$ref".to_string(), Value::String(format!("#/$defs/{}", name_to_field)));
    }
    return new_schema_json_map_section;
}

/// process the $defs field
fn process_defs(defs_value: &Value, custom_name_map: &HashMap<String, String>, custom_type_map: &HashMap<String, String>) -> String {
    let mut rslt: String = "".to_string();
    if let Value::Object(defs_map) = defs_value {
        for defs_map_item in defs_map.iter() {
            let key_name = defs_map_item.0;
            let defn_value = defs_map_item.1;
            if let Value::Object(defn_map) = &defn_value {
                let mut defn_map_mut = defn_map.clone();
                defn_map_mut.insert("title".to_string(), Value::String(key_name.to_string()));
                let this_def: Result<String> = json_schema_map_to_struct(&defn_map_mut, custom_name_map, custom_type_map);
                if let Ok(this_ok) = this_def {
                    rslt = format!("{}\n\n{}", rslt, this_ok);
                } else {
                    panic!("Could not parse JSON Schema, invalid $def {}\n", key_name);
                }
            } else {
                panic!("Could not parse JSON Schema, invalid $def {}\n", key_name);
            }

        }
        return rslt;
    }
    panic!("Could not parse JSON Schema, invalid $defs");
}

/// convert a property to a Rust field declaration
fn get_field_text(key_name: &str, defn_value: &Value, custom_name_map: &HashMap<String, String>, custom_type_map: &HashMap<String, String>) -> String {
    let mut field_name: String = key_name.to_string();
    if custom_name_map.contains_key(key_name) {
        field_name = custom_name_map.get(key_name).unwrap().to_string();
    }
    let rust_type_name: String;
    if custom_type_map.contains_key(key_name) {
        rust_type_name = custom_type_map.get(key_name).unwrap().to_string();
    } else if let Value::Object(defn_m) = defn_value {
        let defn_map: Map<String, Value> = defn_m.clone();
        rust_type_name = get_field_type(key_name, defn_map, custom_name_map);
    } else {
        panic!("Could not parse JSON Schema, bad defintion for {}\n", key_name);
    }   
    return format!("    #[serde(default)]\n    pub {}: {},\n", field_name, rust_type_name);
} 

/// get the rust field type from definition JSON object
fn get_field_type(key_name: &str, defn_map: Map<String, Value>, custom_name_map: &HashMap<String, String>) -> String {
    if defn_map.contains_key("type") == false {
        // $ref
        if let Value::String(ref_name) = &defn_map["$ref"] {
            if &ref_name[0..8] == "#/$defs/" {
                let referenced_type = format_struct_name(&ref_name[8..], custom_name_map);
                return referenced_type;
            } else {
                panic!("Could not parse JSON Schema, unknown $ref for {}\n", key_name);
            }
        } else {
            panic!("Could not parse JSON Schema, no type for {}\n", key_name);
        }
    }
    let json_type_name = defn_map["type"].as_str().unwrap();
    match json_type_name {
        "array"      => {
                            let item_type_name: String = if let Value::Object(item_type_m) = &defn_map["items"] {
                                let item_type_map: Map<String, Value> = item_type_m.clone();
                                get_field_type(&format!("{}[]", key_name), item_type_map, custom_name_map)
                            } else {
                                panic!("Could not parse JSON Schema, no array item type for {}\n", key_name);
                            };
                            return format!("Vec<{}>", item_type_name);
                        },
        _            => {
                            return get_simple_rust_type(json_type_name);
                        }     
    };
}


/// convert JSON Schema types to Rust equivalents
fn get_simple_rust_type(json_type_name: &str) -> String { 
    let rust_type_name: &str = match json_type_name {
        "boolean"    => "bool",
        "number"     => "f64",
        "string"     => "String",
        "integer"    => "i32",
        "object"     => "serde_json::Value::Object",
        _            => panic!("Could not parse JSON Schema, unknown type {}\n", json_type_name)
    };
    return rust_type_name.to_string();
}

/// apply name changes, or else convert string to Capital Case 
fn format_struct_name(src: &str, custom_name_map: &HashMap<String, String>) -> String {
    if custom_name_map.contains_key(src) {
        return custom_name_map.get(src).unwrap().to_string();
    } 
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

        let custom_name_map: HashMap<String, String> = HashMap::new();
        let custom_type_map: HashMap<String, String> = HashMap::new();
        let ts = json_schema_to_struct(&contents, &custom_name_map, &custom_type_map);
        print!("{}\r\n", ts.unwrap());
    }

    #[test]
    fn process_example_file2() {
        let file_path: String = "src/example2.json".to_string();
        let contents: String = fs::read_to_string(file_path)
            .expect("Could not read example file 2\n");

        let custom_name_map: HashMap<String, String> = HashMap::new();
        let custom_type_map: HashMap<String, String> = HashMap::new();
        let ts = json_schema_to_struct(&contents, &custom_name_map, &custom_type_map);
        print!("{}\r\n", ts.unwrap());
    }

    #[test]
    fn process_example_file3_embedded_objs() {
        let file_path: String = "src/example3.json".to_string();
        let contents: String = fs::read_to_string(file_path)
            .expect("Could not read example file 3\n");

        let raw_json_val: Value = serde_json::from_str(&contents).unwrap();
        if let Value::Object(raw_json) = raw_json_val {
            let modified_json = process_embedded_objects_into_defs("People", &raw_json);
            print!("{}\r\n", serde_json::to_string(&modified_json).unwrap());
        }
    }

    #[test]
    fn process_example_file3() {
        let file_path: String = "src/example3.json".to_string();
        let contents: String = fs::read_to_string(file_path)
            .expect("Could not read example file 3\n");

        let mut custom_name_map: HashMap<String, String> = HashMap::new();
        custom_name_map.insert("".to_string(), "People".to_string());
        let custom_type_map: HashMap<String, String> = HashMap::new();
        let ts = json_schema_to_struct(&contents, &custom_name_map, &custom_type_map);
        print!("{}\r\n", ts.unwrap());
    }

    #[test]
    fn process_example_file2_as_file() {
        let file_path: String = "src/example2.json".to_string();
        let mut custom_name_map: HashMap<String, String> = HashMap::new();
        custom_name_map.insert("veggie".to_string(), "Vegetable".to_string());
        let mut custom_type_map: HashMap<String, String> = HashMap::new();
        custom_type_map.insert("veggieLike".to_string(), "i32".to_string());
        let ts: String = json_schema_file_impl(file_path, &custom_name_map, &custom_type_map);

        print!("{}\r\n", ts);
    }

}
