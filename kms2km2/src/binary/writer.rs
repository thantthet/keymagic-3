use keymagic_core::*;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;

pub struct Km2Writer<W: Write> {
    writer: W,
}

impl<W: Write> Km2Writer<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn write_km2_file(mut self, km2: &Km2File) -> std::result::Result<(), KmsError> {
        // Write header
        self.write_header(&km2.header)?;
        
        // Write strings
        for string in &km2.strings {
            self.write_string(&string.value)?;
        }
        
        // Write info entries
        for info in &km2.info {
            self.write_info(info)?;
        }
        
        // Write rules
        for rule in &km2.rules {
            self.write_rule(rule)?;
        }
        
        Ok(())
    }

    fn write_header(&mut self, header: &FileHeader) -> std::result::Result<(), KmsError> {
        // Magic code
        self.writer.write_all(&header.magic_code)?;
        
        // Version
        self.writer.write_u8(header.major_version)?;
        self.writer.write_u8(header.minor_version)?;
        
        // Counts
        self.writer.write_u16::<LittleEndian>(header.string_count)?;
        self.writer.write_u16::<LittleEndian>(header.info_count)?;
        self.writer.write_u16::<LittleEndian>(header.rule_count)?;
        
        // Layout options
        self.writer.write_u8(header.layout_options.track_caps)?;
        self.writer.write_u8(header.layout_options.auto_bksp)?;
        self.writer.write_u8(header.layout_options.eat)?;
        self.writer.write_u8(header.layout_options.pos_based)?;
        self.writer.write_u8(header.layout_options.right_alt)?;
        
        // Padding byte to match C++ struct alignment
        self.writer.write_u8(0)?;
        
        Ok(())
    }

    fn write_string(&mut self, s: &str) -> std::result::Result<(), KmsError> {
        let utf16: Vec<u16> = s.encode_utf16().collect();
        
        // Write length (number of UTF-16 code units)
        self.writer.write_u16::<LittleEndian>(utf16.len() as u16)?;
        
        // Write UTF-16LE data
        for code_unit in utf16 {
            self.writer.write_u16::<LittleEndian>(code_unit)?;
        }
        
        Ok(())
    }

    fn write_info(&mut self, info: &InfoEntry) -> std::result::Result<(), KmsError> {
        // Write ID (4 bytes)
        self.writer.write_all(&info.id)?;
        
        // Write length
        self.writer.write_u16::<LittleEndian>(info.data.len() as u16)?;
        
        // Write data
        self.writer.write_all(&info.data)?;
        
        Ok(())
    }

    fn write_rule(&mut self, rule: &Rule) -> std::result::Result<(), KmsError> {
        // Write LHS
        self.write_rule_elements(&rule.lhs)?;
        
        // Write RHS
        self.write_rule_elements(&rule.rhs)?;
        
        Ok(())
    }

    fn write_rule_elements(&mut self, elements: &[BinaryFormatElement]) -> std::result::Result<(), KmsError> {
        // Calculate total size in opcodes
        let mut size = 0u16;
        for elem in elements {
            size += self.element_size(elem);
        }
        
        // Write size
        self.writer.write_u16::<LittleEndian>(size)?;
        
        // Write elements
        for elem in elements {
            self.write_rule_element(elem)?;
        }
        
        Ok(())
    }

    fn element_size(&self, elem: &BinaryFormatElement) -> u16 {
        match elem {
            BinaryFormatElement::String(s) => {
                let utf16_len = s.encode_utf16().count() as u16;
                1 + 1 + utf16_len  // opcode + length + data
            }
            BinaryFormatElement::Variable(_) |
            BinaryFormatElement::Reference(_) |
            BinaryFormatElement::Predefined(_) |
            BinaryFormatElement::Modifier(_) => 2,  // opcode + parameter
            BinaryFormatElement::And |
            BinaryFormatElement::Any => 1,  // just opcode
            BinaryFormatElement::Switch(_) => 2,  // opcode + index
        }
    }

    fn write_rule_element(&mut self, elem: &BinaryFormatElement) -> std::result::Result<(), KmsError> {
        match elem {
            BinaryFormatElement::String(s) => {
                self.writer.write_u16::<LittleEndian>(OP_STRING)?;
                self.write_string(s)?;
            }
            BinaryFormatElement::Variable(idx) => {
                self.writer.write_u16::<LittleEndian>(OP_VARIABLE)?;
                self.writer.write_u16::<LittleEndian>(*idx as u16)?;
            }
            BinaryFormatElement::Reference(idx) => {
                self.writer.write_u16::<LittleEndian>(OP_REFERENCE)?;
                self.writer.write_u16::<LittleEndian>(*idx as u16)?;
            }
            BinaryFormatElement::Predefined(vk) => {
                self.writer.write_u16::<LittleEndian>(OP_PREDEFINED)?;
                self.writer.write_u16::<LittleEndian>(*vk)?;
            }
            BinaryFormatElement::Modifier(mod_flags) => {
                self.writer.write_u16::<LittleEndian>(OP_MODIFIER)?;
                self.writer.write_u16::<LittleEndian>(*mod_flags)?;
            }
            BinaryFormatElement::And => {
                self.writer.write_u16::<LittleEndian>(OP_AND)?;
            }
            BinaryFormatElement::Any => {
                self.writer.write_u16::<LittleEndian>(OP_ANY)?;
            }
            BinaryFormatElement::Switch(idx) => {
                self.writer.write_u16::<LittleEndian>(OP_SWITCH)?;
                self.writer.write_u16::<LittleEndian>((*idx + 1) as u16)?; // 1-based
            }
        }
        
        Ok(())
    }
}

