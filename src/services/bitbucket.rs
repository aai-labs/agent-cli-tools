use reqwest::Method;
use serde_json::{json, Value};

use crate::{
    cli::*,
    config::Context,
    error::AppError,
    http::ApiClient,
    input,
    services::shared::{
        bitbucket_base, bitbucket_repo, enc, workspace, write_download, CtxProfile,
    },
};

pub(crate) async fn dispatch(
    client: &ApiClient,
    ctx: &Context,
    command: BitbucketCommand,
) -> Result<Value, AppError> {
    match command.resource {
        BitbucketResource::Repos(command) => match command.action {
            ListGetAction::List(args) => {
                let workspace = workspace(ctx.profile(), "repos.list")?;
                let url = format!(
                    "{}/repositories/{}?pagelen={}",
                    bitbucket_base(ctx.profile()),
                    enc(workspace),
                    args.limit
                );
                client
                    .request(
                        "bitbucket",
                        "repos.list",
                        ctx.profile(),
                        Method::GET,
                        url,
                        None,
                    )
                    .await
            }
            ListGetAction::Get(args) => {
                let (workspace, repo) = bitbucket_repo(ctx.profile(), Some(&args.id), "repos.get")?;
                let url = format!(
                    "{}/repositories/{}/{}",
                    bitbucket_base(ctx.profile()),
                    enc(workspace),
                    enc(repo)
                );
                client
                    .request(
                        "bitbucket",
                        "repos.get",
                        ctx.profile(),
                        Method::GET,
                        url,
                        None,
                    )
                    .await
            }
        },
        BitbucketResource::Prs(command) => match command.action {
            BitbucketPullRequestAction::List(args) => {
                let (workspace, repo) =
                    bitbucket_repo(ctx.profile(), args.repo.as_deref(), "prs.list")?;
                let url = format!(
                    "{}/repositories/{}/{}/pullrequests?pagelen={}",
                    bitbucket_base(ctx.profile()),
                    enc(workspace),
                    enc(repo),
                    args.limit
                );
                client
                    .request(
                        "bitbucket",
                        "prs.list",
                        ctx.profile(),
                        Method::GET,
                        url,
                        None,
                    )
                    .await
            }
            BitbucketPullRequestAction::Get(args) => {
                let (workspace, repo) =
                    bitbucket_repo(ctx.profile(), args.repo.as_deref(), "prs.get")?;
                let url = format!(
                    "{}/repositories/{}/{}/pullrequests/{}",
                    bitbucket_base(ctx.profile()),
                    enc(workspace),
                    enc(repo),
                    args.number
                );
                client
                    .request(
                        "bitbucket",
                        "prs.get",
                        ctx.profile(),
                        Method::GET,
                        url,
                        None,
                    )
                    .await
            }
            BitbucketPullRequestAction::Create(args) => {
                let repo_arg = args.repo.clone();
                let (workspace, repo) =
                    bitbucket_repo(ctx.profile(), repo_arg.as_deref(), "prs.create")?;
                let body = pr_create_body(args)?;
                let url = format!(
                    "{}/repositories/{}/{}/pullrequests",
                    bitbucket_base(ctx.profile()),
                    enc(workspace),
                    enc(repo)
                );
                client
                    .request(
                        "bitbucket",
                        "prs.create",
                        ctx.profile(),
                        Method::POST,
                        url,
                        Some(body),
                    )
                    .await
            }
            BitbucketPullRequestAction::Delete(args)
            | BitbucketPullRequestAction::Close(args)
            | BitbucketPullRequestAction::Decline(args) => {
                let (workspace, repo) =
                    bitbucket_repo(ctx.profile(), args.repo.as_deref(), "prs.delete")?;
                let url = format!(
                    "{}/repositories/{}/{}/pullrequests/{}/decline",
                    bitbucket_base(ctx.profile()),
                    enc(workspace),
                    enc(repo),
                    args.number
                );
                client
                    .request(
                        "bitbucket",
                        "prs.delete",
                        ctx.profile(),
                        Method::POST,
                        url,
                        None,
                    )
                    .await
            }
            BitbucketPullRequestAction::Comments(command) => {
                pr_comments(client, ctx, command).await
            }
        },
        BitbucketResource::Pipelines(command) => pipelines(client, ctx, command).await,
    }
}

