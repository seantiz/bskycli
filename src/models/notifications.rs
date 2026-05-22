use atrium_api::app::bsky::notification::list_notifications::Notification;

#[derive(Debug, Clone)]
pub struct NotificationViewModel {
    pub uri: String,
    pub cid: String,
    pub did: String,
    pub handle: String,
    pub display_name: String,
    pub author_avatar: Option<String>,
    pub reason: String,
    pub subject: Option<String>,
    pub record: Option<String>,
    pub is_read: bool,
    pub indexed_at: String,
}

impl NotificationViewModel {
    pub fn from_notification(
        notif: &Notification,
    ) -> Option<Self> {
        let author = &notif.author;
        let record = match serde_json::to_value(&notif.record) {
            Ok(v) => v,
            Err(_) => return None,
        };
        let record_text = record
            .get("text")
            .and_then(|t| t.as_str())
            .map(|s| s.to_string());

        Some(NotificationViewModel {
            uri: notif.uri.clone(),
            cid: notif.cid.as_ref().to_string(),
            did: author.did.to_string(),
            handle: author.handle.to_string(),
            display_name: author
                .display_name
                .clone()
                .unwrap_or_else(|| author.handle.to_string()),
            author_avatar: author.avatar.clone(),
            reason: notif.reason.clone(),
            subject: notif.reason_subject.clone(),
            record: record_text,
            is_read: notif.is_read,
            indexed_at: notif.indexed_at.as_str().to_string(),
        })
    }
}
