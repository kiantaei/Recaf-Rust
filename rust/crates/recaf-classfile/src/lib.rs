//! A dependency-free parser for JVM class files.
//!
//! `recaf-classfile` is the first core piece of Recaf-Rust written in Rust.
//! It reads the class file format defined by the JVM specification (chapter 4)
//! into a small, inspection-friendly data model that the GPUI front end
//! (`recaf-gpui`) renders.
//!
//! The parser is deliberately strict: malformed input is rejected with a
//! message that names the problem and the byte offset where it happened,
//! instead of guessing or panicking. It never reads past the end of the
//! buffer and never panics on adversarial input.

use std::fmt;

/// Magic number every class file starts with (JVM spec §4.1).
pub const MAGIC: u32 = 0xCAFE_BABE;

/// Minimum `major_version` the JVM specification allows (Java 1.1).
pub const MIN_MAJOR_VERSION: u16 = 45;

/// Access flags used by classes, fields and methods (JVM spec tables 4.1-B,
/// 4.6-A and 4.7-A).
pub mod access {
    pub const PUBLIC: u16 = 0x0001;
    pub const PRIVATE: u16 = 0x0002;
    pub const PROTECTED: u16 = 0x0004;
    pub const STATIC: u16 = 0x0008;
    pub const FINAL: u16 = 0x0010;
    pub const SUPER: u16 = 0x0020;
    pub const SYNCHRONIZED: u16 = 0x0020;
    pub const VOLATILE: u16 = 0x0040;
    pub const BRIDGE: u16 = 0x0040;
    pub const TRANSIENT: u16 = 0x0080;
    pub const VARARGS: u16 = 0x0080;
    pub const NATIVE: u16 = 0x0100;
    pub const INTERFACE: u16 = 0x0200;
    pub const ABSTRACT: u16 = 0x0400;
    pub const STRICT: u16 = 0x0800;
    pub const SYNTHETIC: u16 = 0x1000;
    pub const ANNOTATION: u16 = 0x2000;
    pub const ENUM: u16 = 0x4000;
    pub const MODULE: u16 = 0x8000;
}

/// Constant pool entry tags (JVM spec table 4.4-A).
pub mod tags {
    pub const UTF8: u8 = 1;
    pub const INTEGER: u8 = 3;
    pub const FLOAT: u8 = 4;
    pub const LONG: u8 = 5;
    pub const DOUBLE: u8 = 6;
    pub const CLASS: u8 = 7;
    pub const STRING: u8 = 8;
    pub const FIELDREF: u8 = 9;
    pub const METHODREF: u8 = 10;
    pub const INTERFACE_METHODREF: u8 = 11;
    pub const NAME_AND_TYPE: u8 = 12;
    pub const METHOD_HANDLE: u8 = 15;
    pub const METHOD_TYPE: u8 = 16;
    pub const DYNAMIC: u8 = 17;
    pub const INVOKE_DYNAMIC: u8 = 18;
    pub const MODULE: u8 = 19;
    pub const PACKAGE: u8 = 20;
}

/// An error produced while parsing a class file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassFileError {
    /// Human readable description of what went wrong.
    pub message: String,
    /// Byte offset in the input where the problem was detected.
    pub offset: usize,
}

impl ClassFileError {
    fn new(message: impl Into<String>, offset: usize) -> Self {
        Self {
            message: message.into(),
            offset,
        }
    }
}

impl fmt::Display for ClassFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "malformed class file at byte {}: {}",
            self.offset, self.message
        )
    }
}

impl std::error::Error for ClassFileError {}

/// One entry of the constant pool (JVM spec §4.4).
#[derive(Debug, Clone, PartialEq)]
pub enum ConstantPoolEntry {
    /// A string in JVM "modified UTF-8" encoding, already decoded.
    Utf8(String),
    Integer(i32),
    Float(f32),
    /// Occupies two constant pool slots (JVM spec §4.4.5).
    Long(i64),
    /// Occupies two constant pool slots (JVM spec §4.4.5).
    Double(f64),
    Class {
        name_index: u16,
    },
    String {
        string_index: u16,
    },
    Fieldref {
        class_index: u16,
        name_and_type_index: u16,
    },
    Methodref {
        class_index: u16,
        name_and_type_index: u16,
    },
    InterfaceMethodref {
        class_index: u16,
        name_and_type_index: u16,
    },
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    },
    MethodHandle {
        reference_kind: u8,
        reference_index: u16,
    },
    MethodType {
        descriptor_index: u16,
    },
    Dynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
    InvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
    Module {
        name_index: u16,
    },
    Package {
        name_index: u16,
    },
}