async fn pipelines(
    client: &ApiClient,
    ctx: &Context,
    command: BitbucketPipelinesCommand,
) -> Result<Value, AppError> {
    match command.action {
        BitbucketPipelinesAction::List(args) => {
            let (workspace, repo) = bitbucket_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "pipelines.list",
            )?;
            let mut url = format!(
                "{}/repositories/{}/{}/pipelines?pagelen={}",
                bitbucket_base(ctx.profile()),
                enc(workspace),
                enc(repo),
                args.limit.clamp(1, 100)
            );
            append_query(&mut url, "target.branch", args.branch.as_deref());
            append_query(&mut url, "status", args.status.as_deref());
            append_query(&mut url, "sort", args.sort.as_deref());
            client
                .request(
                    "bitbucket",
                    "pipelines.list",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
        }
        BitbucketPipelinesAction::Get(args) => {
            let (workspace, repo) = bitbucket_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "pipelines.get",
            )?;
            let url = format!(
                "{}/repositories/{}/{}/pipelines/{}",
                bitbucket_base(ctx.profile()),
                enc(workspace),
                enc(repo),
                enc(&args.pipeline)
            );
            client
                .request(
                    "bitbucket",
                    "pipelines.get",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
        }
        BitbucketPipelinesAction::Steps(command) => pipeline_steps(client, ctx, command).await,
    }
}

async fn pipeline_steps(
    client: &ApiClient,
    ctx: &Context,
    command: BitbucketPipelineStepsCommand,
) -> Result<Value, AppError> {
    match command.action {
        BitbucketPipelineStepsAction::List(args) => {
            let (workspace, repo) = bitbucket_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "pipelines.steps.list",
            )?;
            let url = format!(
                "{}/repositories/{}/{}/pipelines/{}/steps",
                bitbucket_base(ctx.profile()),
                enc(workspace),
                enc(repo),
                enc(&args.pipeline)
            );
            client
                .request(
                    "bitbucket",
                    "pipelines.steps.list",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
        }
        BitbucketPipelineStepsAction::Get(args) => {
            let (workspace, repo) = bitbucket_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "pipelines.steps.get",
            )?;
            let url = format!(
                "{}/repositories/{}/{}/pipelines/{}/steps/{}",
                bitbucket_base(ctx.profile()),
                enc(workspace),
                enc(repo),
                enc(&args.pipeline),
                enc(&args.step)
            );
            client
                .request(
                    "bitbucket",
                    "pipelines.steps.get",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
        }
        BitbucketPipelineStepsAction::Logs(command) => match command.action {
            BitbucketPipelineStepLogsAction::Download(args) => {
                let (workspace, repo) = bitbucket_repo_from_args(
                    ctx,
                    args.owner.as_deref(),
                    args.repo.as_deref(),
                    "pipelines.steps.logs.download",
                )?;
                let suffix = if let Some(log) = args.log.as_deref() {
                    format!("/logs/{}", enc(log))
                } else {
                    "/log".to_string()
                };
                let url = format!(
                    "{}/repositories/{}/{}/pipelines/{}/steps/{}{}",
                    bitbucket_base(ctx.profile()),
                    enc(workspace),
                    enc(repo),
                    enc(&args.pipeline),
                    enc(&args.step),
                    suffix
                );
                let bytes = client
                    .download(
                        "bitbucket",
                        "pipelines.steps.logs.download",
                        ctx.profile(),
                        url,
                    )
                    .await?;
                write_download(
                    "bitbucket",
                    "pipelines.steps.logs.download",
                    &args.output,
                    &bytes,
                )
            }
        },
    }
}

fn bitbucket_repo_from_args<'a>(
    ctx: &'a Context,
    owner: Option<&'a str>,
    repo: Option<&'a str>,
    operation: &'static str,
) -> Result<(&'a str, &'a str), AppError> {
    if let (Some(owner), Some(repo)) = (owner, repo) {
        if !owner.is_empty() && !repo.is_empty() {
            return Ok((owner, repo));
        }
    }
    bitbucket_repo(ctx.profile(), repo, operation)
}

fn append_query(url: &mut String, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        url.push('&');
        url.push_str(key);
        url.push('=');
        url.push_str(&enc(value));
    }
}

