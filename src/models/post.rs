use crate::models::preferences::PreferencesViewModel;
use atrium_api::app::bsky::embed::record::{ViewRecordEmbedsItem, ViewRecordRefs};
use atrium_api::app::bsky::feed::defs::{
    FeedViewPost, FeedViewPostReasonRefs, PostView, PostViewEmbedRefs, ReplyRefParentRefs,
};
use atrium_api::types::Union;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct PostViewModel {
    pub uri: String,
    pub cid: String,
    pub did: String,
    pub handle: String,
    pub display_name: String,
    pub avatar: Option<String>,
    pub text: String,
    pub facets: Vec<Facet>,
    pub created_at: DateTime<Utc>,
    pub likes: i64,
    pub reposts: i64,
    pub replies: i64,
    pub quote_count: i64,
    pub is_liked: bool,
    pub like_uri: Option<String>,
    pub is_reposted: bool,
    pub repost_uri: Option<String>,
    pub meta: Option<EmbedMeta>,
    pub replied_by: Option<String>,
    pub reposted_by: Option<String>,
    pub images: Vec<ImageMeta>,
}

#[derive(Debug, Clone)]
pub struct Facet {
    pub start: usize,
    pub end: usize,
    pub kind: FacetKind,
}

// TODO: Replace this ref, use the Main object from ATP
#[derive(Debug, Clone)]
pub enum FacetKind {
    Mention(String),
    Link(String),
    Tag(String),
}

#[derive(Debug, Clone)]
pub struct ImageMeta {
    pub size: String,
    pub alt: Option<String>,
}

#[derive(Debug, Clone)]
pub struct QuotedPost {
    pub uri: String,
    pub cid: String,
    pub handle: String,
    pub display_name: String,
    pub text: String,
    pub facets: Vec<Facet>,
    pub meta: Option<EmbedMeta>,
}

#[derive(Debug, Clone)]
pub struct EmbedMeta {
    pub kind: EmbedKind,
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
}

// TODO: Replace this ref, use the Main object from ATP
#[derive(Debug, Clone)]
pub enum EmbedKind {
    ExternalLink,
    Images(Vec<ImageMeta>),
    Video,
    Record(Box<QuotedPost>),
    RecordWithMedia(Box<QuotedPost>),
}

/// Build a [`PostViewModel`] from a [`FeedViewPost`](atrium_api::app::bsky::feed::defs::FeedViewPost) endpoint, for getting post metadata within the context of a rendered feed view e.g. is the post a reply, repost, etc.
///
/// Build a [`PostViewModel`] from a bare [`PostView`](atrium_api::app::bsky::feed::defs::PostView),
/// where post data has no surrounding context, just the post itself, useful for collecting posts
/// from search queries
impl PostViewModel {
    pub fn from_feed_view_post(fvp: &FeedViewPost) -> Option<Self> {
        let replied = fvp.reply.as_ref().and_then(|r| replying_handle(&r.parent));
        let reposted_by = fvp.reason.as_ref().and_then(get_reposted_reason);
        Self::from_post_view_inner(&fvp.post, replied, reposted_by)
    }

    pub fn from_post_view(post: &PostView) -> Option<Self> {
        Self::from_post_view_inner(post, None, None)
    }

