use super::AnnotationType;

#[derive(Debug)]

pub struct AnnotatedStringPart<'a> {
    pub string: &'a str,
    pub typ: Option<AnnotationType>,
}
