use crate::line::Line;

struct ScanState {
    x: f64,
    /// Minimum x at which the current ordering is invalid.
    valid_to: Option<f64>,
}

impl ScanState {}
