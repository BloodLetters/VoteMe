use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoteProtocolVersion {
    V1,
    V2,
}

impl fmt::Display for VoteProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::V1 => write!(f, "V1"),
            Self::V2 => write!(f, "V2"),
        }
    }
}
