// MIT License
//
// Copyright (c) 2026 Abhijna Micro (OPC) PVT LTD
// Author: Abhijna Micro Team <team@abhijnamicro.in>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use async_trait::async_trait;
use grass_core::{
    GrassAgent, GrassAgentBuilder, GrassAgentMetadata, GrassAgentState, GrassError, GrassMessage,
    GrassTaskPayload, GrassTaskResult, GrassTaskStatus, Result,
};
use grass_llm::{
    GrassChatMessage, GrassChoice, GrassLlmProvider, GrassLlmRequest, GrassLlmResponse,
    GrassLlmService, GrassMessageRole, GrassStreamEvent, GrassUsage,
};
use grass_runtime::{GrassPriority, GrassScheduledTask, GrassTaskScheduler};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
enum SocialPlatform {
    Facebook,
    Instagram,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SocialPost {
    id: String,
    platform: SocialPlatform,
    author: String,
    content: String,
    likes: u32,
    comments: u32,
}

#[async_trait]
trait GrassSocialFeed: Send + Sync {
    async fn fetch_posts(&self, limit: usize) -> Result<Vec<SocialPost>>;
}

struct GrassFacebookFeed;

#[async_trait]
impl GrassSocialFeed for GrassFacebookFeed {
    async fn fetch_posts(&self, _limit: usize) -> Result<Vec<SocialPost>> {
        Ok(vec![
            SocialPost {
                id: "fb-1001".into(),
                platform: SocialPlatform::Facebook,
                author: "Northwind Mills".into(),
                content:
                    "Just launched our new organic spice line! Limited first-batch availability."
                        .into(),
                likes: 1240,
                comments: 87,
            },
            SocialPost {
                id: "fb-1002".into(),
                platform: SocialPlatform::Facebook,
                author: "City Bikes Co".into(),
                content: "Weekend group ride this Saturday, all levels welcome.".into(),
                likes: 310,
                comments: 23,
            },
        ])
    }
}

struct GrassInstagramFeed;

#[async_trait]
impl GrassSocialFeed for GrassInstagramFeed {
    async fn fetch_posts(&self, _limit: usize) -> Result<Vec<SocialPost>> {
        Ok(vec![
            SocialPost {
                id: "ig-2001".into(),
                platform: SocialPlatform::Instagram,
                author: "barista.notes".into(),
                content: "Latte art masterclass results from this week. Very happy with the pours!"
                    .into(),
                likes: 890,
                comments: 41,
            },
            SocialPost {
                id: "ig-2002".into(),
                platform: SocialPlatform::Instagram,
                author: "ghost_account".into(),
                content: "".into(),
                likes: 12,
                comments: 1,
            },
        ])
    }
}

struct GrassCommentCraftProvider {
    model: String,
}

#[async_trait]
impl GrassLlmProvider for GrassCommentCraftProvider {
    fn name(&self) -> &str {
        "comment-craft"
    }

    async fn complete(&self, request: GrassLlmRequest) -> Result<GrassLlmResponse> {
        let prompt = request
            .messages
            .iter()
            .find(|m| m.role == GrassMessageRole::User)
            .map(|m| m.content.clone())
            .unwrap_or_default();

        let post: SocialPost = serde_json::from_str(&prompt)
            .map_err(|e| GrassError::Llm(format!("no analyzable post content: {}", e)))?;

        if post.content.trim().is_empty() {
            return Err(GrassError::Llm("post has no content to analyze".into()));
        }

        let engagement = if post.likes > 1000 {
            "high"
        } else {
            "moderate"
        };
        let comment = format!(
            "Thanks @{}! This resonates with a lot of people — {} is generating {} engagement. We'd love to hear about it in the comments.",
            post.author,
            post.content.split('.').next().unwrap_or(&post.content),
            engagement
        );

        Ok(GrassLlmResponse {
            id: Uuid::new_v4().to_string(),
            model: self.model.clone(),
            choices: vec![GrassChoice {
                index: 0,
                message: GrassChatMessage {
                    role: GrassMessageRole::Assistant,
                    content: comment,
                    name: None,
                },
                finish_reason: Some("stop".into()),
            }],
            usage: GrassUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
        })
    }

    async fn stream<'a>(
        &'a self,
        _request: GrassLlmRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<GrassStreamEvent>> + Send + 'a>> {
        Err(GrassError::Llm(
            "comment-craft does not support streaming".into(),
        ))
    }
}

