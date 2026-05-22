use std::path::PathBuf;

use atrium_api::com::atproto::repo::delete_record::OutputData;
use atrium_api::types::Object;
use atrium_api::types::string::Datetime;
use bsky_sdk::Result;

use bsky_sdk::BskyAgent;
use bsky_sdk::agent::config::{Config, FileStore};

use crate::models::notifications::NotificationViewModel;
use crate::models::post::PostViewModel;
use crate::models::preferences::PreferencesViewModel;
use crate::models::profile::ProfileViewModel;
use crate::models::thread::ThreadViewModel;

use keyring_core::Entry;

pub struct AgentWrapper {
    pub agent: BskyAgent,
}

impl AgentWrapper {
    pub async fn many_sessions(&self, from_config: PathBuf, handle: String) -> anyhow::Result<()> {
        let next_time = Entry::new("bskycli", "user")?;
        let password = next_time.get_password()?;

        self.agent.login(&handle, &password).await?;
        self.agent
            .to_config()
            .await
            .save(&FileStore::new(from_config))
            .await?;
        Ok(())
    }

    pub async fn spinupagain() -> anyhow::Result<Self> {
        let startup = dirs::config_dir()
            .expect("Couldn't find app support folder")
            .join("bskycli/config.json");

        // NOTE: Memory Session Store will steer to the right endpoint
        let mss = FileStore::new(&startup);

        let config = Config::load(&mss).await.unwrap_or_default();

        // Don't assume a session exists
        let maybe_handle = config.session.as_ref().map(|s| s.handle.to_string());

        // Endpoint is not always needed - may be a cleaner way
        let maybe_new = config.endpoint.clone();

        let (reborn, agent) = match BskyAgent::builder().config(config).build().await {
            Ok(a) => (false, a),
            Err(_) => {
                eprintln!("ATP server has invalidated the prior session");
                let new_session = Config {
                    endpoint: maybe_new,
                    session: None,
                    ..Default::default()
                };
                (
                    true,
                    BskyAgent::builder().config(new_session).build().await?,
                )
            }
        };

        let wrapper = AgentWrapper { agent };

        if reborn {
            wrapper
                .many_sessions(
                    startup,
                    maybe_handle.expect("No username provided from local config"),
                )
                .await?;
        }

        Ok(wrapper)
    }

    pub async fn get_timeline(
        &self,
        cursor: Option<String>,
        limit: Option<u8>,
        preferences: PreferencesViewModel,
    ) -> Result<(Vec<PostViewModel>, Option<String>)> {
        let params = atrium_api::app::bsky::feed::get_timeline::ParametersData {
            algorithm: None,
            cursor,
            limit: limit.and_then(|l| l.try_into().ok()),
        };

        let output = match self
            .agent
            .api
            .app
            .bsky
            .feed
            .get_timeline(params.clone().into())
            .await
        {
            Ok(o) => o,
            Err(_) => {
                self.agent.api.com.atproto.server.refresh_session().await?;
                self.agent
                    .api
                    .app
                    .bsky
                    .feed
                    .get_timeline(params.into())
                    .await?
            }
        };

        let posts: Vec<PostViewModel> = output
            .feed
            .iter()
            .filter_map(PostViewModel::from_feed_view_post)
            .filter(|p| p.will_i_survive(&preferences))
            .collect();

        Ok((posts, output.cursor.clone()))
    }

    pub async fn get_thread(&self, uri: &str) -> Result<Option<ThreadViewModel>> {
        let params = atrium_api::app::bsky::feed::get_post_thread::ParametersData {
            depth: Some(6u16.try_into().unwrap()),
            parent_height: Some(10u16.try_into().unwrap()),
            uri: uri.to_string(),
        };
        let output = self
            .agent
            .api
            .app
            .bsky
            .feed
            .get_post_thread(params.clone().into())
            .await?;

        use atrium_api::app::bsky::feed::get_post_thread::OutputThreadRefs;
        use atrium_api::types::Union;

        match &output.thread {
            Union::Refs(OutputThreadRefs::AppBskyFeedDefsThreadViewPost(tvp)) => {
                Ok(ThreadViewModel::from_thread_view_post(tvp))
            }
            _ => Ok(None),
        }
    }

    pub async fn create_post(&self, text: String, reply_to: Option<ReplyRef>) -> Result<String> {
        let facets = {
            let rt = bsky_sdk::rich_text::RichText::new_with_detect_facets(&text).await?;
            rt.facets
        };

        let reply = reply_to.map(|r| {
            atrium_api::app::bsky::feed::post::ReplyRefData {
                parent: atrium_api::com::atproto::repo::strong_ref::MainData {
                    cid: r.parent_cid.parse().expect("valid cid"),
                    uri: r.parent_uri.clone(),
                }
                .into(),
                root: atrium_api::com::atproto::repo::strong_ref::MainData {
                    cid: r.root_cid.parse().expect("valid cid"),
                    uri: r.root_uri.clone(),
                }
                .into(),
            }
            .into()
        });

        let record = atrium_api::app::bsky::feed::post::RecordData {
            created_at: Datetime::now(),
            embed: None,
            entities: None,
            facets,
            labels: None,
            langs: None,
            reply,
            tags: None,
            text,
        };

        let result = self.agent.create_record(record.clone()).await?;

        Ok(result.uri.to_string())
    }

