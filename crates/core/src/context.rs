use crate::prelude::*;

/// One step of a context path: a model field,
/// with the index of the repeatable-group instance it enters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Segment {
    pub field: &'static str,
    /// The 1-based index of the entered repeatable-group instance,
    /// or `None` for a single field or group.
    pub index: Option<NonZeroUsize>,
}

/// A pointer to one model node for a consumer to highlight.
///
/// The target is a leaf field, a repeatable-group instance, or the document root.
/// Each segment carries the concrete index of the repeatable group it crosses.
/// An empty path is the document root: either a document-wide rule,
/// or a node with no assigned model term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    /// The ordered segments from the document root down to the target;
    /// empty for the root.
    pub segments: Vec<Segment>,
}

/// Renders the path as dot-separated segments, such as `lines[2].item.name`.
/// The root renders as an empty string.
impl Display for Context {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        for (position, segment) in self.segments.iter().enumerate() {
            if position > 0 {
                formatter.write_str(".")?;
            }
            match segment.index {
                Some(index) => write!(formatter, "{}[{index}]", segment.field)?,
                None => formatter.write_str(segment.field)?,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn segment(field: &'static str, index: Option<usize>) -> Segment {
        Segment {
            field,
            index: index.and_then(NonZeroUsize::new),
        }
    }

    #[test]
    fn renders_the_segments_through_dots() {
        let context = Context {
            segments: vec![
                segment("lines", Some(2)),
                segment("item", None),
                segment("name", None),
            ],
        };

        assert_eq!(context.to_string(), "lines[2].item.name");
    }

    #[test]
    fn renders_the_root_as_an_empty_string() {
        let context = Context {
            segments: Vec::new(),
        };

        assert_eq!(context.to_string(), "");
    }
}
