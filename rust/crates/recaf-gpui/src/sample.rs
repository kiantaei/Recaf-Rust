//! A small but genuine JVM class file, built byte-by-byte per the spec, that
//! the app shows when launched without a file argument.

use recaf_classfile::{tags, ClassFile, MAGIC};

pub const DEMO_CLASS_NAME: &str = "org/recaf/demo/RecafRustDemo";

/// Build the raw bytes of the demo class file (Java 21, `major` 65).
pub fn build_demo_class() -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&MAGIC.to_be_bytes());
    b.extend_from_slice(&0u16.to_be_bytes()); // minor_version
    b.extend_from_slice(&65u16.to_be_bytes()); // major_version
    b.extend_from_slice(&18u16.to_be_bytes()); // constant_pool_count

    utf8(&mut b, DEMO_CLASS_NAME); // 1
    class(&mut b, 1); // 2
    utf8(&mut b, "java/lang/Object"); // 3
    class(&mut b, 3); // 4
    utf8(&mut b, "SourceFile"); // 5
    utf8(&mut b, "RecafRustDemo.java"); // 6
    utf8(&mut b, "ConstantValue"); // 7
    utf8(&mut b, "Code"); // 8
    utf8(&mut b, "MAX"); // 9
    utf8(&mut b, "I"); // 10
    b.push(tags::INTEGER); // 11
    b.extend_from_slice(&4242i32.to_be_bytes());
    utf8(&mut b, "greeting"); // 12
    utf8(&mut b, "Ljava/lang/String;"); // 13
    utf8(&mut b, "Hello from Recaf Rust!"); // 14
    b.push(tags::STRING); // 15
    b.extend_from_slice(&14u16.to_be_bytes());
    utf8(&mut b, "compute"); // 16
    utf8(&mut b, "(I)I"); // 17

    b.extend_from_slice(&0x0021u16.to_be_bytes()); // public super
    b.extend_from_slice(&2u16.to_be_bytes()); // this_class
    b.extend_from_slice(&4u16.to_be_bytes()); // super_class
    b.extend_from_slice(&0u16.to_be_bytes()); // 0 interfaces

    b.extend_from_slice(&2u16.to_be_bytes()); // 2 fields
                                              // field MAX: public static final, ConstantValue -> #11 (4242)
    b.extend_from_slice(&0x0019u16.to_be_bytes());
    b.extend_from_slice(&9u16.to_be_bytes());
    b.extend_from_slice(&10u16.to_be_bytes());
    b.extend_from_slice(&1u16.to_be_bytes());
    b.extend_from_slice(&7u16.to_be_bytes()); // "ConstantValue"
    b.extend_from_slice(&2u32.to_be_bytes());
    b.extend_from_slice(&11u16.to_be_bytes());
    // field greeting: public, no attributes
    b.extend_from_slice(&0x0001u16.to_be_bytes());
    b.extend_from_slice(&12u16.to_be_bytes());
    b.extend_from_slice(&13u16.to_be_bytes());
    b.extend_from_slice(&0u16.to_be_bytes());

    b.extend_from_slice(&1u16.to_be_bytes()); // 1 method
                                              // method compute: public, one stub Code attribute (12 filler bytes)
    b.extend_from_slice(&0x0001u16.to_be_bytes());
    b.extend_from_slice(&16u16.to_be_bytes());
    b.extend_from_slice(&17u16.to_be_bytes());
    b.extend_from_slice(&1u16.to_be_bytes());
    b.extend_from_slice(&8u16.to_be_bytes()); // "Code"
    b.extend_from_slice(&12u32.to_be_bytes());
    b.extend(std::iter::repeat(0u8).take(12));

    b.extend_from_slice(&1u16.to_be_bytes()); // 1 class attribute
    b.extend_from_slice(&5u16.to_be_bytes()); // "SourceFile"
    b.extend_from_slice(&2u32.to_be_bytes());
    b.extend_from_slice(&6u16.to_be_bytes()); // -> "RecafRustDemo.java"

    b
}

fn utf8(b: &mut Vec<u8>, s: &str) {
    b.push(tags::UTF8);
    b.extend_from_slice(&(s.len() as u16).to_be_bytes());
    b.extend_from_slice(s.as_bytes());
}

fn class(b: &mut Vec<u8>, name_index: u16) {
    b.push(tags::CLASS);
    b.extend_from_slice(&name_index.to_be_bytes());
}

/// Parse the demo bytes into a [`ClassFile`].
pub fn build_demo_class_file() -> ClassFile {
    ClassFile::parse(&build_demo_class()).expect("embedded demo class must parse")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_class_parses() {
        let class = build_demo_class_file();
        assert_eq!(class.this_class, DEMO_CLASS_NAME);
        assert_eq!(class.super_class.as_deref(), Some("java/lang/Object"));
        assert_eq!(class.java_version(), "Java 21");
        assert_eq!(class.class_file_version(), "65.0");
        assert_eq!(class.fields.len(), 2);
        assert_eq!(class.fields[0].name, "MAX");
        assert_eq!(class.fields[1].name, "greeting");
        assert_eq!(class.methods.len(), 1);
        assert_eq!(class.methods[0].name, "compute");
        assert_eq!(class.attribute_names, vec!["SourceFile"]);
        assert_eq!(class.constant_pool.valid_count(), 17);
        assert_eq!(
            class.constant_pool.describe(15).as_deref(),
            Some("\"Hello from Recaf Rust!\"")
        );
    }
}
