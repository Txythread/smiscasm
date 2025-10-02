use crate::util::line_mapping::CodeInterpretationMode;

/// Where to store public libraries on download (relative path)
pub const PUBLIC_LIBS_DIR: &str = "pub-libs/";


/// Metadata about some libraries
pub const KNOWN_PUBLIC_LIBS: [(&str, &str, &str); 2] = [
    //  NAME                DOWNLOAD ADDRESS                                                                            FILE NAME
    ("bscmath",             "https://raw.githubusercontent.com/Txythread/smisc-bscmath/main/bscmath.s",                 "bscmath.s"),
    ("debuglib",            "https://raw.githubusercontent.com/Txythread/smisc-debuglib/main/debuglib.s",               "debuglib.s"),
];


/// The size of the memory page and the max size for immediate values.
pub const MEMORY_PAGE_SIZE: usize = 4096;


/// The mode (whether it expects data/code) the assembler is in when none have been specified
pub const DEFAULT_MODE: CodeInterpretationMode = CodeInterpretationMode::None;

