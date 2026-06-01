pub mod conclusion;
pub mod message;
pub mod page;
pub mod peer;
pub mod raw_json;
pub mod session;

pub use conclusion::ConclusionRow;
pub use message::MessageRow;
pub use page::{DomainPage, PageInfo};
pub use peer::{PeerContextView, PeerDetails, PeerRow};
pub use raw_json::{JsonMap, RawJson};
pub use session::{SessionDetails, SessionRow, SessionSummariesView, SummaryKind, SummaryView};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceInfo {
    pub id: String,
    pub base_url: String,
}
