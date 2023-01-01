extern crate serde;
use serde::{Serialize, Deserialize};
extern crate json_schema_parser;
use json_schema_parser::*;


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


fn main() {
    let p: Person = Person { firstName: "Ward".to_string(), lastName: "van der Veer".to_string() };
    let j: String = serde_json::to_string(&p).unwrap();
    print!("{}\n", j);
}
