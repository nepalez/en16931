use crate::prelude::*;
use crate::{Context, Dictionary, Namespace, Path, Segment, Step};

/// The walk state that turns a sequence of node visits into a dictionary over the set `N`.
pub(crate) struct Trace<N: Namespace> {
    // One frame per open element, counting its same-named children to index them.
    frames: Vec<HashMap<(N, String), usize>>,
    // The record-form steps from the root down to the current node.
    path: Vec<Step<N>>,
    // The model-side segments from the root down to the current node.
    context: Vec<Segment>,
    // The accumulated path-to-context dictionary.
    dictionary: Dictionary<N>,
}

impl<N: Namespace> Trace<N> {
    pub(crate) fn new() -> Self {
        Self {
            // The outermost frame indexes the root element among its (single) self.
            frames: vec![HashMap::new()],
            path: Vec::new(),
            context: Vec::new(),
            dictionary: HashMap::new(),
        }
    }

    /// Descends into a child element, extending the record-form path.
    pub(crate) fn enter(&mut self, namespace: N, name: &str) {
        let index = self.next_index(namespace, name);
        self.path.push(Step {
            namespace,
            name: name.to_owned(),
            index,
        });
        self.frames.push(HashMap::new());
    }

    /// Ascends out of the current element.
    pub(crate) fn leave(&mut self) {
        self.path.pop();
        self.frames.pop();
    }

    /// Extends the model context with a single field or group.
    pub(crate) fn push_field(&mut self, field: &'static str) {
        self.context.push(Segment { field, index: None });
    }

    /// Extends the model context with one instance of a repeatable group.
    pub(crate) fn push_instance(&mut self, field: &'static str, index: NonZeroUsize) {
        self.context.push(Segment {
            field,
            index: Some(index),
        });
    }

    /// Drops the innermost model segment.
    pub(crate) fn pop_context(&mut self) {
        self.context.pop();
    }

    /// Records the current node against the model context built so far.
    pub(crate) fn record_context(&mut self) {
        let context = Context {
            segments: self.context.clone(),
        };
        self.record(context);
    }

    /// Records the current node against the root context (a term-less node).
    pub(crate) fn record_root(&mut self) {
        self.record(Context {
            segments: Vec::new(),
        });
    }

    /// Records the current node against a field of the invoice itself,
    /// whatever group encloses the node.
    pub(crate) fn record_header(&mut self, field: &'static str) {
        self.record(Context {
            segments: vec![Segment { field, index: None }],
        });
    }

    /// Consumes the trace, yielding the dictionary it has built.
    pub(crate) fn into_dictionary(self) -> Dictionary<N> {
        self.dictionary
    }

    fn record(&mut self, context: Context) {
        self.dictionary.insert(
            Path {
                steps: self.path.clone(),
            },
            context,
        );
    }

    fn next_index(&mut self, namespace: N, name: &str) -> NonZeroUsize {
        let frame = self.frames.last_mut().expect("an open frame");
        let count = frame.entry((namespace, name.to_owned())).or_insert(0);
        *count += 1;
        NonZeroUsize::new(*count).expect("a positive sibling count")
    }
}