impl ConstantPoolEntry {
    /// The spec name of this entry's tag, e.g. `"Utf8"` or `"Methodref"`.
    pub fn tag_name(&self) -> &'static str {
        match self {
            ConstantPoolEntry::Utf8(_) => "Utf8",
            ConstantPoolEntry::Integer(_) => "Integer",
            ConstantPoolEntry::Float(_) => "Float",
            ConstantPoolEntry::Long(_) => "Long",
            ConstantPoolEntry::Double(_) => "Double",
            ConstantPoolEntry::Class { .. } => "Class",
            ConstantPoolEntry::String { .. } => "String",
            ConstantPoolEntry::Fieldref { .. } => "Fieldref",
            ConstantPoolEntry::Methodref { .. } => "Methodref",
            ConstantPoolEntry::InterfaceMethodref { .. } => "InterfaceMethodref",
            ConstantPoolEntry::NameAndType { .. } => "NameAndType",
            ConstantPoolEntry::MethodHandle { .. } => "MethodHandle",
            ConstantPoolEntry::MethodType { .. } => "MethodType",
            ConstantPoolEntry::Dynamic { .. } => "Dynamic",
            ConstantPoolEntry::InvokeDynamic { .. } => "InvokeDynamic",
            ConstantPoolEntry::Module { .. } => "Module",
            ConstantPoolEntry::Package { .. } => "Package",
        }
    }
}

impl fmt::Display for ConstantPoolEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConstantPoolEntry::Utf8(s) => write!(f, "\"{s}\""),
            ConstantPoolEntry::Integer(v) => write!(f, "{v}"),
            ConstantPoolEntry::Long(v) => write!(f, "{v}"),
            ConstantPoolEntry::Float(v) => write!(f, "{v}"),
            ConstantPoolEntry::Double(v) => write!(f, "{v}"),
            ConstantPoolEntry::Class { name_index }
            | ConstantPoolEntry::Module { name_index }
            | ConstantPoolEntry::Package { name_index } => write!(f, "-> #{name_index}"),
            ConstantPoolEntry::String { string_index } => write!(f, "-> #{string_index}"),
            ConstantPoolEntry::Fieldref {
                class_index,
                name_and_type_index,
            }
            | ConstantPoolEntry::Methodref {
                class_index,
                name_and_type_index,
            }
            | ConstantPoolEntry::InterfaceMethodref {
                class_index,
                name_and_type_index,
            } => write!(f, "class #{class_index}, name/type #{name_and_type_index}"),
            ConstantPoolEntry::NameAndType {
                name_index,
                descriptor_index,
            } => write!(f, "#{name_index}:{descriptor_index}"),
            ConstantPoolEntry::MethodHandle {
                reference_kind,
                reference_index,
            } => write!(f, "kind {reference_kind} -> #{reference_index}"),
            ConstantPoolEntry::MethodType { descriptor_index } => {
                write!(f, "-> #{descriptor_index}")
            }
            ConstantPoolEntry::Dynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            }
            | ConstantPoolEntry::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => write!(
                f,
                "bsm #{bootstrap_method_attr_index}, name/type #{name_and_type_index}"
            ),
        }
    }
}

/// The constant pool of a class file.
///
/// Entries are addressed by their 1-based spec index. Index 0 and the
/// "unused" second slot after a `Long`/`Double` map to `None`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConstantPool {
    count: u16,
    entries: Vec<Option<ConstantPoolEntry>>,
}

impl ConstantPool {
    /// The raw `constant_pool_count` value from the file.
    pub fn count(&self) -> u16 {
        self.count
    }

    /// How many entries actually hold a constant (excludes index 0 and the
    /// phantom slots after `Long`/`Double`).
    pub fn valid_count(&self) -> usize {
        self.entries.iter().filter(|e| e.is_some()).count()
    }

    /// The entry at a constant pool index, if one exists there.
    pub fn get(&self, index: u16) -> Option<&ConstantPoolEntry> {
        self.entries.get(index as usize).and_then(|e| e.as_ref())
    }

