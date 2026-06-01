#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageInfo {
    pub page: u64,
    pub size: u64,
    pub total: u64,
    pub pages: u64,
    pub has_next: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainPage<T> {
    pub items: Vec<T>,
    pub info: PageInfo,
}

impl<T> DomainPage<T> {
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            info: PageInfo {
                page: 1,
                size: 50,
                total: 0,
                pages: 0,
                has_next: false,
            },
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
