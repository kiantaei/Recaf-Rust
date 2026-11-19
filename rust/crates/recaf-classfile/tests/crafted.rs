//! Integration tests for `recaf-classfile` using hand-crafted class files.
//!
//! The bytes are built programmatically (no binary fixtures) so every test
//! documents exactly which bytes the parser consumes. Byte order follows the
//! spec: magic, minor/major version, `constant_pool_count`, pool entries,
//! then access flags / this / super / interfaces / fields / methods /
//! attributes.

use recaf_classfile::*;

/// Small big-endian byte builder for crafting class files.
struct Bytes(Vec<u8>);

impl Bytes {
    fn new() -> Self {
        Bytes(Vec::new())
    }

    fn u8(&mut self, v: u8) -> &mut Self {
        self.0.push(v);
        self
    }

    fn u16(&mut self, v: u16) -> &mut Self {
        self.0.extend_from_slice(&v.to_be_bytes());
        self
    }

    fn u32(&mut self, v: u32) -> &mut Self {
        self.0.extend_from_slice(&v.to_be_bytes());
        self
    }

    fn raw(&mut self, bytes: &[u8]) -> &mut Self {
        self.0.extend_from_slice(bytes);
        self
    }

    /// Append a Utf8 constant pool entry (indices are assigned in order).
    fn utf8(&mut self, s: &str) -> &mut Self {
        self.u8(tags::UTF8).u16(s.len() as u16).raw(s.as_bytes())
    }

    /// Append a Class constant pool entry pointing at `name_index`.
    fn class(&mut self, name_index: u16) -> &mut Self {
        self.u8(tags::CLASS).u16(name_index)
    }

    /// Write `constant_pool_count`; append pool entries afterwards.
    fn start_pool(&mut self, count: u16) -> &mut Self {
        self.u16(count)
    }

    /// Append access flags, this_class and super_class after the pool.
    fn finish(&mut self, access: u16, this: u16, sup: u16) -> &mut Self {
        self.u16(access).u16(this).u16(sup)
    }

    /// [`Bytes::finish`] plus the four trailing counts (interfaces, fields,
    /// methods, attributes) as zeros — a complete class file with no members.
    fn finish_empty(&mut self, access: u16, this: u16, sup: u16) -> Vec<u8> {
        self.finish(access, this, sup).u16(0).u16(0).u16(0).u16(0);
        self.0.clone()
    }
}

fn header_bytes(minor: u16, major: u16) -> Bytes {
    let mut b = Bytes::new();
    b.u32(MAGIC).u16(minor).u16(major);
    b
}

#[test]
fn parse_minimal_class() {
    // Pool: 1 = Utf8 "RecafDemo", 2 = Class -> 1
    let bytes = header_bytes(0, 52)
        .start_pool(3)
        .utf8("RecafDemo")
        .class(1)
        .finish_empty(0x0021, 2, 0);

    let class = ClassFile::parse(&bytes).expect("minimal class must parse");
    assert_eq!(class.major_version, 52);
    assert_eq!(class.minor_version, 0);
    assert_eq!(class.java_version(), "Java 8");
    assert_eq!(class.class_file_version(), "52.0");
    assert_eq!(class.this_class, "RecafDemo");
    assert_eq!(class.super_class, None);
    assert!(class.interfaces.is_empty());
    assert!(class.fields.is_empty());
    assert!(class.methods.is_empty());
    assert!(class.attribute_names.is_empty());
    assert_eq!(class.constant_pool.count(), 3);
    assert_eq!(class.constant_pool.valid_count(), 2);

    let names = class_flags_to_names(class.access_flags);
    assert!(names.contains(&"public"));
    assert!(names.contains(&"super"));
}