    /// Iterate all valid entries as `(spec_index, entry)` pairs.
    pub fn entries(&self) -> impl Iterator<Item = (u16, &ConstantPoolEntry)> + '_ {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(i, e)| e.as_ref().map(|e| (i as u16, e)))
    }

    /// The `Utf8` string at `index`, if it is one.
    pub fn utf8(&self, index: u16) -> Option<&str> {
        match self.get(index) {
            Some(ConstantPoolEntry::Utf8(s)) => Some(s),
            _ => None,
        }
    }

    /// The class name (e.g. `"java/lang/Object"`) referenced by a `Class`
    /// entry at `index`.
    pub fn class_name(&self, index: u16) -> Option<&str> {
        match self.get(index) {
            Some(ConstantPoolEntry::Class { name_index }) => self.utf8(*name_index),
            _ => None,
        }
    }

    /// `"name:descriptor"` for a `NameAndType` entry at `index`.
    pub fn name_and_type(&self, index: u16) -> Option<String> {
        match self.get(index) {
            Some(ConstantPoolEntry::NameAndType {
                name_index,
                descriptor_index,
            }) => {
                let name = self.utf8(*name_index)?;
                let descriptor = self.utf8(*descriptor_index)?;
                Some(format!("{name}:{descriptor}"))
            }
            _ => None,
        }
    }

    /// A fully resolved, human readable description of the entry at `index`.
    ///
    /// References are followed, so a `Methodref` renders as
    /// `"java/lang/Object.<init>:()V"` rather than a pair of raw indices.
    /// Returns `None` when the index has no valid entry.
    pub fn describe(&self, index: u16) -> Option<String> {
        let entry = self.get(index)?.clone();
        match entry {
            ConstantPoolEntry::Utf8(s) => Some(format!("\"{s}\"")),
            ConstantPoolEntry::Integer(v) => Some(v.to_string()),
            ConstantPoolEntry::Long(v) => Some(v.to_string()),
            ConstantPoolEntry::Float(v) => Some(v.to_string()),
            ConstantPoolEntry::Double(v) => Some(v.to_string()),
            ConstantPoolEntry::Class { name_index } => self.utf8(name_index).map(str::to_owned),
            ConstantPoolEntry::String { string_index } => {
                self.utf8(string_index).map(|s| format!("\"{s}\""))
            }
            ConstantPoolEntry::Fieldref {
                class_index,
                name_and_type_index,
            }
            | ConstantPoolEntry::Methodref {
                class_index,
                name_and_type_index,
            }
            | ConstantPoolEntry::InterfaceMethodref {
                class_index,
                name_and_type_index,
            } => {
                let class = self.class_name(class_index)?;
                let member = self.name_and_type(name_and_type_index)?;
                Some(format!("{class}.{member}"))
            }
            ConstantPoolEntry::NameAndType { .. } => self.name_and_type(index),
            ConstantPoolEntry::MethodHandle {
                reference_kind,
                reference_index,
            } => {
                let target = self.describe(reference_index)?;
                Some(format!("kind {reference_kind} {target}"))
            }
            ConstantPoolEntry::MethodType { descriptor_index } => {
                self.utf8(descriptor_index).map(str::to_owned)
            }
            ConstantPoolEntry::Dynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            }
            | ConstantPoolEntry::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => {
                let member = self.name_and_type(name_and_type_index)?;
                Some(format!("bsm #{bootstrap_method_attr_index}: {member}"))
            }
            ConstantPoolEntry::Module { name_index }
            | ConstantPoolEntry::Package { name_index } => self.utf8(name_index).map(str::to_owned),
        }
    }
}

/// A summary of one field or method of the class.
///
/// Attribute *contents* are not parsed yet — only their names are recorded —
/// which keeps this crate small; the GPUI front end renders the summaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberSummary {
    pub access_flags: u16,
    pub name: String,
    pub descriptor: String,
    pub attribute_names: Vec<String>,
}

/// A parsed JVM class file.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassFile {
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: ConstantPool,
    pub access_flags: u16,
    /// The fully qualified binary name, e.g. `"org/recaf/demo/Demo"`.
    pub this_class: String,
    /// `None` only for `java/lang/Object` itself, whose `super_class` index
    /// is 0.
    pub super_class: Option<String>,
    pub interfaces: Vec<String>,
    pub fields: Vec<MemberSummary>,
    pub methods: Vec<MemberSummary>,
    pub attribute_names: Vec<String>,
}

