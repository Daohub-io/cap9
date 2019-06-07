use super::func;
use super::import_entry;
use super::parse_varuint_32;
use super::Cursor;
use super::ImportEntry;
use crate::instructions;
use crate::primitives::CountedList;
use crate::serialization::Deserialize;
#[cfg(not(feature = "std"))]
use pwasm_std::String;
#[cfg(not(feature = "std"))]
use pwasm_std::Vec;
/// A read-only representation of a WASM module. The data is held in WASM binary
/// format in the buffer. All of the functions simply access this buffer. These
/// fields are private as they need initialisation. Currently it only holds
/// references to the sections we care about.
#[derive(Debug, Default)]
pub struct Module<'a> {
    /// A reference to the buffer that actually holds the WASM data.
    buffer: &'a [u8],
    /// The offset into the buffer of the start of the type section
    /// (excluding the section type byte). It therefore points to the size
    /// of the section.
    type_section_offset: Option<usize>,
    /// The offset into the buffer of the start of the import
    /// section (excluding the section type byte). It therefore points to
    /// the size of the section.
    import_section_offset: Option<usize>,
    /// The offset into the buffer of the start of the function
    /// section (excluding the section type byte). It therefore points to
    /// the size of the section.
    function_section_offset: Option<usize>,
    /// The offset into the buffer of the start of the code section
    /// (excluding the section type byte). It therefore points to the size
    /// of the section.
    code_section_offset: Option<usize>,
    /// The offset into the buffer of the start of the table section
    /// (excluding the section type byte). It therefore points to the size
    /// of the section.
    table_section_offset: Option<usize>,
}

impl<'a> Module<'a> {
    /// Create a new `Module` struct using the given buffer.
    pub fn new(buffer: &'a [u8]) -> Self {
        // Create a cursor, with which we will seek over the WASM code in
        // the buffer (self is the buffer, and is read-only).
        let mut cursor = Cursor {
            current_offset: 0,
            body: buffer,
        };

        // The first two steps are to take the magic number and version to
        // check that it is valid wasm. This is not strictly necessary, as
        // it is the job of the runtime to ensure the wasm is valid (ad we
        // rely on that fact), however, it's cheap and allows us prevent
        // future versions of wasm code being deployed (for which our
        // assumptions may not hold).

        // Take the magic number, check that it matches
        if cursor.read_ref_n(4) != &[0, 97, 115, 109] {
            panic!("magic number not found");
        }

        // Take the version, check that it matches
        if cursor.read_ref_n(4) != &[1, 0, 0, 0] {
            panic!("proper version number not found");
        }

        // First we find all of the relevant section offsets.
        let mut type_section_offset: Option<usize> = None;
        let mut import_section_offset: Option<usize> = None;
        let mut function_section_offset: Option<usize> = None;
        let mut code_section_offset: Option<usize> = None;
        let mut table_section_offset: Option<usize> = None;
        while cursor.current_offset < buffer.len() {
            let section: Section = parse_section(&mut cursor);
            // There are many section types we don't care about, for
            // example, Custom sections generally contain debugging symbols
            // and meaningful function names which are irrelevant to the
            // current process. We care only about types, imports,
            // functions, and code.
            match section.type_ {
                SectionType::Type => {
                    if type_section_offset.is_some() {
                        panic!("multiple type sections");
                    }
                    type_section_offset = Some(section.offset);
                }
                SectionType::Import => {
                    if import_section_offset.is_some() {
                        panic!("multiple import sections");
                    }
                    import_section_offset = Some(section.offset);
                }
                SectionType::Function => {
                    if function_section_offset.is_some() {
                        panic!("multiple function sections");
                    }
                    function_section_offset = Some(section.offset);
                }
                SectionType::Code => {
                    if code_section_offset.is_some() {
                        panic!("multiple code sections");
                    }
                    code_section_offset = Some(section.offset);
                }
                SectionType::Table => {
                    if table_section_offset.is_some() {
                        panic!("multiple code sections");
                    }
                    table_section_offset = Some(section.offset);
                }
                // We ignore any section we are not interested in.
                _ => (),
            }
        }
        if cursor.current_offset != buffer.len() {
            panic!("mismatched length");
        }
        Module {
            buffer,
            type_section_offset,
            import_section_offset,
            function_section_offset,
            code_section_offset,
            table_section_offset,
        }
    }

    /// Return an iterator over the imports in the import section. The
    /// imports are in order.
    pub fn imports(&self) -> Option<ImportIterator> {
        // TODO: generalise to SectionIter
        if let Some(imports_offset) = self.import_section_offset {
            Some(ImportIterator::new(self.buffer, imports_offset))
        } else {
            None
        }
    }

    /// Return an iterator over the combined function and code sections.
    /// Individual access to these sections is not currently exposed.
    pub fn functions(&self) -> Option<FunctionIterator> {
        // TODO: generalise to SectionIter
        if let (Some(functions_offset), Some(code_offset)) =
            (self.function_section_offset, self.code_section_offset)
        {
            Some(FunctionIterator::new(
                self.buffer,
                functions_offset,
                code_offset,
            ))
        } else {
            None
        }
    }
}

