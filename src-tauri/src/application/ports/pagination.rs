/// Pagination parameters for list queries. Page numbers are 1-based.
#[derive(Debug, Clone)]
pub struct PaginationParams {
    pub page: u32,
    pub per_page: u32,
}

impl PaginationParams {
    pub fn offset(&self) -> u64 {
        ((self.page.saturating_sub(1)) as u64) * (self.per_page as u64)
    }
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 50,
        }
    }
}

/// Central pagination standard. Every paginated list endpoint returns this.
#[derive(Debug, Clone)]
pub struct Page<T> {
    pub first: u32,
    pub last: u32,
    pub previous: Option<u32>,
    pub next: Option<u32>,
    pub total: u64,
    pub data: Vec<T>,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, total: u64, params: &PaginationParams) -> Self {
        let last = if total == 0 {
            1
        } else {
            ((total as u32).saturating_sub(1) / params.per_page) + 1
        };
        let page = params.page.clamp(1, last);
        Self {
            first: 1,
            last,
            previous: if page > 1 { Some(page - 1) } else { None },
            next: if page < last { Some(page + 1) } else { None },
            total,
            data: items,
        }
    }

    pub fn map<U>(self, f: impl FnMut(T) -> U) -> Page<U> {
        Page {
            first: self.first,
            last: self.last,
            previous: self.previous,
            next: self.next,
            total: self.total,
            data: self.data.into_iter().map(f).collect(),
        }
    }
}
