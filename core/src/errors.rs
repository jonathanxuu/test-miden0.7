use core::fmt;

// INPUT ERROR
// ================================================================================================

#[derive(Clone, Debug)]
pub enum InputError {
    NotFieldElement(u64, &'static str),
    DuplicateAdviceRoot([u8; 32]),
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use InputError::*;
        match self {
            NotFieldElement(num, description) => {
                write!(f, "{num} is not a valid field element: {description}")
            }
            DuplicateAdviceRoot(key) => {
                write!(f, "{key:02x?} is a duplicate of the current merkle set")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InputError {}

// OUTPUT ERROR
// ================================================================================================
#[derive(Clone, Debug)]
pub enum OutputError {
    InvalidOverflowAddress(u64),
    InvalidOverflowAddressLength(usize, usize),
    InvalidStackElement(u64),
}

impl fmt::Display for OutputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use OutputError::*;
        match self {
            InvalidOverflowAddress(address) => {
                write!(f, "overflow addresses contains {address} that is not a valid field element")
            }
            InvalidOverflowAddressLength(actual, expected) => {
                write!(f, "overflow addresses length is {actual}, but expected {expected}")
            }
            InvalidStackElement(element) => {
                write!(f, "stack contains {element} that is not a valid field element")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for OutputError {}
