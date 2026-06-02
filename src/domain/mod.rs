pub mod chat;
pub mod conclusion;
pub mod message;
pub mod page;
pub mod peer;
pub mod queue;
pub mod raw_json;
pub mod search;
pub mod session;
pub mod upload;

pub use chat::{ChatMessage, ChatRole};
pub use conclusion::{ConclusionInput, ConclusionRow};
pub use message::MessageRow;
pub use page::{DomainPage, PageInfo};
pub use peer::{PeerContextView, PeerDetails, PeerRow, ReprOpts};
pub use queue::QueueStatus;
pub use raw_json::{JsonMap, RawJson};
pub use search::SearchScope;
pub use session::{
    SessionContextView, SessionDetails, SessionPeerRow, SessionRow, SessionSummariesView,
    SummaryKind, SummaryView,
};
pub use upload::FileSource;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceInfo {
    pub id: String,
    pub base_url: String,
}
