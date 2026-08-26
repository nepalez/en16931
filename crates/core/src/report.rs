use crate::prelude::*;
use crate::{Context, Severity};

/// A single problem a validator reported, bound to the node of the model it speaks of.
///
/// The address the validator wrote and its normalized form both served the binding,
/// and neither of them outlives it.
/// What stays is the weight of the problem, the rule that fired,
/// the message for a human reader, and the node a consumer highlights.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// The weight of the problem.
    pub severity: Severity,
    /// The identifier of the rule that fired, when the report names one.
    pub code: Option<String>,
    /// The message the validator wrote for a human reader.
    pub text: String,
    /// The node of the model the problem points at.
    pub context: Context,
}

/// Renders as `severity at path: message`, such as
/// `error at lines[2].item.name: the name is too long`.
impl Display for Problem {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.severity, self.context, self.text
        )
    }
}

/// The outcome of one validation pass: every problem bound to a node of the model.
///
/// A `ValidDocument` carries a report of no error, which may still carry
/// warnings and remarks. An `InvalidDocument` carries a report of at least one error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// The problems in the order the validator listed them.
    pub problems: Vec<Problem>,
}

impl Report {
    /// Tells whether the report holds a problem that invalidates the document.
    pub(crate) fn has_errors(&self) -> bool {
        self.problems
            .iter()
            .any(|problem| problem.severity == Severity::Error)
    }
}
