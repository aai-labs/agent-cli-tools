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
            BitbucketPullRequestAction::Diff(args) => pr_diff(client, ctx, args).await,
            BitbucketPullRequestAction::Diffstat(args) => pr_diffstat(client, ctx, args).await,
            BitbucketPullRequestAction::Commits(args) => pr_commits(client, ctx, args).await,
            BitbucketPullRequestAction::Activity(args) => pr_activity(client, ctx, args).await,
            BitbucketPullRequestAction::Comments(command) => {
                pr_comments(client, ctx, command).await
            }
        },
        BitbucketResource::Branches(command) => branches(client, ctx, command).await,
        BitbucketResource::Commits(command) => commits(client, ctx, command).await,
        BitbucketResource::Source(command) => source(client, ctx, command).await,
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

fn enc_path(path: &str) -> String {
    path.trim_start_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(enc)
        .collect::<Vec<_>>()
        .join("/")
}

async fn pr_diff(
    client: &ApiClient,
    ctx: &Context,
    args: BitbucketPrDiff,
) -> Result<Value, AppError> {
    let (workspace, repo) =
        bitbucket_repo_from_args(ctx, args.owner.as_deref(), args.repo.as_deref(), "prs.diff")?;
    let url = format!(
        "{}/repositories/{}/{}/pullrequests/{}/diff",
        bitbucket_base(ctx.profile()),
        enc(workspace),
        enc(repo),
        args.pr
    );
    if let Some(output) = args.output {
        let bytes = client
            .download("bitbucket", "prs.diff", ctx.profile(), url)
            .await?;
        return write_download("bitbucket", "prs.diff", &output, &bytes);
    }
    client
        .request(
            "bitbucket",
            "prs.diff",
            ctx.profile(),
            Method::GET,
            url,
            None,
        )
        .await
}

async fn pr_diffstat(
    client: &ApiClient,
    ctx: &Context,
    args: BitbucketPrDiffstat,
) -> Result<Value, AppError> {
    let (workspace, repo) = bitbucket_repo_from_args(
        ctx,
        args.owner.as_deref(),
        args.repo.as_deref(),
        "prs.diffstat",
    )?;
    let url = format!(
        "{}/repositories/{}/{}/pullrequests/{}/diffstat?pagelen={}",
        bitbucket_base(ctx.profile()),
        enc(workspace),
        enc(repo),
        args.pr,
        args.limit
    );
    client
        .request(
            "bitbucket",
            "prs.diffstat",
            ctx.profile(),
            Method::GET,
            url,
            None,
        )
        .await
}

async fn pr_commits(
    client: &ApiClient,
    ctx: &Context,
    args: BitbucketPrCommits,
) -> Result<Value, AppError> {
    let (workspace, repo) = bitbucket_repo_from_args(
        ctx,
        args.owner.as_deref(),
        args.repo.as_deref(),
        "prs.commits",
    )?;
    let url = format!(
        "{}/repositories/{}/{}/pullrequests/{}/commits?pagelen={}",
        bitbucket_base(ctx.profile()),
        enc(workspace),
        enc(repo),
        args.pr,
        args.limit
    );
    client
        .request(
            "bitbucket",
            "prs.commits",
            ctx.profile(),
            Method::GET,
            url,
            None,
        )
        .await
}

async fn pr_activity(
    client: &ApiClient,
    ctx: &Context,
    args: BitbucketPrActivity,
) -> Result<Value, AppError> {
    let (workspace, repo) = bitbucket_repo_from_args(
        ctx,
        args.owner.as_deref(),
        args.repo.as_deref(),
        "prs.activity",
    )?;
    let url = format!(
        "{}/repositories/{}/{}/pullrequests/{}/activity?pagelen={}",
        bitbucket_base(ctx.profile()),
        enc(workspace),
        enc(repo),
        args.pr,
        args.limit
    );
    client
        .request(
            "bitbucket",
            "prs.activity",
            ctx.profile(),
            Method::GET,
            url,
            None,
        )
        .await
}

async fn branches(
    client: &ApiClient,
    ctx: &Context,
    command: BitbucketBranchesCommand,
) -> Result<Value, AppError> {
    match command.action {
        BitbucketBranchesAction::List(args) => {
            let (workspace, repo) = bitbucket_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "branches.list",
            )?;
            let mut url = format!(
                "{}/repositories/{}/{}/refs/branches?pagelen={}",
                bitbucket_base(ctx.profile()),
                enc(workspace),
                enc(repo),
                args.limit
            );
            append_query(&mut url, "q", args.query.as_deref());
            client
                .request(
                    "bitbucket",
                    "branches.list",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
        }
        BitbucketBranchesAction::Get(args) => {
            let (workspace, repo) = bitbucket_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "branches.get",
            )?;
            let url = format!(
                "{}/repositories/{}/{}/refs/branches/{}",
                bitbucket_base(ctx.profile()),
                enc(workspace),
                enc(repo),
                enc(&args.name)
            );
            client
                .request(
                    "bitbucket",
                    "branches.get",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
        }
    }
}