impl ClassFile {
    /// Parse a class file from its raw bytes.
    pub fn parse(bytes: &[u8]) -> Result<Self, ClassFileError> {
        let mut r = Reader::new(bytes);

        if r.remaining() < 10 {
            return Err(ClassFileError::new(
                format!(
                    "file too small to be a class file ({} bytes, need at least 10)",
                    r.remaining()
                ),
                0,
            ));
        }

        let magic = r.u32()?;
        if magic != MAGIC {
            return Err(ClassFileError::new(
                format!("bad magic number: expected 0xCAFEBABE, found {magic:#010x}"),
                0,
            ));
        }

        let minor_version = r.u16()?;
        let major_version = r.u16()?;
        if major_version < MIN_MAJOR_VERSION {
            return Err(ClassFileError::new(
                format!(
                    "unsupported major version {major_version} (must be >= {MIN_MAJOR_VERSION})"
                ),
                4,
            ));
        }

        let cp_count = r.u16()?;
        if cp_count == 0 {
            return Err(ClassFileError::new(
                "constant_pool_count must be at least 1",
                r.pos(),
            ));
        }

        let mut entries: Vec<Option<ConstantPoolEntry>> = vec![None; cp_count as usize];
        let mut index: u32 = 1;
        while index < cp_count as u32 {
            let tag_offset = r.pos();
            let tag = r.u8()?;
            let entry = match tag {
                tags::UTF8 => {
                    let length = r.u16()? as usize;
                    let raw = r.take(length)?;
                    ConstantPoolEntry::Utf8(decode_modified_utf8(raw))
                }
                tags::INTEGER => ConstantPoolEntry::Integer(be_i32(r.take(4)?)),
                tags::FLOAT => ConstantPoolEntry::Float(f32::from_bits(be_u32(r.take(4)?))),
                tags::LONG => ConstantPoolEntry::Long(be_i64(r.take(8)?)),
                tags::DOUBLE => ConstantPoolEntry::Double(f64::from_bits(be_u64(r.take(8)?))),
                tags::CLASS => ConstantPoolEntry::Class {
                    name_index: r.u16()?,
                },
                tags::STRING => ConstantPoolEntry::String {
                    string_index: r.u16()?,
                },
                tags::FIELDREF => ConstantPoolEntry::Fieldref {
                    class_index: r.u16()?,
                    name_and_type_index: r.u16()?,
                },
                tags::METHODREF => ConstantPoolEntry::Methodref {
                    class_index: r.u16()?,
                    name_and_type_index: r.u16()?,
                },
                tags::INTERFACE_METHODREF => ConstantPoolEntry::InterfaceMethodref {
                    class_index: r.u16()?,
                    name_and_type_index: r.u16()?,
                },
                tags::NAME_AND_TYPE => ConstantPoolEntry::NameAndType {
                    name_index: r.u16()?,
                    descriptor_index: r.u16()?,
                },
                tags::METHOD_HANDLE => {
                    let reference_kind = r.u8()?;
                    if !(1..=9).contains(&reference_kind) {
                        return Err(ClassFileError::new(
                            format!("invalid MethodHandle reference_kind {reference_kind} (must be 1..=9)"),
                            tag_offset,
                        ));
                    }
                    ConstantPoolEntry::MethodHandle {
                        reference_kind,
                        reference_index: r.u16()?,
                    }
                }
                tags::METHOD_TYPE => ConstantPoolEntry::MethodType {
                    descriptor_index: r.u16()?,
                },
                tags::DYNAMIC => ConstantPoolEntry::Dynamic {
                    bootstrap_method_attr_index: r.u16()?,
                    name_and_type_index: r.u16()?,
                },
                tags::INVOKE_DYNAMIC => ConstantPoolEntry::InvokeDynamic {
                    bootstrap_method_attr_index: r.u16()?,
                    name_and_type_index: r.u16()?,
                },
                tags::MODULE => ConstantPoolEntry::Module {
                    name_index: r.u16()?,
                },
                tags::PACKAGE => ConstantPoolEntry::Package {
                    name_index: r.u16()?,
                },
                other => {
                    return Err(ClassFileError::new(
                        format!("unknown constant pool tag {other}"),
                        tag_offset,
                    ));
                }
            };
            entries[index as usize] = Some(entry);
            index += match entries[index as usize] {
                Some(ConstantPoolEntry::Long(_)) | Some(ConstantPoolEntry::Double(_)) => 2,
                _ => 1,
            };
        }

        let constant_pool = ConstantPool {
            count: cp_count,
            entries,
        };

        let access_flags = r.u16()?;
        let this_index = r.u16()?;
        let super_index = r.u16()?;

        let this_class = resolve_class(&constant_pool, this_index, r.pos(), "this_class")?;
        let super_class = if super_index == 0 {
            None
        } else {
            Some(resolve_class(
                &constant_pool,
                super_index,
                r.pos(),
                "super_class",
            )?)
        };

        let interface_count = r.u16()?;
        let mut interfaces = Vec::with_capacity(interface_count as usize);
        for _ in 0..interface_count {
            let index = r.u16()?;
            interfaces.push(resolve_class(&constant_pool, index, r.pos(), "interface")?);
        }

        let field_count = r.u16()?;
        let mut fields = Vec::with_capacity(field_count as usize);
        for _ in 0..field_count {
            fields.push(parse_member(&mut r, &constant_pool)?);
        }

        let method_count = r.u16()?;
        let mut methods = Vec::with_capacity(method_count as usize);
        for _ in 0..method_count {
            methods.push(parse_member(&mut r, &constant_pool)?);
        }

        let attribute_count = r.u16()?;
        let mut attribute_names = Vec::with_capacity(attribute_count as usize);
        for _ in 0..attribute_count {
            attribute_names.push(parse_attribute_name(&mut r, &constant_pool)?);
        }

        if r.remaining() != 0 {
            return Err(ClassFileError::new(
                format!(
                    "{} trailing byte(s) after the last class attribute",
                    r.remaining()
                ),
                r.pos(),
            ));
        }

        Ok(ClassFile {
            minor_version,
            major_version,
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attribute_names,
        })
    }