    fn from_post_view_inner(
        post: &PostView,
        replied_by: Option<String>,
        reposted_by: Option<String>,
    ) -> Option<Self> {
        let author = &post.author;

        let record = match serde_json::to_value(&post.record) {
            Ok(v) => v,
            Err(_) => return None,
        };

        let text = record.get("text")?.as_str().unwrap().to_string();
        let created_at = record.get("createdAt")?.as_str().unwrap();
        let created_at_time = DateTime::parse_from_rfc3339(created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap();

        let facets = parse_facets_from_record(&record);

        let viewer = post.viewer.as_ref();
        let is_liked = viewer.and_then(|v| v.like.as_ref()).is_some();
        let like_uri = viewer.and_then(|v| v.like.clone());
        let is_reposted = viewer.and_then(|v| v.repost.as_ref()).is_some();
        let repost_uri = viewer.and_then(|v| v.repost.clone());

        let embed_meta = post.embed.as_ref().and_then(get_embed_meta);

        Some(PostViewModel {
            uri: post.uri.clone(),
            cid: post.cid.as_ref().to_string(),
            did: author.did.to_string(),
            handle: author.handle.to_string(),
            display_name: author.display_name.clone().unwrap_or_else(|| author.handle.to_string()),
            avatar: author.avatar.clone(),
            text,
            facets,
            created_at: created_at_time,
            likes: post.like_count.unwrap_or(0),
            reposts: post.repost_count.unwrap_or(0),
            replies: post.reply_count.unwrap_or(0),
            quote_count: post.quote_count.unwrap_or(0),
            is_liked,
            like_uri,
            is_reposted,
            repost_uri,
            meta: embed_meta,
            replied_by,
            reposted_by,
            images: Vec::new(),
        })
    }

    pub fn will_i_survive(&self, prefer_to: &PreferencesViewModel) -> bool {
        if prefer_to.hide_reposts && self.reposted_by.is_some() {
            return false;
        }
        if prefer_to.hide_replies && self.replied_by.is_some() {
            return false;
        }
        if prefer_to.hide_quote_posts
            && matches!(
                self.meta.as_ref().map(|emb| &emb.kind),
                Some(EmbedKind::Record(_))
            )
        {
            return false;
        }

        true
    }
}

fn parse_facets_from_record(record: &serde_json::Value) -> Vec<Facet> {
    let mut facets = Vec::new();
    if let Some(response) = record.get("facets").and_then(|f| f.as_array()) {
        for facet in response {
            let index = facet
                .get("index")
                .expect("There's meant to be an index here");

            let start = index.get("byteStart").and_then(|v| v.as_u64()).unwrap() as usize;
            let end = index.get("byteEnd").and_then(|v| v.as_u64()).unwrap() as usize;

            if let Some(features) = facet.get("features").and_then(|f| f.as_array()) {
                for feature in features {
                    let lexicon_type_arg = feature.get("$type").and_then(|t| t.as_str()).unwrap();
                    let kind = match lexicon_type_arg {
                        // TODO: These all need to be refactored to use MainData
                        "app.bsky.richtext.facet#mention" => {
                            let did = feature.get("did").and_then(|d| d.as_str()).unwrap();
                            FacetKind::Mention(did.to_string())
                        }
                        "app.bsky.richtext.facet#link" => {
                            let uri = feature.get("uri").and_then(|u| u.as_str()).unwrap();
                            FacetKind::Link(uri.to_string())
                        }
                        "app.bsky.richtext.facet#tag" => {
                            let tag = feature.get("tag").and_then(|t| t.as_str()).unwrap();
                            FacetKind::Tag(tag.to_string())
                        }
                        _ => continue,
                    };
                    facets.push(Facet { start, end, kind });
                }
            }
        }
    }
    facets
}

fn get_embed_meta(embed: &Union<PostViewEmbedRefs>) -> Option<EmbedMeta> {
    match embed {
        Union::Refs(PostViewEmbedRefs::AppBskyEmbedExternalView(ext)) => Some(EmbedMeta {
            kind: EmbedKind::ExternalLink,
            title: Some(ext.external.title.clone()),
            description: Some(ext.external.description.clone()),
            url: Some(ext.external.uri.clone()),
        }),
        Union::Refs(PostViewEmbedRefs::AppBskyEmbedImagesView(imgs)) => Some(EmbedMeta {
            kind: EmbedKind::Images(
                imgs.images
                    .iter()
                    .map(|i| ImageMeta {
                        size: i.fullsize.clone(),
                        alt: Some(i.alt.clone()),
                    })
                    .collect(),
            ),
            title: None,
            description: None,
            url: None,
        }),
        Union::Refs(PostViewEmbedRefs::AppBskyEmbedVideoView(_)) => Some(EmbedMeta {
            kind: EmbedKind::Video,
            title: None,
            description: None,
            url: None,
        }),
        Union::Refs(PostViewEmbedRefs::AppBskyEmbedRecordView(rec)) => {
            let quoted = qp(&rec.record);
            Some(EmbedMeta {
                kind: EmbedKind::Record(Box::new(quoted)),
                title: None,
                description: None,
                url: None,
            })
        }
        Union::Refs(PostViewEmbedRefs::AppBskyEmbedRecordWithMediaView(rec)) => {
            let quoted = qp(&rec.record.record);
            Some(EmbedMeta {
                kind: EmbedKind::RecordWithMedia(Box::new(quoted)),
                title: None,
                description: None,
                url: None,
            })
        }
        _ => None,
    }
}

fn qp(record_refs: &Union<ViewRecordRefs>) -> QuotedPost {
    match record_refs {
        Union::Refs(ViewRecordRefs::ViewRecord(vr)) => {
            let author = &vr.author;
            let handle = author.handle.to_string();
            let display_name = author.display_name.clone().unwrap();

            let (text, facets) = match serde_json::to_value(&vr.value) {
                Ok(val) => {
                    let t = val
                        .get("text")
                        .and_then(|v| v.as_str())
                        .unwrap()
                        .to_string();
                    let f = parse_facets_from_record(&val);
                    (t, f)
                }
                Err(_) => (String::new(), Vec::new()),
            };

            let meta = vr
                .embeds
                .as_ref()
                .and_then(|embeds| embeds.first().and_then(embed_io));

            QuotedPost {
                uri: vr.uri.clone(),
                cid: vr.cid.as_ref().to_string(),
                handle,
                display_name,
                text,
                facets,
                meta,
            }
        }
        Union::Refs(ViewRecordRefs::ViewNotFound(_)) => QuotedPost {
            uri: String::new(),
            cid: String::new(),
            handle: String::new(),
            display_name: String::new(),
            text: "[Post not found]".to_string(),
            facets: Vec::new(),
            meta: None,
        },
        Union::Refs(ViewRecordRefs::ViewBlocked(_)) => QuotedPost {
            uri: String::new(),
            cid: String::new(),
            handle: String::new(),
            display_name: String::new(),
            text: "[Post blocked]".to_string(),
            facets: Vec::new(),
            meta: None,
        },
        Union::Refs(ViewRecordRefs::ViewDetached(_)) => QuotedPost {
            uri: String::new(),
            cid: String::new(),
            handle: String::new(),
            display_name: String::new(),
            text: "[Detached quote]".to_string(),
            facets: Vec::new(),
            meta: None,
        },
        _ => QuotedPost {
            uri: String::new(),
            cid: String::new(),
            handle: String::new(),
            display_name: String::new(),
            text: "[Record]".to_string(),
            facets: Vec::new(),
            meta: None,
        },
    }
}

fn embed_io(embed: &Union<ViewRecordEmbedsItem>) -> Option<EmbedMeta> {
    match embed {
        Union::Refs(ViewRecordEmbedsItem::AppBskyEmbedExternalView(ext)) => Some(EmbedMeta {
            kind: EmbedKind::ExternalLink,
            title: Some(ext.external.title.clone()),
            description: Some(ext.external.description.clone()),
            url: Some(ext.external.uri.clone()),
        }),
        Union::Refs(ViewRecordEmbedsItem::AppBskyEmbedImagesView(imgs)) => Some(EmbedMeta {
            kind: EmbedKind::Images(
                imgs.images
                    .iter()
                    .map(|i| ImageMeta {
                        size: i.fullsize.clone(),
                        alt: Some(i.alt.clone()),
                    })
                    .collect(),
            ),
            title: None,
            description: None,
            url: None,
        }),
        Union::Refs(ViewRecordEmbedsItem::AppBskyEmbedVideoView(_)) => Some(EmbedMeta {
            kind: EmbedKind::Video,
            title: None,
            description: None,
            url: None,
        }),
        Union::Refs(ViewRecordEmbedsItem::AppBskyEmbedRecordView(rec)) => {
            let quoted = qp(&rec.record);
            Some(EmbedMeta {
                kind: EmbedKind::Record(Box::new(quoted)),
                title: None,
                description: None,
                url: None,
            })
        }
        Union::Refs(ViewRecordEmbedsItem::AppBskyEmbedRecordWithMediaView(rec)) => {
            let quoted = qp(&rec.record.record);
            Some(EmbedMeta {
                kind: EmbedKind::RecordWithMedia(Box::new(quoted)),
                title: None,
                description: None,
                url: None,
            })
        }
        _ => None,
    }
}

fn replying_handle(parent: &Union<ReplyRefParentRefs>) -> Option<String> {
    match parent {
        Union::Refs(ReplyRefParentRefs::PostView(pv)) => {
            Some(pv.author.display_name.clone().unwrap())
        }
        _ => None,
    }
}

fn get_reposted_reason(reason: &Union<FeedViewPostReasonRefs>) -> Option<String> {
    match reason {
        Union::Refs(FeedViewPostReasonRefs::ReasonRepost(rr)) => {
            Some(rr.by.display_name.clone().unwrap())
        }
        _ => None,
    }
}
