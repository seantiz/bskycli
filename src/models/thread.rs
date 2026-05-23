use super::post::PostViewModel;
use atrium_api::app::bsky::feed::defs::{
    ThreadViewPost, ThreadViewPostParentRefs, ThreadViewPostRepliesItem,
};
use atrium_api::types::Union;

/// [`ThreadViewModel`] formats threads with parents above the focal post and replies below.
/// The formatting is for when user selects a thread from a timeline to view its surrounding discussion.
/// See [`PostViewModel`].
#[derive(Debug, Clone)]
pub struct ThreadViewModel {
    pub parents: Vec<PostViewModel>,
    pub focal: PostViewModel,
    pub replies: Vec<PostViewModel>,
}

impl ThreadViewModel {
    pub fn from_thread_view_post(tvp: &ThreadViewPost) -> Option<Self> {
        let focal = PostViewModel::from_post_view(&tvp.post)?;

        let mut parents = Vec::new();
        Self::collect_parents(&tvp.parent, &mut parents);
        parents.reverse();

        let mut replies = Vec::new();
        if let Some(ref reply_list) = tvp.replies {
            for reply in reply_list {
                Self::collect_reply(reply, &mut replies);
            }
        }

        Some(ThreadViewModel {
            parents,
            focal,
            replies,
        })
    }

    fn collect_parents(
        parent: &Option<Union<ThreadViewPostParentRefs>>,
        out: &mut Vec<PostViewModel>,
    ) {
        if let Some(Union::Refs(ThreadViewPostParentRefs::ThreadViewPost(tvp))) = parent {
            if let Some(post) = PostViewModel::from_post_view(&tvp.post) {
                out.push(post);
            }
            Self::collect_parents(&tvp.parent, out);
        }
    }

    fn collect_reply(reply: &Union<ThreadViewPostRepliesItem>, out: &mut Vec<PostViewModel>) {
        if let Union::Refs(ThreadViewPostRepliesItem::ThreadViewPost(tvp)) = reply
            && let Some(post) = PostViewModel::from_post_view(&tvp.post)
        {
            out.push(post);
        }
    }
}