    /// `"Java 21"` style label for this file's `major_version`.
    pub fn java_version(&self) -> String {
        java_version_for_major(self.major_version)
    }

    /// `"65.0"` style version string (`major.minor`).
    pub fn class_file_version(&self) -> String {
        format!("{}.{}", self.major_version, self.minor_version)
    }
}

/// Map a class file `major_version` to a human readable Java release name.
pub fn java_version_for_major(major: u16) -> String {
    match major {
        45..=51 => format!("Java 1.{}", major - 44),
        m if m >= 52 => format!("Java {}", m - 44),
        _ => format!("unknown (major {major})"),
    }
}

/// Names of the access flags set on a class (JVM spec table 4.1-B).
pub fn class_flags_to_names(flags: u16) -> Vec<&'static str> {
    let mut names = Vec::new();
    for (bit, name) in [
        (access::PUBLIC, "public"),
        (access::FINAL, "final"),
        (access::SUPER, "super"),
        (access::INTERFACE, "interface"),
        (access::ABSTRACT, "abstract"),
        (access::SYNTHETIC, "synthetic"),
        (access::ANNOTATION, "annotation"),
        (access::ENUM, "enum"),
        (access::MODULE, "module"),
    ] {
        if flags & bit != 0 {
            names.push(name);
        }
    }
    names
}

/// Names of the access flags set on a field (JVM spec table 4.6-A).
pub fn field_flags_to_names(flags: u16) -> Vec<&'static str> {
    let mut names = Vec::new();
    for (bit, name) in [
        (access::PUBLIC, "public"),
        (access::PRIVATE, "private"),
        (access::PROTECTED, "protected"),
        (access::STATIC, "static"),
        (access::FINAL, "final"),
        (access::VOLATILE, "volatile"),
        (access::TRANSIENT, "transient"),
        (access::SYNTHETIC, "synthetic"),
        (access::ENUM, "enum"),
    ] {
        if flags & bit != 0 {
            names.push(name);
        }
    }
    names
}

/// Names of the access flags set on a method (JVM spec table 4.6-A).
pub fn method_flags_to_names(flags: u16) -> Vec<&'static str> {
    let mut names = Vec::new();
    for (bit, name) in [
        (access::PUBLIC, "public"),
        (access::PRIVATE, "private"),
        (access::PROTECTED, "protected"),
        (access::STATIC, "static"),
        (access::FINAL, "final"),
        (access::SYNCHRONIZED, "synchronized"),
        (access::BRIDGE, "bridge"),
        (access::VARARGS, "varargs"),
        (access::NATIVE, "native"),
        (access::ABSTRACT, "abstract"),
        (access::STRICT, "strict"),
        (access::SYNTHETIC, "synthetic"),
    ] {
        if flags & bit != 0 {
            names.push(name);
        }
    }
    names
}

