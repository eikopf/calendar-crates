//! Primitive types not belonging to a specific area.

/// A numeric sign, which may be either positive or negative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Sign {
    Neg,
    #[default]
    Pos,
}

impl Sign {
    pub const fn as_char(self) -> char {
        match self {
            Sign::Neg => '-',
            Sign::Pos => '+',
        }
    }
}
