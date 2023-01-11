# json_schema_parser

This is a JSON Schema parser for Rust.  

This provides two macros that will parse JSON Schema text and create the necessary Rust struct types for 
serialization and deserialization of JSON matching that JSON Schema.

To use these macros, include the following line under 
[dependencies] in cargo.toml:

```
json_schema_parser = { version = "0.2.4", git = "https://github.com/wvdveer/json_schema_parser" }
```

Alternatively, clone this git repository locally, and specify the path in dependencies:

```
json_schema_parser = { path = "../json_schema_parser" }
```

You will also require "serde" (including the "derive" feature) and "serde_json".  

The macro "json_schema_here" allows the inclusion of JSON Schema directly within the Rust source code file,
while the macro "json_schema_file" specified that the JSON Schema document is in a file at the location specified.

It is required that the JSON Schema have a "title" specified, this value will be used as the Rust name of the struct.
However, if json_schema_file is used, a missing title can be supplied by using a custom name instead of blank,

e.g. "->My_Struct_Name" will name the struct My_Struct_Name where "title" is missing.

JSON objects that exist under the main object (e.g. where there is an array of objects), will be moved under the 
"$defs" object in preprocessing.  These can be referenced as { "$ref": "#/$defs/*name*"} in the main object.  Each 
object under $defs will create a separate Rust struct with the name given under $defs.  It is advisable if the same
structure appears in multiple places in your schema, that $defs be used explicitly.

e.g.
```
json_schema_here!({
    "title": "Person",
    "type": "object",
    "properties": {
      "firstName": {
        "type": "string"
      },
      "lastName": {
        "type": "string"
      }
    }  
});
```
will create a struct named "Person" with string fields "firstName" and "lastName". 

```
json_schema_file!("src/example.json");
```
will read the JSON Schema from the file src/example.json.  This path is relative to the location of the Cargo.toml file.


It is possible to also specify custom maps to override the names and types that would otherwise be used.

To override the name, use "*JSON name*->*Rust name*"

e.g.
```
json_schema_file!("src/example.json", "bank statement->Bank_Statement");
```

will change the field or struct named "bank statement" to use the name "BankStatement" in Rust.

To overide the Rust type given to fields, use "*field name*=*Rust type*"

e.g.
```
json_schema_file!("src/example.json", "amount=i64");
```

will override the field named "amount" to be an i64 instead of whatever type would otherwise be assigned.  The field name
specified be the original name without the any custom name override.

You can include as many custom names and types as needed, in any order:

e.g.
```
json_schema_file!("src/example.json", "bank statement->Bank_Statement", "amount=i64");
```

will apply both of the above changes.


Unsupported JSON Schema features:

* required.  All fields are optional and use the Rust default.
* Using multiple possible types, e.g. [ "string", "number" ]
  