async fn pr_comments(
    client: &ApiClient,
    ctx: &Context,
    command: BitbucketPrCommentsCommand,
) -> Result<Value, AppError> {
    match command.action {
        BitbucketPrCommentAction::List(args) => {
            let (workspace, repo) =
                bitbucket_repo(ctx.profile(), args.repo.as_deref(), "pr-comments.list")?;
            let url = format!(
                "{}/repositories/{}/{}/pullrequests/{}/comments?pagelen={}",
                bitbucket_base(ctx.profile()),
                enc(workspace),
                enc(repo),
                args.pr,
                args.limit
            );
            client
                .request(
                    "bitbucket",
                    "pr-comments.list",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
        }
        BitbucketPrCommentAction::Get(args) => {
            let (workspace, repo) =
                bitbucket_repo(ctx.profile(), args.repo.as_deref(), "pr-comments.get")?;
            let url = format!(
                "{}/repositories/{}/{}/pullrequests/{}/comments/{}",
                bitbucket_base(ctx.profile()),
                enc(workspace),
                enc(repo),
                args.pr,
                args.comment
            );
            client
                .request(
                    "bitbucket",
                    "pr-comments.get",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
        }
        BitbucketPrCommentAction::Create(args) => {
            let pr = args.pr;
            let repo_arg = args.repo.clone();
            let (workspace, repo) =
                bitbucket_repo(ctx.profile(), repo_arg.as_deref(), "pr-comments.create")?;
            let body = pr_comment_body(args, "pr-comments.create")?;
            let url = format!(
                "{}/repositories/{}/{}/pullrequests/{}/comments",
                bitbucket_base(ctx.profile()),
                enc(workspace),
                enc(repo),
                pr
            );
            client
                .request(
                    "bitbucket",
                    "pr-comments.create",
                    ctx.profile(),
                    Method::POST,
                    url,
                    Some(body),
                )
                .await
        }
        BitbucketPrCommentAction::Update(args) => {
            let pr = args.pr;
            let comment = args.comment.ok_or_else(|| {
                AppError::invalid_input("bitbucket", "pr-comments.update", "--comment is required")
            })?;
            let repo_arg = args.repo.clone();
            let (workspace, repo) =
                bitbucket_repo(ctx.profile(), repo_arg.as_deref(), "pr-comments.update")?;
            let body = pr_comment_body(args, "pr-comments.update")?;
            let url = format!(
                "{}/repositories/{}/{}/pullrequests/{}/comments/{}",
                bitbucket_base(ctx.profile()),
                enc(workspace),
                enc(repo),
                pr,
                comment
            );
            client
                .request(
                    "bitbucket",
                    "pr-comments.update",
                    ctx.profile(),
                    Method::PUT,
                    url,
                    Some(body),
                )
                .await
        }
        BitbucketPrCommentAction::Delete(args) => {
            let (workspace, repo) =
                bitbucket_repo(ctx.profile(), args.repo.as_deref(), "pr-comments.delete")?;
            let url = format!(
                "{}/repositories/{}/{}/pullrequests/{}/comments/{}",
                bitbucket_base(ctx.profile()),
                enc(workspace),
                enc(repo),
                args.pr,
                args.comment
            );
            client
                .request(
                    "bitbucket",
                    "pr-comments.delete",
                    ctx.profile(),
                    Method::DELETE,
                    url,
                    None,
                )
                .await
        }
    }
}

fn pr_create_body(args: PullRequestCreate) -> Result<Value, AppError> {
    let mut body = input::read_json_arg("bitbucket", "prs.create", args.json.as_deref())?;
    input::set_string(&mut body, "title", &args.title);
    input::set_string(&mut body, "description", &args.body);
    if let Some(source) = args.source {
        input::ensure_object(&mut body).insert(
            "source".to_string(),
            json!({ "branch": { "name": source } }),
        );
    }
    if let Some(destination) = args.destination {
        input::ensure_object(&mut body).insert(
            "destination".to_string(),
            json!({ "branch": { "name": destination } }),
        );
    }
    Ok(body)
}

fn pr_comment_body(
    args: BitbucketPrCommentWrite,
    operation: &'static str,
) -> Result<Value, AppError> {
    let mut body = input::read_json_arg("bitbucket", operation, args.json.as_deref())?;
    if let Some(raw) = args.body {
        input::ensure_object(&mut body).insert("content".to_string(), json!({ "raw": raw }));
    }
    if body.get("content").is_none() {
        return Err(AppError::invalid_input(
            "bitbucket",
            operation,
            "--body or JSON content is required",
        ));
    }
    Ok(body)
}
