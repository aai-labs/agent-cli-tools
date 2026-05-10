use reqwest::Method;
use serde_json::{json, Value};

use crate::{
    cli::*,
    config::Context,
    error::AppError,
    http::ApiClient,
    input,
    services::shared::{bitbucket_base, bitbucket_repo, enc, workspace, CtxProfile},
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
