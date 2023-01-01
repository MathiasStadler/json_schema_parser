# json_schema_parser

This is a JSON Schema parser for Rust.  

This provides two macros that will parse JSON Schema text and create the necessary Rust struct types for 
serialization and deserialization of JSON matching that JSON Schema.

The macro "json_schema_here" allows the inclusion of JSON Schema directly within the Rust source code file,
while the macro "json_schema_file" specified that the JSON Schema document is in a file at the location specified.

It is required that the JSON Schema have a "title" specified, this value will be used as the Rust name of the struct.

JSON objects that exist under the main object (e.g. where there is an array of objects), should be specified in the 
"$defs" object.  These should be referenced as { "$ref": "#/$defs/*name*"} in the main object.  Each object under $defs 
will create a separate Rust struct with the name given under $defs.

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


It is possible to also specify custom maps to override the Rust type given to fields, 

e.g.
```
json_schema_file!("src/example.json", "amount=i64");
```

will override the field named "amount" to be an i64 instead of whatever type would otherwise be assigned.
