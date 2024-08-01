use std::fs::File;
use std::io::{BufRead, BufReader, Write};

use anyhow::Ok;
use sigalign::results::{AlignmentOperation, AlignmentOperations, LabeledQueryAlignment};

use crate::reference::ReferencePathDetector;
use crate::{error, error_msg, Result};

mod tsv;
pub use tsv::{extend_tsv_line_with_itoa_buffer, write_tsv_header};

mod sam;
pub use sam::{extend_sam_line_with_itoa_buffer, write_sam_header};