/// An iterator over the imports in the import section.
pub struct ImportIterator<'a> {
    section_offset: usize,
    offset_into_section: usize,
    buffer: &'a [u8],
    n: u32,
    current_entry: u32,
}

impl<'a> ImportIterator<'a> {
    fn new(buffer: &'a [u8], section_offset: usize) -> Self {
        let mut imports_cursor = Cursor {
            current_offset: section_offset,
            body: buffer,
        };
        // How big is this section in bytes?
        let _section_size = parse_varuint_32(&mut imports_cursor);
        // How many imports do we have?
        let n = parse_varuint_32(&mut imports_cursor);
        ImportIterator {
            section_offset,
            offset_into_section: (imports_cursor.current_offset - section_offset),
            buffer,
            n,
            current_entry: 0,
        }
    }
}

impl<'a> Iterator for ImportIterator<'a> {
    type Item = ImportEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_entry < self.n {
            let mut reader = Cursor {
                current_offset: self.section_offset + self.offset_into_section,
                body: self.buffer,
            };
            let val = parse_import(&mut reader);
            self.offset_into_section = reader.current_offset - self.section_offset;
            self.current_entry += 1;
            Some(val)
        } else {
            None
        }
    }
}

// TODO: ugly function
fn parse_import(cursor: &mut Cursor) -> ImportEntry {
    let mut reader = Cursor {
        current_offset: cursor.current_offset,
        body: cursor.body,
    };
    let import: import_entry::ImportEntry =
        import_entry::ImportEntry::deserialize(&mut reader).expect("counted list");
    let val = ImportEntry {
        mod_name: String::from(import.module()),
        field_name: String::from(import.field()),
    };
    cursor.current_offset = reader.current_offset;
    val
}

/// TODO: this should be made by combining function and code iterators.
/// An iterator over the imports in the import section.
pub struct FunctionIterator<'a> {
    function_section_offset: usize,
    code_section_offset: usize,
    offset_into_function_section: usize,
    offset_into_code_section: usize,
    buffer: &'a [u8],
    n: u32,
    current_entry: u32,
}

impl<'a> FunctionIterator<'a> {
    fn new(buffer: &'a [u8], function_section_offset: usize, code_section_offset: usize) -> Self {
        let mut functions_cursor = Cursor {
            current_offset: function_section_offset,
            body: buffer,
        };
        let mut code_cursor = Cursor {
            current_offset: code_section_offset,
            body: buffer,
        };
        // Get the sizes of the two sections. These aren't important for
        // these, we just need to skip past them.
        let _function_section_size = parse_varuint_32(&mut functions_cursor);
        let _code_section_size = parse_varuint_32(&mut code_cursor);
        let n_functions = parse_varuint_32(&mut functions_cursor);
        let n_bodies = parse_varuint_32(&mut code_cursor);

        // These should be the same, if not, our assumptions are invalid or
        // the WASM is invalid. In either case we need to abort.
        assert_eq!(n_functions, n_bodies);
        FunctionIterator {
            function_section_offset,
            code_section_offset,
            offset_into_function_section: (code_cursor.current_offset - code_section_offset),
            offset_into_code_section: (functions_cursor.current_offset - function_section_offset),
            buffer,
            n: n_functions,
            current_entry: 0,
        }
    }
}

impl<'a> Iterator for FunctionIterator<'a> {
    type Item = Function<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_entry < self.n {
            let mut functions_cursor = Cursor {
                current_offset: self.function_section_offset + self.offset_into_function_section,
                body: self.buffer,
            };
            let mut code_cursor = Cursor {
                current_offset: self.code_section_offset + self.offset_into_code_section,
                body: self.buffer,
            };
            let val = Function {
                function_entry_offset: functions_cursor.current_offset,
                code_entry_offset: code_cursor.current_offset,
                buffer: self.buffer,
            };
            let body_size = parse_varuint_32(&mut code_cursor);
            let f_size = parse_varuint_32(&mut functions_cursor);
            self.offset_into_function_section =
                functions_cursor.current_offset - self.function_section_offset + f_size as usize;
            self.offset_into_code_section =
                code_cursor.current_offset - self.code_section_offset + body_size as usize;
            self.current_entry += 1;
            Some(val)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Function<'a> {
    pub function_entry_offset: usize,
    pub code_entry_offset: usize,
    buffer: &'a [u8],
}

impl<'a> Function<'a> {
    pub fn code(&self) -> Code {
        Code::new(self.buffer)
    }

