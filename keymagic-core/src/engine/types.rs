//! Type aliases and conversions for the engine

use crate::types::{BinaryFormatElement, VirtualKey};

/// Element type used by the engine (simplified from BinaryFormatElement)
#[derive(Debug, Clone)]
pub enum Element {
    String(String),
    Variable(usize),            // 0-based index (converted from 1-based)
    Reference(usize),           // Back-reference ($1, $2, etc.)
    Predefined(Predefined),     // Virtual key code
    Modifier(u16),              // Modifier flags or index
    And,                        // Logical AND
    Any,                        // ANY keyword
    Switch(usize),              // State index (0-based)
}

/// Virtual key code type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Predefined(pub u16);

impl Predefined {
    /// No key
    pub const NONE: Self = Self(0);
    
    /// Creates from raw value
    pub fn from_raw(value: u16) -> Self {
        Self(value)
    }
    
    /// Gets the raw value
    pub fn raw(&self) -> u16 {
        self.0
    }
}

impl From<VirtualKey> for Predefined {
    fn from(vk: VirtualKey) -> Self {
        Self(vk as u16)
    }
}

impl From<BinaryFormatElement> for Element {
    fn from(elem: BinaryFormatElement) -> Self {
        match elem {
            BinaryFormatElement::String(s) => Element::String(s),
            BinaryFormatElement::Variable(idx) => {
                // Convert from 1-based to 0-based
                Element::Variable(idx.saturating_sub(1))
            }
            BinaryFormatElement::Reference(idx) => Element::Reference(idx),
            BinaryFormatElement::Predefined(vk) => Element::Predefined(Predefined(vk)),
            BinaryFormatElement::Modifier(m) => Element::Modifier(m),
            BinaryFormatElement::And => Element::And,
            BinaryFormatElement::Any => Element::Any,
            BinaryFormatElement::Switch(idx) => {
                // Use state index as-is
                Element::Switch(idx)
            }
        }
    }
}