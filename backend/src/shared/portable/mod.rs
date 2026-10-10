/// Versioned portable-data contract shared by the `imports` and `exports` features.
///
/// Bumping `PORTABLE_FORMAT_VERSION` is the only supported way to change the shape;
/// older archives stay importable because every reader tolerates unknown fields.
pub mod schema;
