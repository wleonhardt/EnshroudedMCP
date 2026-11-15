//! Game-free workload admission for client recipe archive observations.

use crate::item_lookup::{MAX_BYTES, MAX_NAME, span};

pub const MAX_RECIPES: usize = 16_384;
pub const MAX_INPUTS: usize = 32;
pub const MAX_OUTPUTS: usize = 8;
pub const MAX_CATEGORIES: usize = 4096;
pub const MAX_CATEGORY_MEMBERS: usize = 1024;
pub const RETURNED_CATEGORY_MEMBERS: usize = 32;

#[derive(Clone, Copy, Debug)]
pub struct CategoryLayout {
    pub categories: usize,
    pub category_size: usize,
    pub id: usize,
    pub items: usize,
    pub member_size: usize,
}

impl CategoryLayout {
    pub fn check(self, bytes: usize) -> Result<(), String> {
        request(bytes, 1)?;
        stride(self.category_size)?;
        stride(self.member_size)?;
        field(self.categories, 8, bytes)?;
        field(self.id, 4, self.category_size)?;
        field(self.items, 8, self.category_size)
    }
}

pub fn request(bytes: usize, limit: usize) -> Result<(), String> {
    if bytes > MAX_BYTES || !(1..=8).contains(&limit) {
        return Err("recipe bytes exceed 16 MiB or limit is outside 1..8".into());
    }
    Ok(())
}

pub fn stride(size: usize) -> Result<(), String> {
    if !(1..=4096).contains(&size) {
        return Err("recipe record stride must be in 1..4096".into());
    }
    Ok(())
}

pub fn count(count: usize, maximum: usize) -> Result<(), String> {
    if count > maximum {
        return Err("recipe array exceeds its traversal cap".into());
    }
    Ok(())
}

pub fn name(bytes: usize) -> Result<(), String> {
    count(bytes, MAX_NAME)
}

/// Fields must fit wholly within their declared records, including headers.
pub fn field(offset: usize, bytes: usize, record: usize) -> Result<(), String> {
    span(offset, 1, bytes, record).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_counts_strides_and_field_spans_are_bounded() {
        assert!(request(MAX_BYTES, 8).is_ok());
        assert!(request(MAX_BYTES + 1, 1).is_err());
        assert!(request(0, 0).is_err());
        assert!(request(0, 9).is_err());
        for size in [0, 4097, usize::MAX] {
            assert!(stride(size).is_err());
        }
        assert!(stride(4096).is_ok());
        assert!(count(MAX_RECIPES + 1, MAX_RECIPES).is_err());
        assert!(count(MAX_INPUTS + 1, MAX_INPUTS).is_err());
        assert!(count(MAX_OUTPUTS + 1, MAX_OUTPUTS).is_err());
        assert!(name(128).is_ok());
        assert!(name(129).is_err());
        assert!(field(8, 8, 16).is_ok());
        assert!(field(9, 8, 16).is_err());
        assert!(field(usize::MAX, 8, 16).is_err());
    }

    #[test]
    fn category_layout_and_workloads_refuse_invalid_spans_before_traversal() {
        let layout = CategoryLayout {
            categories: 0,
            category_size: 12,
            id: 0,
            items: 4,
            member_size: 8,
        };
        assert!(layout.check(8).is_ok());
        for bad in [
            CategoryLayout {
                categories: usize::MAX,
                ..layout
            },
            CategoryLayout { id: 9, ..layout },
            CategoryLayout { items: 5, ..layout },
            CategoryLayout {
                member_size: 0,
                ..layout
            },
            CategoryLayout {
                category_size: 4097,
                ..layout
            },
        ] {
            assert!(bad.check(32).is_err());
        }
        assert!(layout.check(MAX_BYTES + 1).is_err());
        assert!(layout.check(7).is_err());
        assert!(count(MAX_CATEGORIES, MAX_CATEGORIES).is_ok());
        assert!(count(MAX_CATEGORIES + 1, MAX_CATEGORIES).is_err());
        assert!(count(MAX_CATEGORY_MEMBERS + 1, MAX_CATEGORY_MEMBERS).is_err());
    }
}
