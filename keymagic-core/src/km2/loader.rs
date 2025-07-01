use crate::types::{FileHeader, Km2File, StringEntry, InfoEntry, Rule, RuleElement, LayoutOptions};
use crate::types::opcodes::*;
use super::error::{Km2Error, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

pub struct Km2Loader;

impl Km2Loader {
    /// Load a KM2 file from binary data
    pub fn load(data: &[u8]) -> Result<Km2File> {
        let mut cursor = Cursor::new(data);
        
        // Read header
        let header = Self::read_header(&mut cursor)?;
        
        // Validate version
        if header.major_version != 1 || header.minor_version > 5 {
            return Err(Km2Error::UnsupportedVersion {
                major: header.major_version,
                minor: header.minor_version,
            });
        }
        
        // Read strings
        let strings = Self::read_strings(&mut cursor, header.string_count as usize)?;
        
        // Read info entries
        let info = Self::read_info(&mut cursor, header.info_count as usize)?;
        
        // Read rules
        let rules = Self::read_rules(&mut cursor, header.rule_count as usize)?;
        
        Ok(Km2File {
            header,
            strings,
            info,
            rules,
        })
    }
    
    /// Read file header
    fn read_header(cursor: &mut Cursor<&[u8]>) -> Result<FileHeader> {
        if cursor.get_ref().len() < 16 {
            return Err(Km2Error::FileTooSmall(cursor.get_ref().len()));
        }
        
        let mut magic_code = [0u8; 4];
        cursor.read_exact(&mut magic_code)?;
        
        if &magic_code != b"KMKL" {
            return Err(Km2Error::InvalidMagicCode(magic_code));
        }
        
        let major_version = cursor.read_u8()?;
        let minor_version = cursor.read_u8()?;
        let string_count = cursor.read_u16::<LittleEndian>()?;
        let info_count = cursor.read_u16::<LittleEndian>()?;
        let rule_count = cursor.read_u16::<LittleEndian>()?;
        
        let layout_options = LayoutOptions {
            track_caps: cursor.read_u8()?,
            auto_bksp: cursor.read_u8()?,
            eat: cursor.read_u8()?,
            pos_based: cursor.read_u8()?,
            right_alt: if minor_version >= 5 { cursor.read_u8()? } else { 0 },
        };
        
        // Skip C++ struct padding byte
        cursor.read_u8()?;
        
        Ok(FileHeader {
            magic_code,
            major_version,
            minor_version,
            string_count,
            info_count,
            rule_count,
            layout_options,
        })
    }
    
    /// Read string table
    fn read_strings(cursor: &mut Cursor<&[u8]>, count: usize) -> Result<Vec<StringEntry>> {
        let mut strings = Vec::with_capacity(count);
        
        for _ in 0..count {
            let length = cursor.read_u16::<LittleEndian>()? as usize;
            let mut utf16_data = vec![0u16; length];
            
            for i in 0..length {
                utf16_data[i] = cursor.read_u16::<LittleEndian>()?;
            }
            
            let value = String::from_utf16(&utf16_data)
                .map_err(|_| Km2Error::InvalidUtf16(cursor.position() as usize))?;
            
            strings.push(StringEntry { value });
        }
        
        Ok(strings)
    }
    
    /// Read info section
    fn read_info(cursor: &mut Cursor<&[u8]>, count: usize) -> Result<Vec<InfoEntry>> {
        let mut info = Vec::with_capacity(count);
        
        for _ in 0..count {
            let mut id = [0u8; 4];
            cursor.read_exact(&mut id)?;
            
            let length = cursor.read_u16::<LittleEndian>()? as usize;
            let mut data = vec![0u8; length];
            cursor.read_exact(&mut data)?;
            
            info.push(InfoEntry { id, data });
        }
        
        Ok(info)
    }
    
    /// Read rules section
    fn read_rules(cursor: &mut Cursor<&[u8]>, count: usize) -> Result<Vec<Rule>> {
        let mut rules = Vec::with_capacity(count);
        
        for i in 0..count {
            // Read LHS (size is in 16-bit units, convert to bytes)
            let lhs_len = cursor.read_u16::<LittleEndian>()? as usize;
            let lhs = Self::read_rule_elements(cursor, lhs_len * 2)
                .map_err(|_| Km2Error::InvalidRule(i))?;
            
            // Read RHS (size is in 16-bit units, convert to bytes)
            let rhs_len = cursor.read_u16::<LittleEndian>()? as usize;
            let rhs = Self::read_rule_elements(cursor, rhs_len * 2)
                .map_err(|_| Km2Error::InvalidRule(i))?;
            
            rules.push(Rule { lhs, rhs });
        }
        
        Ok(rules)
    }
    
    /// Read rule elements
    fn read_rule_elements(cursor: &mut Cursor<&[u8]>, byte_len: usize) -> Result<Vec<RuleElement>> {
        let start_pos = cursor.position() as usize;
        let mut elements = Vec::new();
        
        while (cursor.position() as usize - start_pos) < byte_len {
            let opcode = cursor.read_u16::<LittleEndian>()?;
            
            let element = match opcode {
                OP_STRING => {
                    let length = cursor.read_u16::<LittleEndian>()? as usize;
                    let mut utf16_data = vec![0u16; length];
                    for i in 0..length {
                        utf16_data[i] = cursor.read_u16::<LittleEndian>()?;
                    }
                    let value = String::from_utf16(&utf16_data)
                        .map_err(|_| Km2Error::InvalidUtf16(cursor.position() as usize))?;
                    RuleElement::String(value)
                }
                OP_VARIABLE => {
                    let index = cursor.read_u16::<LittleEndian>()? as usize;
                    RuleElement::Variable(index)
                }
                OP_REFERENCE => {
                    let index = cursor.read_u16::<LittleEndian>()? as usize;
                    RuleElement::Reference(index)
                }
                OP_PREDEFINED => {
                    let vk_code = cursor.read_u16::<LittleEndian>()?;
                    RuleElement::Predefined(vk_code)
                }
                OP_MODIFIER => {
                    let flags = cursor.read_u16::<LittleEndian>()?;
                    RuleElement::Modifier(flags)
                }
                OP_ANYOF => {
                    let var_index = cursor.read_u16::<LittleEndian>()? as usize;
                    RuleElement::AnyOf(var_index)
                }
                OP_AND => {
                    RuleElement::And
                }
                OP_NANYOF => {
                    let var_index = cursor.read_u16::<LittleEndian>()? as usize;
                    RuleElement::NotAnyOf(var_index)
                }
                OP_ANY => {
                    RuleElement::Any
                }
                OP_SWITCH => {
                    let string_index = cursor.read_u16::<LittleEndian>()? as usize;
                    RuleElement::Switch(string_index)
                }
                _ => return Err(Km2Error::InvalidOpcode(opcode)),
            };
            
            elements.push(element);
        }
        
        Ok(elements)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_load_empty_km2() {
        // Minimal valid KM2 file with no strings, info, or rules
        let mut data = vec![];
        data.extend_from_slice(b"KMKL"); // magic
        data.push(1); // major version
        data.push(5); // minor version
        data.extend_from_slice(&0u16.to_le_bytes()); // string count
        data.extend_from_slice(&0u16.to_le_bytes()); // info count
        data.extend_from_slice(&0u16.to_le_bytes()); // rule count
        data.extend_from_slice(&[0, 0, 0, 0, 0]); // layout options
        data.push(0); // padding byte
        
        let result = Km2Loader::load(&data);
        assert!(result.is_ok());
        
        let km2 = result.unwrap();
        assert_eq!(km2.header.major_version, 1);
        assert_eq!(km2.header.minor_version, 5);
        assert_eq!(km2.strings.len(), 0);
        assert_eq!(km2.info.len(), 0);
        assert_eq!(km2.rules.len(), 0);
    }
    
    #[test]
    fn test_invalid_magic() {
        let mut data = vec![];
        data.extend_from_slice(b"XXXX"); // invalid magic
        data.extend_from_slice(&[1, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // rest of header
        
        let result = Km2Loader::load(&data);
        assert!(matches!(result, Err(Km2Error::InvalidMagicCode(_))));
    }
}