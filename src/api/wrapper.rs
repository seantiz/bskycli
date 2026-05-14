use atrium_api::com::atproto::repo::delete_record::OutputData;
use atrium_api::types::Object;
use atrium_api::types::string::Datetime;
use bsky_sdk::Result;

use bsky_sdk::BskyAgent;
use bsky_sdk::agent::config::{Config, FileStore};

use crate::models::post::PostViewModel;
use crate::models::profile::ProfileViewModel;
use crate::models::thread::ThreadViewModel;

pub struct AgentWrapper {
    pub agent: BskyAgent,
}

impl AgentWrapper {
    pub async fn spinupagain() -> Result<Self> {
        let start_with_this = dirs::config_dir()
            .expect("Couldn't start the first step of logging in")
            .join("bskycli/config.json");

        let some_bs = FileStore::new(&start_with_this);

        let config = match Config::load(&some_bs).await {
            Ok(c) => c,
            Err(_) => Config::default(),
        };

        // WARN: Don't assume an agent can be built from stored config

        let agent = match BskyAgent::builder().config(config).build().await {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Bsky has invalidated the prior session: {}", e);
                BskyAgent::builder().build().await?
            }
        };

        Ok(AgentWrapper { agent })
    }

    pub async fn get_timeline(
        &self,
        cursor: Option<String>,
        limit: Option<u8>,
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
            .collect();

        Ok((posts, output.cursor.clone()))
    }

    pub async fn get_thread(&self, uri: &str) -> Result<Option<ThreadViewModel>> {
        let params = atrium_api::app::bsky::feed::get_post_thread::ParametersData {
            depth: Some(6u16.try_into().unwrap()),
            parent_height: Some(10u16.try_into().unwrap()),
            uri: uri.to_string(),
        };
        let output = match self
            .agent
            .api
            .app
            .bsky
            .feed
            .get_post_thread(params.clone().into())
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
                    .get_post_thread(params.into())
                    .await?
            }
        };

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

        let result = match self.agent.create_record(record.clone()).await {
            Ok(r) => r,
            Err(_) => {
                self.agent.api.com.atproto.server.refresh_session().await?;
                self.agent.create_record(record).await?
            }
        };
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

        let result = match self.agent.create_record(record.clone()).await {
            Ok(r) => r,
            Err(_) => {
                self.agent.api.com.atproto.server.refresh_session().await?;
                self.agent.create_record(record).await?
            }
        };
        Ok(result.uri.to_string())
    }

    pub async fn unlike(&self, like_uri: &str) -> Result<Object<OutputData>> {
        match self.agent.delete_record(like_uri).await {
            Ok(r) => Ok(r),
            Err(_) => {
                self.agent.api.com.atproto.server.refresh_session().await?;
                self.agent.delete_record(like_uri).await
            }
        }
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

        let result = match self.agent.create_record(record.clone()).await {
            Ok(r) => r,
            Err(_) => {
                self.agent.api.com.atproto.server.refresh_session().await?;
                self.agent.create_record(record).await?
            }
        };
        Ok(result.uri.to_string())
    }

    pub async fn unrepost(&self, repost_uri: &str) -> Result<Object<OutputData>> {
        match self.agent.delete_record(repost_uri).await {
            Ok(r) => Ok(r),
            Err(_) => {
                self.agent.api.com.atproto.server.refresh_session().await?;
                self.agent.delete_record(repost_uri).await
            }
        }
    }

    pub async fn get_profile(&self, actor: &str) -> Result<ProfileViewModel> {
        let params = atrium_api::app::bsky::actor::get_profile::ParametersData {
            actor: actor.parse().expect("valid handle or did"),
        };
        let output = match self
            .agent
            .api
            .app
            .bsky
            .actor
            .get_profile(params.clone().into())
            .await
        {
            Ok(o) => o,
            Err(_) => {
                self.agent.api.com.atproto.server.refresh_session().await?;
                self.agent
                    .api
                    .app
                    .bsky
                    .actor
                    .get_profile(params.into())
                    .await?
            }
        };

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
        let output = match self
            .agent
            .api
            .app
            .bsky
            .feed
            .get_author_feed(params.clone().into())
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
                    .get_author_feed(
                        atrium_api::app::bsky::feed::get_author_feed::ParametersData {
                            actor: actor_str
                                .parse()
                                .expect("Couldn't recognise your login info"),
                            cursor,
                            filter: None,
                            include_pins: None,
                            // WARN: This is from a LimitedNonZeroU8 type in atrium
                            limit: 50u8.try_into().ok(),
                        }
                        .into(),
                    )
                    .await?
            }
        };

        let posts: Vec<PostViewModel> = output
            .feed
            .iter()
            .filter_map(PostViewModel::from_feed_view_post)
            .collect();

        Ok((posts, output.cursor.clone()))
    }
}

#[derive(Debug, Clone)]
pub struct ReplyRef {
    pub parent_uri: String,
    pub parent_cid: String,
    pub root_uri: String,
    pub root_cid: String,
}