    pub fn is_syscall(&self, dcall_i: u32, gasleft_i: u32, sender_i: u32) -> bool {
        let mut code_cursor = Cursor {
            current_offset: self.code_entry_offset,
            body: self.buffer,
        };
        let body_size = parse_varuint_32(&mut code_cursor);
        let body = &self.buffer
            [(code_cursor.current_offset)..(code_cursor.current_offset + body_size as usize)];
        let mut code_iter = Code::new(body);

        // Check that no locals are used
        if code_iter.locals.len() > 0 {
            return false;
        }

        // First we need to check that the instructions are correct, that is:
        //   0. call $a
        //   1. call $b
        //   2. get_local 0
        //   3. get_local 1
        //   4. get_local 2
        //   5. get_local 3
        //   6. call $c
        // $a, $b, and $c will be used later.

        //   0. call gasleft
        if let Some(instructions::Instruction::Call(f_ind)) = code_iter.next() {
            if f_ind != gasleft_i {
                return false;
            }
        } else {
            return false;
        }
        //   1. call sender
        if let Some(instructions::Instruction::Call(f_ind)) = code_iter.next() {
            if f_ind != sender_i {
                return false;
            }
        } else {
            return false;
        }
        //   2. get_local 0
        if let Some(instructions::Instruction::GetLocal(0)) = code_iter.next() {
        } else {
            return false;
        }
        //   3. get_local 1
        if let Some(instructions::Instruction::GetLocal(1)) = code_iter.next() {
        } else {
            return false;
        }
        //   4. get_local 2
        if let Some(instructions::Instruction::GetLocal(2)) = code_iter.next() {
        } else {
            return false;
        }
        //   5. get_local 3
        if let Some(instructions::Instruction::GetLocal(3)) = code_iter.next() {
        } else {
            return false;
        }

        //   6. call dcall
        if let Some(instructions::Instruction::Call(f_ind)) = code_iter.next() {
            if f_ind != dcall_i {
                return false;
            }
        } else {
            return false;
        }
        //   7. END
        if let Some(instructions::Instruction::End) = code_iter.next() {
        } else {
            return false;
        }
        // We have checked locals and code, we don't really care abou the type, so
        // we can return true.
        true
    }
    // TODO: we need to account for indirect calls too.
    pub fn contains_grey_call(&self, dcall_i: u32) -> bool {
        let mut code_cursor = Cursor {
            current_offset: self.code_entry_offset,
            body: self.buffer,
        };
        let body_size = parse_varuint_32(&mut code_cursor);
        let body = &self.buffer
            [(code_cursor.current_offset)..(code_cursor.current_offset + body_size as usize)];
        let code_iter = Code::new(body);
        for instruction in code_iter {
            // We only care about Call or CallIndirect instructions
            match instruction {
                instructions::Instruction::Call(f_ind) => {
                    // if f_ind is a grey call then we return true, as we are asking the
                    // question "Does this function contain a call to a greylisted
                    // import?".
                    if f_ind == dcall_i {
                        return true;
                    }
                }
                instructions::Instruction::CallIndirect(_type_index, _table_index) => {
                    // We currently don't have the functionality to check that
                    // tables are safe. For now we will just forbid indirect
                    // calls by assuming any indirect call could be a dcall.
                    return true;
                }
                _ => {}
            }
        }
        // No instructions were greylisted, so we can return false.
        false
    }
}

/// An iterator over the instructions of a function body.
pub struct Code<'a> {
    pub locals: Vec<func::Local>,
    pub current_offset: usize,
    pub body: &'a [u8],
}

impl<'a> Code<'a> {
    pub fn new(body: &'a [u8]) -> Code {
        let mut reader = Cursor {
            current_offset: 0,
            body: body,
        };
        let locals: Vec<func::Local> = CountedList::<func::Local>::deserialize(&mut reader)
            .expect("counted list")
            .into_inner();
        Code {
            locals,
            current_offset: reader.current_offset,
            body: body,
        }
    }
}

impl<'a> Iterator for Code<'a> {
    type Item = crate::instructions::Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_offset < self.body.len() {
            // We need to parse the code into something meaningful
            let mut reader = Cursor {
                current_offset: self.current_offset,
                body: self.body,
            };
            let val = Some(
                crate::instructions::Instruction::deserialize(&mut reader)
                    .expect("expected valid instruction"),
            );
            self.current_offset = reader.current_offset;
            val
        } else {
            None
        }
    }
}

#[derive(Debug)]
enum SectionType {
    Custom,
    Type,
    Import,
    Function,
    Table,
    Memory,
    Global,
    Export,
    Start,
    Element,
    Code,
    Data,
}

#[derive(Debug)]
struct Section {
    type_: SectionType,
    // The offset is the byte offset of the start of this
    // section, i.e. it points directly to the length byte.
    offset: usize,
}

fn parse_section(cursor: &mut Cursor) -> Section {
    let type_n = cursor.read_ref();
    let offset = cursor.current_offset;
    let size_n = parse_varuint_32(cursor);
    let type_ = n_to_section(type_n);
    let section = Section { type_, offset };
    cursor.current_offset += size_n as usize;
    section
}

fn n_to_section(byte: &u8) -> SectionType {
    match byte {
        0 => SectionType::Custom,
        1 => SectionType::Type,
        2 => SectionType::Import,
        3 => SectionType::Function,
        4 => SectionType::Table,
        5 => SectionType::Memory,
        6 => SectionType::Global,
        7 => SectionType::Export,
        8 => SectionType::Start,
        9 => SectionType::Element,
        10 => SectionType::Code,
        11 => SectionType::Data,
        _ => panic!("invalid section type"),
    }
}
