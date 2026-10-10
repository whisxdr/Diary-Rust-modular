use serde::{Deserialize, Serialize};

use crate::shared::errors::{AppError, ValidationError};

pub const DEFAULT_LIMIT: i64 = 20;
pub const MAX_LIMIT: i64 = 100;

/// Query parameters shared by every list endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl PaginationQuery {
    pub fn normalized(&self) -> Result<(i64, i64), AppError> {
        let limit = self.limit.unwrap_or(DEFAULT_LIMIT);
        let offset = self.offset.unwrap_or(0);

        if !(1..=MAX_LIMIT).contains(&limit) {
            return Err(
                ValidationError::new(format!("limit must be between 1 and {MAX_LIMIT}")).into(),
            );
        }
        if offset < 0 {
            return Err(ValidationError::new("offset must not be negative").into());
        }
        Ok((limit, offset))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, total: i64, limit: i64, offset: i64) -> Self {
        Self {
            items,
            total,
            limit,
            offset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_bounds_are_enforced() {
        let default = PaginationQuery {
            limit: None,
            offset: None,
        }
        .normalized()
        .unwrap();
        assert_eq!(default, (DEFAULT_LIMIT, 0));

        let too_large = PaginationQuery {
            limit: Some(500),
            offset: None,
        }
        .normalized();
        assert!(too_large.is_err());

        let negative = PaginationQuery {
            limit: None,
            offset: Some(-1),
        }
        .normalized();
        assert!(negative.is_err());
    }
}
