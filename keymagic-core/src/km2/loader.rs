use crate::types::{FileHeader, FileHeader_1_3, FileHeader_1_4, Km2File, StringEntry, InfoEntry, Rule, BinaryFormatElement, LayoutOptions};
use crate::types::opcodes::*;
use super::error::{Km2Error, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read, Seek, SeekFrom};

pub struct Km2Loader;

impl Km2Loader {
    /// Load a KM2 file from binary data
    pub fn load(data: &[u8]) -> Result<Km2File> {
        let mut cursor = Cursor::new(data);
        
        // Read header
        let header = Self::read_header(&mut cursor)?;
        
        // Validate version (we support 1.3, 1.4, and 1.5)
        if header.major_version != 1 || header.minor_version < 3 || header.minor_version > 5 {
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
        if cursor.get_ref().len() < 12 {
            return Err(Km2Error::FileTooSmall(cursor.get_ref().len()));
        }
        
        let mut magic_code = [0u8; 4];
        cursor.read_exact(&mut magic_code)?;
        
        if &magic_code != b"KMKL" {
            return Err(Km2Error::InvalidMagicCode(magic_code));
        }
        
        let major_version = cursor.read_u8()?;
        let minor_version = cursor.read_u8()?;
        
        // Handle different versions
        let header = match (major_version, minor_version) {
            (1, 3) => Self::read_header_v1_3(cursor)?,
            (1, 4) => Self::read_header_v1_4(cursor)?,
            (1, 5) => Self::read_header_v1_5(cursor)?,
            _ => {
                // For unknown versions, try to read as v1.5
                // but allow older versions we haven't explicitly handled
                if major_version == 1 && minor_version < 3 {
                    // Very old version, not supported
                    return Err(Km2Error::UnsupportedVersion {
                        major: major_version,
                        minor: minor_version,
                    });
                }
                // Newer version, try to read as v1.5
                Self::read_header_v1_5(cursor)?
            }
        };
        
        Ok(header)
    }
    
    /// Read version 1.3 header
    fn read_header_v1_3(cursor: &mut Cursor<&[u8]>) -> Result<FileHeader> {
        // Reset to start and read the full v1.3 header
        cursor.seek(SeekFrom::Start(0))?;
        
        // Read FileHeader_1_3
        let mut header_bytes = vec![0u8; std::mem::size_of::<FileHeader_1_3>()];
        cursor.read_exact(&mut header_bytes)?;
        
        let header_1_3: FileHeader_1_3 = unsafe {
            std::ptr::read(header_bytes.as_ptr() as *const FileHeader_1_3)
        };
        
        // Convert to modern FileHeader
        Ok(FileHeader {
            magic_code: header_1_3.magic_code,
            major_version: header_1_3.major_version,
            minor_version: header_1_3.minor_version,
            string_count: header_1_3.string_count,
            info_count: 0, // v1.3 doesn't have info section
            rule_count: header_1_3.rule_count,
            layout_options: LayoutOptions {
                track_caps: header_1_3.layout_options.track_caps,
                auto_bksp: header_1_3.layout_options.auto_bksp,
                eat: header_1_3.layout_options.eat,
                pos_based: header_1_3.layout_options.pos_based,
                right_alt: 1, // Default to true for older versions
            },
        })
    }
    
    /// Read version 1.4 header
    fn read_header_v1_4(cursor: &mut Cursor<&[u8]>) -> Result<FileHeader> {
        // Reset to start and read the full v1.4 header
        cursor.seek(SeekFrom::Start(0))?;
        
        // Read FileHeader_1_4
        let mut header_bytes = vec![0u8; std::mem::size_of::<FileHeader_1_4>()];
        cursor.read_exact(&mut header_bytes)?;
        
        let header_1_4: FileHeader_1_4 = unsafe {
            std::ptr::read(header_bytes.as_ptr() as *const FileHeader_1_4)
        };
        
        // Convert to modern FileHeader
        Ok(FileHeader {
            magic_code: header_1_4.magic_code,
            major_version: header_1_4.major_version,
            minor_version: header_1_4.minor_version,
            string_count: header_1_4.string_count,
            info_count: header_1_4.info_count,
            rule_count: header_1_4.rule_count,
            layout_options: LayoutOptions {
                track_caps: header_1_4.layout_options.track_caps,
                auto_bksp: header_1_4.layout_options.auto_bksp,
                eat: header_1_4.layout_options.eat,
                pos_based: header_1_4.layout_options.pos_based,
                right_alt: 1, // Default to true for older versions
            },
        })
    }
    
    /// Read version 1.5 header (current version)
    fn read_header_v1_5(cursor: &mut Cursor<&[u8]>) -> Result<FileHeader> {
        // Reset to start if we've already read version bytes
        cursor.seek(SeekFrom::Start(0))?;
        
        let mut magic_code = [0u8; 4];
        cursor.read_exact(&mut magic_code)?;
        
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
            right_alt: cursor.read_u8()?,
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
            
            // Validate LHS: Predefined elements must be preceded by AND
            Self::validate_lhs_predefined(&lhs)
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
    fn read_rule_elements(cursor: &mut Cursor<&[u8]>, byte_len: usize) -> Result<Vec<BinaryFormatElement>> {
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
                    BinaryFormatElement::String(value)
                }
                OP_VARIABLE => {
                    let index = cursor.read_u16::<LittleEndian>()? as usize;
                    BinaryFormatElement::Variable(index)
                }
                OP_REFERENCE => {
                    let index = cursor.read_u16::<LittleEndian>()? as usize;
                    BinaryFormatElement::Reference(index)
                }
                OP_PREDEFINED => {
                    let vk_code = cursor.read_u16::<LittleEndian>()?;
                    BinaryFormatElement::Predefined(vk_code)
                }
                OP_MODIFIER => {
                    let flags = cursor.read_u16::<LittleEndian>()?;
                    BinaryFormatElement::Modifier(flags)
                }
                OP_AND => {
                    BinaryFormatElement::And
                }
                OP_ANY => {
                    BinaryFormatElement::Any
                }
                OP_SWITCH => {
                    let state_index = cursor.read_u16::<LittleEndian>()? as usize;
                    BinaryFormatElement::Switch(state_index)
                }
                _ => return Err(Km2Error::InvalidOpcode(opcode))
            };
            
            elements.push(element);
        }
        
        Ok(elements)
    }
    
    /// Validate that standalone Predefined elements are not allowed in LHS
    /// Valid pattern: AND VK1 VK2 ... VKn
    fn validate_lhs_predefined(lhs: &[BinaryFormatElement]) -> Result<()> {
        let mut i = 0;
        let mut in_vk_sequence = false;
        
        while i < lhs.len() {
            match &lhs[i] {
                BinaryFormatElement::And => {
                    // AND marks the start of a VK sequence
                    in_vk_sequence = true;
                }
                BinaryFormatElement::Predefined(_) => {
                    // Predefined must be part of a VK sequence
                    if !in_vk_sequence {
                        return Err(Km2Error::InvalidPredefinedUsage);
                    }
                }
                _ => {
                    // Any other element ends the VK sequence
                    in_vk_sequence = false;
                }
            }
            i += 1;
        }
        Ok(())
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