struct GrassSocialAgent {
    metadata: GrassAgentMetadata,
    state: GrassAgentState,
    llm: GrassLlmService,
}

impl GrassSocialAgent {
    fn new(llm: GrassLlmService) -> Self {
        let metadata = GrassAgentBuilder::new("social-comment-agent")
            .version("0.1.0")
            .description("Fetches Facebook/Instagram posts, drafts engaging replies with an LLM, and posts or queues them for approval")
            .tag("social")
            .tag("scheduler")
            .build();
        Self {
            metadata,
            state: GrassAgentState::Idle,
            llm,
        }
    }

    async fn fetch_posts(&self) -> Result<Vec<SocialPost>> {
        let feeds: Vec<Box<dyn GrassSocialFeed>> =
            vec![Box::new(GrassFacebookFeed), Box::new(GrassInstagramFeed)];
        let mut posts = Vec::new();
        for feed in feeds {
            posts.extend(feed.fetch_posts(50).await?);
        }
        Ok(posts)
    }

    async fn generate_comment(&self, post: &SocialPost) -> Result<String> {
        let response = self
            .llm
            .complete(GrassLlmRequest {
                model: "comment-craft-model".into(),
                messages: vec![
                    GrassChatMessage {
                        role: GrassMessageRole::System,
                        content:
                            "You craft short, warm, engagement-driving replies for social posts."
                                .into(),
                        name: None,
                    },
                    GrassChatMessage {
                        role: GrassMessageRole::User,
                        content: serde_json::to_string(post).unwrap_or_default(),
                        name: None,
                    },
                ],
                temperature: Some(0.7),
                max_tokens: Some(120),
                stream: false,
                tools: None,
            })
            .await?;
        response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| GrassError::Llm("empty choices from LLM".into()))
    }
}

#[async_trait]
impl GrassAgent for GrassSocialAgent {
    fn metadata(&self) -> &GrassAgentMetadata {
        &self.metadata
    }
    fn state(&self) -> GrassAgentState {
        self.state.clone()
    }

    async fn start(&mut self) -> Result<()> {
        self.state = GrassAgentState::Idle;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.state = GrassAgentState::Idle;
        Ok(())
    }