#[test]
fn long_and_double_occupy_two_slots() {
    // Pool: 1 = Long (occupies 1+2), 3 = Double (occupies 3+4),
    //       5 = Utf8 "X", 6 = Class -> 5
    let bytes = header_bytes(0, 61)
        .start_pool(7)
        .u8(tags::LONG)
        .raw(&70_000i64.to_be_bytes())
        .u8(tags::DOUBLE)
        .raw(&3.5f64.to_bits().to_be_bytes())
        .utf8("X")
        .class(5)
        .finish_empty(0x0021, 6, 0);

    let class = ClassFile::parse(&bytes).expect("class with long/double must parse");
    assert_eq!(class.constant_pool.count(), 7);
    assert_eq!(class.constant_pool.valid_count(), 4);

    assert!(matches!(
        class.constant_pool.get(1),
        Some(ConstantPoolEntry::Long(70_000))
    ));
    assert!(matches!(
        class.constant_pool.get(3),
        Some(ConstantPoolEntry::Double(_))
    ));
    assert_eq!(class.constant_pool.get(2), None);
    assert_eq!(class.constant_pool.get(4), None);
}

#[test]
fn interfaces_fields_methods_and_attributes_are_parsed() {
    // Pool layout:
    //   1 Utf8 "Demo"            2 Class -> 1
    //   3 Utf8 "java/lang/Runnable"  4 Class -> 3
    //   5 Utf8 "counter"         6 Utf8 "I"
    //   7 Utf8 "run"             8 Utf8 "()V"
    //   9 Utf8 "Code"            10 Utf8 "Demo.java"
    //  11 Utf8 "SourceFile"
    let mut b = header_bytes(0, 52);
    b.start_pool(12);
    b.utf8("Demo"); // 1
    b.class(1); // 2
    b.utf8("java/lang/Runnable"); // 3
    b.class(3); // 4
    b.utf8("counter"); // 5
    b.utf8("I"); // 6
    b.utf8("run"); // 7
    b.utf8("()V"); // 8
    b.utf8("Code"); // 9
    b.utf8("Demo.java"); // 10
    b.utf8("SourceFile"); // 11

    // access / this / super via finish, then the rest by hand:
    let mut bytes = b.finish(0x0021, 2, 0).0.clone();
    bytes.extend_from_slice(&1u16.to_be_bytes()); // 1 interface
    bytes.extend_from_slice(&4u16.to_be_bytes()); // -> Class 4
    bytes.extend_from_slice(&1u16.to_be_bytes()); // 1 field
                                                  // field: public static final, name "counter", desc "I", 0 attributes
    bytes.extend_from_slice(&0x0019u16.to_be_bytes());
    bytes.extend_from_slice(&5u16.to_be_bytes());
    bytes.extend_from_slice(&6u16.to_be_bytes());
    bytes.extend_from_slice(&0u16.to_be_bytes());
    bytes.extend_from_slice(&1u16.to_be_bytes()); // 1 method
                                                  // method: public, name "run", desc "()V", 1 attribute "Code" (len 0)
    bytes.extend_from_slice(&0x0001u16.to_be_bytes());
    bytes.extend_from_slice(&7u16.to_be_bytes());
    bytes.extend_from_slice(&8u16.to_be_bytes());
    bytes.extend_from_slice(&1u16.to_be_bytes());
    bytes.extend_from_slice(&9u16.to_be_bytes()); // "Code"
    bytes.extend_from_slice(&0u32.to_be_bytes());
    bytes.extend_from_slice(&1u16.to_be_bytes()); // 1 class attribute
                                                  // attribute: "SourceFile", len 2, content -> Utf8 #10 ("Demo.java")
    bytes.extend_from_slice(&11u16.to_be_bytes());
    bytes.extend_from_slice(&2u32.to_be_bytes());
    bytes.extend_from_slice(&10u16.to_be_bytes());

    let class = ClassFile::parse(&bytes).expect("full class must parse");
    assert_eq!(class.java_version(), "Java 8");
    assert_eq!(class.this_class, "Demo");
    assert_eq!(class.super_class, None);
    assert_eq!(class.interfaces, vec!["java/lang/Runnable"]);

    assert_eq!(class.fields.len(), 1);
    let field = &class.fields[0];
    assert_eq!(field.name, "counter");
    assert_eq!(field.descriptor, "I");
    assert_eq!(
        field_flags_to_names(field.access_flags),
        vec!["public", "static", "final"]
    );
    assert!(field.attribute_names.is_empty());

    assert_eq!(class.methods.len(), 1);
    let method = &class.methods[0];
    assert_eq!(method.name, "run");
    assert_eq!(method.descriptor, "()V");
    assert_eq!(method_flags_to_names(method.access_flags), vec!["public"]);
    assert_eq!(method.attribute_names, vec!["Code"]);

    assert_eq!(class.attribute_names, vec!["SourceFile"]);
}

