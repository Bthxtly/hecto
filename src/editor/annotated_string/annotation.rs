use super::annotation_type::AnnotationType;

type ByteIdx = usize;

#[derive(Debug)]
pub struct Annotation {
    pub typ: AnnotationType, // we can't name it as `type`
    pub start_byte_idx: ByteIdx,
    pub end_byte_idx: ByteIdx,
}
