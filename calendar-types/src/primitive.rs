//! Primitive types not belonging to a specific area.

/// A numeric sign, which may be either positive or negative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(i8)]
pub enum Sign {
    /// Negative sign (`-`).
    Neg = -1,
    /// Positive sign (`+`).
    #[default]
    Pos = 1,
}

impl Sign {
    /// Returns the ASCII character representation of this sign (`'+'` or `'-'`).
    pub const fn as_char(self) -> char {
        match self {
            Sign::Neg => '-',
            Sign::Pos => '+',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_ord_impl() {
        assert!(Sign::Neg < Sign::Pos);
    }
}
