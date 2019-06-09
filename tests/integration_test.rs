extern crate carta_schema;

// start, len, name, value, children
use carta_schema::Nugget;

#[test]
fn basic_header_with_text_array() {
    let schema_data = "
        struct root {
            header: Header
        }
        struct Header {
            name: String
        }
        struct String {
            len: int8,
            value: [ascii; len],
        }
    ";
    let schema = carta_schema::compile_schema_file(schema_data).unwrap();

    let bin_data = b"\x04abcd";
    let nugget = carta_schema::apply_schema(&schema, bin_data);
    assert_eq!(
        nugget,
        Nugget {
            start: 0,
            len: 5,
            name: "root".to_string(),
            value: None,
            children: vec![
                Nugget {
                    start: 0,
                    len:  5,
                    name: "header".to_string(),
                    value: None,
                    children: vec![
                        Nugget {
                            start: 0,
                            len: 5,
                            name: "name".to_string(),
                            value: None,
                            children: vec![
                                Nugget {
                                    start: 0,
                                    len: 1,
                                    name: "len".to_string(),
                                    value: Some("4".to_string()),
                                    children: Vec::new(),
                                },
                                Nugget {
                                    start: 1,
                                    len: 4,
                                    name: "value".to_string(),
                                    value: Some("abcd".to_string()),
                                    children: Vec::new(),
                                }
                            ]
                        }
                    ]
                }
            ]
        }
    );
}