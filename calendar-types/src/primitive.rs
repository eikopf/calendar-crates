//! Primitive types not belonging to a specific area.

/// A numeric sign, which may be either positive or negative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(i8)]
pub enum Sign {
    Neg = -1,
    #[default]
    Pos = 1,
}

impl Sign {
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