    pub async fn like(&self, uri: &str, cid: &str) -> Result<String> {
        let record = atrium_api::app::bsky::feed::like::RecordData {
            created_at: Datetime::now(),
            subject: atrium_api::com::atproto::repo::strong_ref::MainData {
                cid: cid.parse().expect("valid cid"),
                uri: uri.to_string(),
            }
            .into(),
            via: None,
        };

        let result = self.agent.create_record(record.clone()).await?;

        Ok(result.uri.to_string())
    }

    pub async fn unlike(&self, like_uri: &str) -> Result<Object<OutputData>> {
        self.agent.delete_record(like_uri).await
    }

    pub async fn repost(&self, uri: &str, cid: &str) -> Result<String> {
        let record = atrium_api::app::bsky::feed::repost::RecordData {
            created_at: Datetime::now(),
            subject: atrium_api::com::atproto::repo::strong_ref::MainData {
                cid: cid.parse().expect("valid cid"),
                uri: uri.to_string(),
            }
            .into(),
            via: None,
        };

        let result = self.agent.create_record(record.clone()).await?;
        Ok(result.uri.to_string())
    }

    pub async fn unrepost(&self, repost_uri: &str) -> Result<Object<OutputData>> {
        self.agent.delete_record(repost_uri).await
    }

    pub async fn get_profile(&self, actor: &str) -> Result<ProfileViewModel> {
        let params = atrium_api::app::bsky::actor::get_profile::ParametersData {
            actor: actor.parse().expect("valid handle or did"),
        };
        let output = self
            .agent
            .api
            .app
            .bsky
            .actor
            .get_profile(params.clone().into())
            .await?;

        Ok(ProfileViewModel::from_detailed(&output))
    }

    pub async fn get_author_feed(
        &self,
        actor: &str,
        cursor: Option<String>,
    ) -> Result<(Vec<PostViewModel>, Option<String>)> {
        let actor_str = actor.to_string();
        let params = atrium_api::app::bsky::feed::get_author_feed::ParametersData {
            actor: actor_str
                .clone()
                .parse()
                .expect("Couldn't recognise your login info"),
            cursor: cursor.clone(),
            filter: None,
            include_pins: None,
            // WARN: This is from a LimitedNonZeroU8 type in atrium
            limit: 50u8.try_into().ok(),
        };
        let output = self
            .agent
            .api
            .app
            .bsky
            .feed
            .get_author_feed(params.clone().into())
            .await?;

        let posts: Vec<PostViewModel> = output
            .feed
            .iter()
            .filter_map(PostViewModel::from_feed_view_post)
            .collect();

        Ok((posts, output.cursor.clone()))
    }

    pub async fn search_firehose(
        &self,
        search_query: String,
        cursor: Option<String>,
    ) -> Result<(Vec<PostViewModel>, Option<String>)> {
        let endpoint = atrium_api::app::bsky::feed::search_posts::ParametersData {
            q: search_query,
            sort: Some("latest".to_string()),
            author: None,
            cursor,
            domain: None,
            lang: None,
            limit: None,
            mentions: None,
            since: None,
            tag: None,
            until: None,
            url: None,
        };

        let output = self
            .agent
            .api
            .app
            .bsky
            .feed
            .search_posts(endpoint.into())
            .await?;

        let posts: Vec<PostViewModel> = output
            .posts
            .iter()
            .filter_map(PostViewModel::from_post_view)
            .collect();

        Ok((posts, output.cursor.clone()))
    }

    pub async fn get_notifications(
        &self,
        cursor: Option<String>,
    ) -> Result<(Vec<NotificationViewModel>, Option<String>)> {
        let preferences = PreferencesViewModel::load();

        let endpoint = atrium_api::app::bsky::notification::list_notifications::ParametersData {
            limit: 100u8.try_into().ok(),
            reasons: preferences.enabled_notifications(),
            cursor,
            seen_at: None,
            priority: None,
        };

        let output = self
            .agent
            .api
            .app
            .bsky
            .notification
            .list_notifications(endpoint.into())
            .await?;

        let threshold = match &preferences.last_seen_at {
            Some(s) => chrono::DateTime::parse_from_rfc3339(s)
                .map(|dt| (dt - chrono::Duration::days(2)).with_timezone(&chrono::Utc))
                .ok(),
            None => Some(chrono::Utc::now() - chrono::Duration::days(2)),
        };

        let notifications: Vec<NotificationViewModel> = output
            .notifications
            .iter()
            .filter_map(NotificationViewModel::from_notification)
            .filter(|n| match threshold {
                Some(t) => chrono::DateTime::parse_from_rfc3339(&n.indexed_at)
                    .map(|dt| dt > t)
                    .unwrap_or(true),
                None => true,
            })
            .collect();

        Ok((notifications, output.cursor.clone()))
    }

    pub async fn update_seen(&self, seen_at: Option<Datetime>) -> Result<()> {
        let seen = seen_at.unwrap_or_else(Datetime::now);
        let params = atrium_api::app::bsky::notification::update_seen::InputData {
            seen_at: seen.clone(),
        };
        self.agent
            .api
            .app
            .bsky
            .notification
            .update_seen(params.into())
            .await?;

        let mut prefs = PreferencesViewModel::load();
        prefs.last_seen_at = Some(seen.as_str().to_string());
        let _ = prefs.save();

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ReplyRef {
    pub parent_uri: String,
    pub parent_cid: String,
    pub root_uri: String,
    pub root_cid: String,
}