    async fn execute(&self, task: GrassTaskPayload) -> Result<GrassTaskResult> {
        let start = std::time::Instant::now();

        let (status, output) = match task.action.as_str() {
            "fetch_posts" => {
                let posts = self.fetch_posts().await?;
                (
                    GrassTaskStatus::Completed,
                    serde_json::to_value(posts).unwrap_or_default(),
                )
            }
            "generate_comment" => {
                let post: SocialPost = serde_json::from_value(task.input.clone())
                    .map_err(|e| GrassError::Task(format!("invalid post payload: {}", e)))?;
                let comment = self.generate_comment(&post).await?;
                (
                    GrassTaskStatus::Completed,
                    serde_json::json!({ "post_id": post.id, "comment": comment }),
                )
            }
            "post_comment" => {
                let post_id = task.input["post_id"].as_str().unwrap_or("unknown");
                let comment = task.input["comment"].as_str().unwrap_or_default();
                let text = format!("[auto] Posted reply on {}: {}", post_id, comment);
                (
                    GrassTaskStatus::Completed,
                    serde_json::json!({ "posted": text }),
                )
            }
            other => {
                return Err(GrassError::Task(format!("unknown action '{}'", other)));
            }
        };

        Ok(GrassTaskResult {
            task_id: task.task_id,
            agent_id: task.agent_id,
            status,
            output,
            error: None,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    async fn handle_message(&self, _msg: GrassMessage) -> Result<Option<GrassMessage>> {
        Ok(None)
    }
}

#[derive(Debug, Clone)]
struct GrassCommentDraft {
    post_id: String,
    platform: String,
    comment: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GrassApprovalPolicy {
    AutoPost,
    RequireApproval,
}

fn task_payload(action: &str, input: serde_json::Value, agent_id: Uuid) -> GrassTaskPayload {
    GrassTaskPayload {
        task_id: Uuid::new_v4(),
        agent_id,
        action: action.into(),
        input,
        metadata: HashMap::new(),
    }
}

fn priority_for(post: &SocialPost) -> GrassPriority {
    if post.likes >= 1000 {
        GrassPriority::Critical
    } else if post.likes >= 500 {
        GrassPriority::High
    } else {
        GrassPriority::Normal
    }
}

async fn drain_scheduler(
    scheduler: &GrassTaskScheduler,
    agent: &GrassSocialAgent,
    policy: GrassApprovalPolicy,
    drafts: &Arc<Mutex<Vec<GrassCommentDraft>>>,
    posted: &Arc<Mutex<Vec<String>>>,
) -> Result<()> {
    loop {
        if scheduler.queue_len().await == 0 && scheduler.active_count().await == 0 {
            break;
        }
        let Some(scheduled) = scheduler.dequeue().await else {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            continue;
        };

        let action = scheduled.task.action.clone();
        match agent.execute(scheduled.task).await {
            Ok(result) => match action.as_str() {
                "fetch_posts" => {
                    let posts: Vec<SocialPost> =
                        serde_json::from_value(result.output).unwrap_or_default();
                    println!("[scheduler] fetched {} posts", posts.len());
                    for post in posts {
                        let priority = priority_for(&post);
                        println!(
                            "[scheduler] enqueue generate_comment for {} (priority {:?})",
                            post.id, priority
                        );
                        let payload = task_payload(
                            "generate_comment",
                            serde_json::to_value(&post).unwrap_or_default(),
                            agent.metadata().id,
                        );
                        scheduler
                            .enqueue(GrassScheduledTask::new(payload, priority))
                            .await?;
                    }
                }
                "generate_comment" => {
                    let post_id = result.output["post_id"]
                        .as_str()
                        .unwrap_or("unknown")
                        .to_string();
                    let comment = result.output["comment"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string();
                    match policy {
                        GrassApprovalPolicy::AutoPost => {
                            let payload = task_payload(
                                "post_comment",
                                serde_json::json!({ "post_id": post_id, "comment": comment }),
                                agent.metadata().id,
                            );
                            scheduler
                                .enqueue(GrassScheduledTask::new(payload, GrassPriority::Low))
                                .await?;
                        }
                        GrassApprovalPolicy::RequireApproval => {
                            println!("[draft] awaiting user approval -> {}", comment);
                            let mut store = drafts.lock().await;
                            store.push(GrassCommentDraft {
                                post_id: post_id.clone(),
                                platform: if post_id.starts_with("fb") {
                                    "facebook".into()
                                } else {
                                    "instagram".into()
                                },
                                comment,
                            });
                        }
                    }
                }
                "post_comment" => {
                    if let Some(text) = result.output["posted"].as_str() {
                        println!("{}", text);
                        posted.lock().await.push(text.to_string());
                    }
                }
                _ => {}
            },
            Err(err) => {
                println!("[error] task '{}' failed: {}", action, err);
                if let Some(entry) = err.entry() {
                    println!(
                        "  -> table: code={} severity={} remediation={:?}",
                        entry.code, entry.severity, entry.remediation
                    );
                }
            }
        }

        scheduler.complete_task().await;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut llm = GrassLlmService::new();
    llm.register_provider(
        "comment-craft",
        Box::new(GrassCommentCraftProvider {
            model: "comment-craft-1".into(),
        }),
    );

    let agent = GrassSocialAgent::new(llm);
    let agent_id = agent.metadata().id;
    let scheduler = GrassTaskScheduler::new(2);
    let drafts: Arc<Mutex<Vec<GrassCommentDraft>>> = Arc::new(Mutex::new(Vec::new()));
    let posted: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let policy = if std::env::var("GRASS_DRAFT_ONLY").is_ok() {
        GrassApprovalPolicy::RequireApproval
    } else {
        GrassApprovalPolicy::AutoPost
    };

    println!("=== grass scheduling demo ===");
    println!("approval policy: {:?}", policy);
    println!("=== phase 1: schedule fetch from facebook/instagram ===");

    scheduler
        .enqueue(GrassScheduledTask::new(
            task_payload("fetch_posts", serde_json::json!({}), agent_id),
            GrassPriority::Critical,
        ))
        .await?;

    drain_scheduler(&scheduler, &agent, policy, &drafts, &posted).await?;

    println!("=== summary ===");
    println!("posted comments: {}", posted.lock().await.len());
    let draft_count = drafts.lock().await.len();
    println!("drafts awaiting user approval: {}", draft_count);
    if draft_count > 0 {
        for d in drafts.lock().await.iter() {
            println!("  - [draft] {} on {}: {}", d.platform, d.post_id, d.comment);
        }
        println!("run with GRASS_DRAFT_ONLY=1 to enable the approval workflow, or approve these ");
        println!("in a real deployment before publishing.");
    }

    Ok(())
}
