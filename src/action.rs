use crate::api::wrapper::ReplyRef;
use crate::models::notifications::NotificationViewModel;
use crate::models::post::PostViewModel;
use crate::models::preferences::PreferencesViewModel;
use crate::models::profile::ProfileViewModel;
use crate::models::thread::ThreadViewModel;
use ratatui_image::protocol::StatefulProtocol;


pub enum Action {
    Quit,
    Tick,
    Render,

    SelectNext,
    SelectPrev,
    ScrollToTop,
    ScrollToBottom,
    OpenThread,
    GoBack,
    SwitchTab(usize),
    ViewAuthorProfile,

    ShowLogin,
    SubmitLogin {
        handle: String,
        password: String,
    },
    LoginSuccess(String),
    LoginFailed(String),
    Logout,
    LogoutConfirm,
    DefinitelyLogout,
    LogoutCancelled,

    RefreshTimeline,
    LoadMoreTimeline,
    TimelineLoaded {
        posts: Vec<PostViewModel>,
        cursor: Option<String>,
        append: bool,
    },

    ThreadLoaded(Box<Option<ThreadViewModel>>),

    OpenComposer {
        reply_to: Option<ReplyRef>,
        reply_to_author: Option<String>,
    },
    CloseComposer,
    SubmitPost {
        text: String,
        reply_to: Option<ReplyRef>,
    },
    PostCreated(String),

    ToggleLike,
    LikeSuccess {
        post_uri: String,
        like_uri: String,
    },
    UnlikeSuccess {
        post_uri: String,
    },
    ToggleRepost,
    RepostSuccess {
        post_uri: String,
        repost_uri: String,
    },
    UnrepostSuccess {
        post_uri: String,
    },

    LoadProfile(String),
    ProfileLoaded {
        profile: ProfileViewModel,
        posts: Vec<PostViewModel>,
        cursor: Option<String>,
    },

    FocusSearchInput,
    Search(String),
    LoadMoreResults,
    SearchResults {
        posts: Vec<PostViewModel>,
        cursor: Option<String>,
        append: bool,
    },

    RefreshNotifications,
    LoadMoreNotifications,
    NotificationsLoaded {
        notifications: Vec<NotificationViewModel>,
        cursor: Option<String>,
        append: bool,
    },

    Error(String),
    ClearError,

    ImageLoaded {
    post_uri: String,
    protocol: StatefulProtocol,
    cols: u16,
    rows: u16,
},

    OpenPreferences,
    SavePreferences(PreferencesViewModel),
    TogglePreferences,
    ReplyTimeout,
}