async fn commits(
    client: &ApiClient,
    ctx: &Context,
    command: BitbucketCommitsCommand,
) -> Result<Value, AppError> {
    match command.action {
        BitbucketCommitsAction::List(args) => {
            let (workspace, repo) = bitbucket_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "commits.list",
            )?;
            let mut url = format!(
                "{}/repositories/{}/{}/commits?pagelen={}",
                bitbucket_base(ctx.profile()),
                enc(workspace),
                enc(repo),
                args.limit
            );
            if let Some(branch) = args.branch.as_deref() {
                append_query(&mut url, "include", Some(branch));
            } else {
                append_query(&mut url, "include", args.include.as_deref());
            }
            append_query(&mut url, "exclude", args.exclude.as_deref());
            client
                .request(
                    "bitbucket",
                    "commits.list",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
        }
        BitbucketCommitsAction::Get(args) => {
            let (workspace, repo) = bitbucket_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "commits.get",
            )?;
            let url = format!(
                "{}/repositories/{}/{}/commit/{}",
                bitbucket_base(ctx.profile()),
                enc(workspace),
                enc(repo),
                enc(&args.sha)
            );
            client
                .request(
                    "bitbucket",
                    "commits.get",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
        }
    }
}

async fn source(
    client: &ApiClient,
    ctx: &Context,
    command: BitbucketSourceCommand,
) -> Result<Value, AppError> {
    match command.action {
        BitbucketSourceAction::Get(args) => {
            let (workspace, repo) = bitbucket_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "source.get",
            )?;
            let path = enc_path(&args.path);
            let mut url = format!(
                "{}/repositories/{}/{}/src/{}/{}",
                bitbucket_base(ctx.profile()),
                enc(workspace),
                enc(repo),
                enc(&args.commit),
                path
            );
            if args.meta {
                url.push_str("?format=meta");
                return client
                    .request(
                        "bitbucket",
                        "source.get",
                        ctx.profile(),
                        Method::GET,
                        url,
                        None,
                    )
                    .await;
            }
            if let Some(output) = args.output {
                let bytes = client
                    .download("bitbucket", "source.get", ctx.profile(), url)
                    .await?;
                return write_download("bitbucket", "source.get", &output, &bytes);
            }
            client
                .request(
                    "bitbucket",
                    "source.get",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
        }
        BitbucketSourceAction::History(args) => {
            let (workspace, repo) = bitbucket_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "source.history",
            )?;
            let path = enc_path(&args.path);
            let url = format!(
                "{}/repositories/{}/{}/filehistory/{}/{}?pagelen={}",
                bitbucket_base(ctx.profile()),
                enc(workspace),
                enc(repo),
                enc(&args.commit),
                path,
                args.limit
            );
            client
                .request(
                    "bitbucket",
                    "source.history",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
        }
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
            let response = client
                .request(
                    "bitbucket",
                    "pr-comments.list",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await?;
            if args.inline_only {
                return Ok(filter_inline_comments(response));
            }
            Ok(response)
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

fn filter_inline_comments(mut response: Value) -> Value {
    let Some(values) = response.get_mut("values").and_then(Value::as_array_mut) else {
        return response;
    };
    values.retain(|comment| comment.get("inline").is_some());
    response
}

fn pr_comment_body(
    args: BitbucketPrCommentWrite,
    operation: &'static str,
) -> Result<Value, AppError> {
    if (args.inline_from.is_some() || args.inline_to.is_some()) && args.inline_path.is_none() {
        return Err(AppError::invalid_input(
            "bitbucket",
            operation,
            "--inline-path is required when --inline-from or --inline-to is set",
        ));
    }

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

    if let Some(path) = args.inline_path {
        let mut inline = json!({ "path": path });
        if let Some(from) = args.inline_from {
            inline["from"] = json!(from);
        }
        if let Some(to) = args.inline_to {
            inline["to"] = json!(to);
        }
        input::ensure_object(&mut body).insert("inline".to_string(), inline);
    }

    if let Some(parent_id) = args.parent_id {
        input::ensure_object(&mut body).insert("parent".to_string(), json!({ "id": parent_id }));
    }

    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_args(
        body: Option<&str>,
        inline_path: Option<&str>,
        inline_from: Option<u64>,
        inline_to: Option<u64>,
        parent_id: Option<u64>,
        json: Option<&str>,
    ) -> BitbucketPrCommentWrite {
        BitbucketPrCommentWrite {
            pr: 1,
            comment: None,
            owner: None,
            repo: None,
            json: json.map(str::to_string),
            body: body.map(str::to_string),
            inline_path: inline_path.map(str::to_string),
            inline_from,
            inline_to,
            parent_id,
        }
    }

    #[test]
    fn inline_comment_body_attaches_inline_object() {
        let body = pr_comment_body(
            write_args(
                Some("looks good"),
                Some("src/main.rs"),
                None,
                Some(42),
                Some(7),
                None,
            ),
            "pr-comments.create",
        )
        .unwrap();
        assert_eq!(body["content"]["raw"], "looks good");
        assert_eq!(body["inline"]["path"], "src/main.rs");
        assert_eq!(body["inline"]["to"], 42);
        assert_eq!(body["parent"]["id"], 7);
    }

    #[test]
    fn inline_comment_requires_path_when_lines_provided() {
        let err = pr_comment_body(
            write_args(Some("missing path"), None, Some(1), None, None, None),
            "pr-comments.create",
        )
        .unwrap_err();
        assert_eq!(err.code, "invalid_input");
    }

    #[test]
    fn json_payload_merges_with_inline_flags() {
        let body = pr_comment_body(
            write_args(
                None,
                Some("README.md"),
                None,
                Some(10),
                None,
                Some(r#"{"content":{"raw":"from json"}}"#),
            ),
            "pr-comments.create",
        )
        .unwrap();
        assert_eq!(body["content"]["raw"], "from json");
        assert_eq!(body["inline"]["path"], "README.md");
        assert_eq!(body["inline"]["to"], 10);
    }
}