#[test]
fn bad_magic_is_rejected() {
    let mut bytes = vec![0u8; 32];
    bytes[0] = 0xDE;
    bytes[1] = 0xAD;
    bytes[2] = 0xBE;
    bytes[3] = 0xEF;

    let err = ClassFile::parse(&bytes).expect_err("bad magic must be rejected");
    assert!(err.message.contains("bad magic"), "{}", err.message);
    assert_eq!(err.offset, 0);
}

#[test]
fn truncated_file_is_rejected() {
    let mut bytes = header_bytes(0, 52)
        .start_pool(3)
        .utf8("RecafDemo")
        .class(1)
        .finish_empty(0x0021, 2, 0);
    bytes.truncate(bytes.len() - 2); // cut inside this_class/super_class

    let err = ClassFile::parse(&bytes).expect_err("truncated file must be rejected");
    assert!(
        err.message.contains("unexpected end of file"),
        "{}",
        err.message
    );
}

#[test]
fn unknown_tag_is_rejected() {
    // Tag 2 is not defined by the JVM spec.
    let bytes = header_bytes(0, 52)
        .start_pool(2)
        .u8(2)
        .u16(0)
        .finish_empty(0x0021, 1, 0);

    let err = ClassFile::parse(&bytes).expect_err("unknown tag must be rejected");
    assert!(
        err.message.contains("unknown constant pool tag 2"),
        "{}",
        err.message
    );
}

#[test]
fn trailing_bytes_are_rejected() {
    let mut bytes = header_bytes(0, 52)
        .start_pool(3)
        .utf8("RecafDemo")
        .class(1)
        .finish_empty(0x0021, 2, 0);
    bytes.push(0xFF);

    let err = ClassFile::parse(&bytes).expect_err("trailing bytes must be rejected");
    assert!(err.message.contains("trailing byte"), "{}", err.message);
}

#[test]
fn modified_utf8_null_and_accents() {
    // 0xC0 0x80 is the modified UTF-8 encoding of U+0000,
    // 0xC3 0xA9 is "é" in ordinary UTF-8.
    let payload: &[u8] = &[0xC0, 0x80, 0xC3, 0xA9, b'A'];

    let bytes = header_bytes(0, 52)
        .start_pool(3)
        .u8(tags::UTF8)
        .u16(payload.len() as u16)
        .raw(payload)
        .class(1)
        .finish_empty(0x0021, 2, 0);

    let class = ClassFile::parse(&bytes).expect("modified utf8 class must parse");
    assert_eq!(class.this_class, "\u{0}éA");
}

#[test]
fn describe_resolves_references() {
    // Pool: 1 Utf8 "java/lang/Object", 2 Class -> 1,
    //       3 Utf8 "<init>", 4 Utf8 "()V", 5 NameAndType(3, 4),
    //       6 Methodref(2, 5)
    let bytes = header_bytes(0, 52)
        .start_pool(7)
        .utf8("java/lang/Object")
        .class(1)
        .utf8("<init>")
        .utf8("()V")
        .u8(tags::NAME_AND_TYPE)
        .u16(3)
        .u16(4)
        .u8(tags::METHODREF)
        .u16(2)
        .u16(5)
        .finish_empty(0x0021, 2, 0);

    let class = ClassFile::parse(&bytes).expect("describe fixture must parse");
    let pool = &class.constant_pool;
    assert_eq!(pool.describe(1), Some("\"java/lang/Object\"".to_string()));
    assert_eq!(pool.describe(2), Some("java/lang/Object".to_string()));
    assert_eq!(pool.describe(5), Some("<init>:()V".to_string()));
    assert_eq!(
        pool.describe(6),
        Some("java/lang/Object.<init>:()V".to_string())
    );
    assert_eq!(pool.get(6).map(|e| e.tag_name()), Some("Methodref"));
}