fn resolve_class(
    pool: &ConstantPool,
    index: u16,
    offset: usize,
    context: &str,
) -> Result<String, ClassFileError> {
    pool.class_name(index).map(str::to_owned).ok_or_else(|| {
        ClassFileError::new(
            format!("{context}: constant pool entry #{index} does not reference a Class entry"),
            offset,
        )
    })
}

fn resolve_utf8(
    pool: &ConstantPool,
    index: u16,
    offset: usize,
    context: &str,
) -> Result<String, ClassFileError> {
    pool.utf8(index).map(str::to_owned).ok_or_else(|| {
        ClassFileError::new(
            format!("{context}: constant pool entry #{index} does not reference a Utf8 entry"),
            offset,
        )
    })
}

fn parse_member(r: &mut Reader<'_>, pool: &ConstantPool) -> Result<MemberSummary, ClassFileError> {
    let offset = r.pos();
    let access_flags = r.u16()?;
    let name_index = r.u16()?;
    let descriptor_index = r.u16()?;
    let attribute_count = r.u16()?;

    let mut attribute_names = Vec::with_capacity(attribute_count as usize);
    for _ in 0..attribute_count {
        attribute_names.push(parse_attribute_name(r, pool)?);
    }

    Ok(MemberSummary {
        access_flags,
        name: resolve_utf8(pool, name_index, offset, "member name")?,
        descriptor: resolve_utf8(pool, descriptor_index, offset, "member descriptor")?,
        attribute_names,
    })
}

fn parse_attribute_name(r: &mut Reader<'_>, pool: &ConstantPool) -> Result<String, ClassFileError> {
    let offset = r.pos();
    let name_index = r.u16()?;
    let length = r.u32()? as usize;
    r.skip(length)?;
    resolve_utf8(pool, name_index, offset, "attribute name")
}

/// Little cursor over the input with bounds-checked reads.
struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn pos(&self) -> usize {
        self.pos
    }

    fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], ClassFileError> {
        if self.remaining() < n {
            return Err(ClassFileError::new(
                format!(
                    "unexpected end of file: needed {n} byte(s), only {} remain(s)",
                    self.remaining()
                ),
                self.pos,
            ));
        }
        let slice = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    fn skip(&mut self, n: usize) -> Result<(), ClassFileError> {
        self.take(n).map(|_| ())
    }

    fn u8(&mut self) -> Result<u8, ClassFileError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, ClassFileError> {
        let bytes = self.take(2)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn u32(&mut self) -> Result<u32, ClassFileError> {
        Ok(be_u32(self.take(4)?))
    }
}

fn be_u32(bytes: &[u8]) -> u32 {
    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn be_u64(bytes: &[u8]) -> u64 {
    u64::from_be_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

fn be_i32(bytes: &[u8]) -> i32 {
    be_u32(bytes) as i32
}

fn be_i64(bytes: &[u8]) -> i64 {
    be_u64(bytes) as i64
}

/// Decode JVM "modified UTF-8" (JVM spec §4.4.7).
///
/// Differences from standard UTF-8: U+0000 is encoded as `0xC0 0x80` and
/// supplementary characters appear as two 3-byte surrogate sequences.
/// Decoding goes through UTF-16 code units so pairs recombine naturally;
/// any invalid sequence becomes U+FFFD instead of failing the parse.
fn decode_modified_utf8(bytes: &[u8]) -> String {
    let mut units: Vec<u16> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let b0 = bytes[i];
        let unit = match b0 {
            0x01..=0x7F => {
                i += 1;
                b0 as u16
            }
            0xC0 if i + 1 < bytes.len() && bytes[i + 1] == 0x80 => {
                i += 2;
                0u16
            }
            0xC2..=0xDF if i + 1 < bytes.len() && bytes[i + 1] & 0xC0 == 0x80 => {
                let v = (((b0 & 0x1F) as u16) << 6) | (bytes[i + 1] & 0x3F) as u16;
                i += 2;
                v
            }
            0xE0..=0xEF
                if i + 2 < bytes.len()
                    && bytes[i + 1] & 0xC0 == 0x80
                    && bytes[i + 2] & 0xC0 == 0x80 =>
            {
                let v = (((b0 & 0x0F) as u16) << 12)
                    | (((bytes[i + 1] & 0x3F) as u16) << 6)
                    | (bytes[i + 2] & 0x3F) as u16;
                i += 3;
                v
            }
            _ => {
                i += 1;
                0xFFFD
            }
        };
        units.push(unit);
    }
    String::from_utf16_lossy(&units)
}
