use reqwest::Method;
use serde_json::{json, Value};

use crate::{
    cli::*,
    config::Context,
    error::AppError,
    http::ApiClient,
    input,
    services::{
        generic_request,
        shared::{bitbucket_base, bitbucket_repo, enc, workspace, write_download, CtxProfile},
    },
};

pub(crate) async fn dispatch(
    client: &ApiClient,
    ctx: &Context,
    command: BitbucketCommand,
) -> Result<Value, AppError> {
    match command.resource {
        BitbucketResource::Request(args) => {
            generic_request::dispatch(
                client,
                ctx,
                "bitbucket",
                bitbucket_base(ctx.profile()),
                args,
            )
            .await
        }
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
    if let Some(owner) = owner {
        if owner.is_empty() {
            return Err(AppError::invalid_input(
                "bitbucket",
                operation,
                "--owner must not be empty",
            ));
        }

        if repo.is_some_and(str::is_empty) {
            return Err(AppError::invalid_input(
                "bitbucket",
                operation,
                "--repo must not be empty",
            ));
        }

        if let Some(repo) = repo {
            if repo.contains('/') {
                return Err(AppError::invalid_input(
                    "bitbucket",
                    operation,
                    "--owner cannot be combined with a workspace/repo value for --repo; pass a repo slug",
                ));
            }
            return Ok((owner, repo));
        }

        let profile_repo = ctx.profile().repo.as_deref().ok_or_else(|| {
            AppError::service_config(
                "bitbucket",
                operation,
                format!("bitbucket.{operation} requires --repo or profile.repo"),
            )
        })?;
        if profile_repo.is_empty() {
            return Err(AppError::invalid_input(
                "bitbucket",
                operation,
                "profile.repo must not be empty",
            ));
        }
        let repo_name = profile_repo
            .split_once('/')
            .map(|(_, repo_name)| repo_name)
            .unwrap_or(profile_repo);
        if repo_name.is_empty() {
            return Err(AppError::invalid_input(
                "bitbucket",
                operation,
                "profile.repo must include a repo slug",
            ));
        }
        return Ok((owner, repo_name));
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

const BITBUCKET_PAGELEN_MAX: u32 = 100;

fn pagelen_for(limit: u32) -> u32 {
    limit.clamp(1, BITBUCKET_PAGELEN_MAX)
}

fn append_pagelen(url: &mut String, limit: u32) {
    let separator = if url.contains('?') { '&' } else { '?' };
    url.push(separator);
    url.push_str("pagelen=");
    url.push_str(&pagelen_for(limit).to_string());
}

fn aggregate_values(values: Vec<Value>, truncated: bool) -> Value {
    let size = values.len();
    json!({
        "values": values,
        "size": size,
        "truncated": truncated,
    })
}

async fn paginate_bitbucket<F>(
    client: &ApiClient,
    operation: &'static str,
    ctx: &Context,
    first_url: String,
    limit: u32,
    filter: F,
) -> Result<Value, AppError>
where
    F: Fn(&Value) -> bool,
{
    let limit = limit as usize;
    let mut url = first_url;
    let mut accumulated: Vec<Value> = Vec::new();
    let mut truncated = false;
    'outer: loop {
        let page = client
            .request(
                "bitbucket",
                operation,
                ctx.profile(),
                Method::GET,
                url.clone(),
                None,
            )
            .await?;
        let Some(values) = page.get("values").and_then(Value::as_array) else {
            return Ok(page);
        };
        for value in values {
            if !filter(value) {
                continue;
            }
            accumulated.push(value.clone());
            if accumulated.len() >= limit {
                truncated = true;
                break 'outer;
            }
        }
        match page.get("next").and_then(Value::as_str) {
            Some(next) if !next.is_empty() => url = next.to_string(),
            _ => break 'outer,
        }
    }
    Ok(aggregate_values(accumulated, truncated))
}

fn keep_all(_value: &Value) -> bool {
    true
}

fn keep_inline_only(value: &Value) -> bool {
    value.get("inline").is_some()
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
    let mut url = format!(
        "{}/repositories/{}/{}/pullrequests/{}/diffstat",
        bitbucket_base(ctx.profile()),
        enc(workspace),
        enc(repo),
        args.pr
    );
    append_pagelen(&mut url, args.limit);
    paginate_bitbucket(client, "prs.diffstat", ctx, url, args.limit, keep_all).await
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
    let mut url = format!(
        "{}/repositories/{}/{}/pullrequests/{}/commits",
        bitbucket_base(ctx.profile()),
        enc(workspace),
        enc(repo),
        args.pr
    );
    append_pagelen(&mut url, args.limit);
    paginate_bitbucket(client, "prs.commits", ctx, url, args.limit, keep_all).await
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
    let mut url = format!(
        "{}/repositories/{}/{}/pullrequests/{}/activity",
        bitbucket_base(ctx.profile()),
        enc(workspace),
        enc(repo),
        args.pr
    );
    append_pagelen(&mut url, args.limit);
    paginate_bitbucket(client, "prs.activity", ctx, url, args.limit, keep_all).await
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
                "{}/repositories/{}/{}/refs/branches",
                bitbucket_base(ctx.profile()),
                enc(workspace),
                enc(repo),
            );
            append_pagelen(&mut url, args.limit);
            let bbql = branches_bbql_from_flags(&args)?;
            append_query(&mut url, "q", bbql.as_deref());
            let prefix = args.name_prefix.clone();
            let filter = move |value: &Value| -> bool {
                let Some(prefix) = prefix.as_deref() else {
                    return true;
                };
                value
                    .get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|name| name.starts_with(prefix))
            };
            paginate_bitbucket(client, "branches.list", ctx, url, args.limit, filter).await
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
                "{}/repositories/{}/{}/commits",
                bitbucket_base(ctx.profile()),
                enc(workspace),
                enc(repo),
            );
            append_pagelen(&mut url, args.limit);
            if let Some(branch) = args.branch.as_deref() {
                append_query(&mut url, "include", Some(branch));
            } else {
                append_query(&mut url, "include", args.include.as_deref());
            }
            append_query(&mut url, "exclude", args.exclude.as_deref());
            paginate_bitbucket(client, "commits.list", ctx, url, args.limit, keep_all).await
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
            let mut url = format!(
                "{}/repositories/{}/{}/filehistory/{}/{}",
                bitbucket_base(ctx.profile()),
                enc(workspace),
                enc(repo),
                enc(&args.commit),
                path,
            );
            append_pagelen(&mut url, args.limit);
            paginate_bitbucket(client, "source.history", ctx, url, args.limit, keep_all).await
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
            let (workspace, repo) = bitbucket_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "pr-comments.list",
            )?;
            let mut url = format!(
                "{}/repositories/{}/{}/pullrequests/{}/comments",
                bitbucket_base(ctx.profile()),
                enc(workspace),
                enc(repo),
                args.pr,
            );
            append_pagelen(&mut url, args.limit);
            if args.inline_only {
                paginate_bitbucket(
                    client,
                    "pr-comments.list",
                    ctx,
                    url,
                    args.limit,
                    keep_inline_only,
                )
                .await
            } else {
                paginate_bitbucket(client, "pr-comments.list", ctx, url, args.limit, keep_all).await
            }
        }
        BitbucketPrCommentAction::Get(args) => {
            let (workspace, repo) = bitbucket_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "pr-comments.get",
            )?;
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
            let owner_arg = args.owner.clone();
            let repo_arg = args.repo.clone();
            let (workspace, repo) = bitbucket_repo_from_args(
                ctx,
                owner_arg.as_deref(),
                repo_arg.as_deref(),
                "pr-comments.create",
            )?;
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
            let owner_arg = args.owner.clone();
            let repo_arg = args.repo.clone();
            let (workspace, repo) = bitbucket_repo_from_args(
                ctx,
                owner_arg.as_deref(),
                repo_arg.as_deref(),
                "pr-comments.update",
            )?;
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
            let (workspace, repo) = bitbucket_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "pr-comments.delete",
            )?;
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

fn branches_bbql_from_flags(args: &BitbucketBranchList) -> Result<Option<String>, AppError> {
    if let Some(query) = args.query.as_deref() {
        return Ok(Some(query.to_string()));
    }
    let escape = |text: &str| text.replace('\\', "\\\\").replace('"', "\\\"");
    // Bitbucket BBQL's `~` is a case-insensitive substring match (no regex anchors).
    // Both --name-contains and --name-prefix use it as a server-side hint;
    // --name-prefix is then narrowed client-side via the paginator filter.
    if let Some(text) = args.name_contains.as_deref() {
        if text.is_empty() {
            return Err(AppError::invalid_input(
                "bitbucket",
                "branches.list",
                "--name-contains must not be empty",
            ));
        }
        return Ok(Some(format!("name ~ \"{}\"", escape(text))));
    }
    if let Some(text) = args.name_prefix.as_deref() {
        if text.is_empty() {
            return Err(AppError::invalid_input(
                "bitbucket",
                "branches.list",
                "--name-prefix must not be empty",
            ));
        }
        return Ok(Some(format!("name ~ \"{}\"", escape(text))));
    }
    Ok(None)
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

    fn branch_list_args(
        name_contains: Option<&str>,
        name_prefix: Option<&str>,
        query: Option<&str>,
    ) -> BitbucketBranchList {
        BitbucketBranchList {
            owner: None,
            repo: None,
            limit: 50,
            name_contains: name_contains.map(str::to_string),
            name_prefix: name_prefix.map(str::to_string),
            query: query.map(str::to_string),
        }
    }

    fn bitbucket_test_ctx(workspace: Option<&str>, repo: Option<&str>) -> Context {
        Context {
            profile: crate::config::Profile {
                workspace: workspace.map(str::to_string),
                repo: repo.map(str::to_string),
                ..Default::default()
            },
            secrets_file: Default::default(),
            key_file: Default::default(),
        }
    }

    #[test]
    fn bitbucket_repo_from_args_owner_overrides_profile_workspace() {
        let ctx = bitbucket_test_ctx(Some("profile-workspace"), Some("profile-repo"));
        let resolved =
            bitbucket_repo_from_args(&ctx, Some("override-workspace"), None, "test").unwrap();
        assert_eq!(resolved, ("override-workspace", "profile-repo"));
    }

    #[test]
    fn bitbucket_repo_from_args_owner_uses_repo_slug_arg() {
        let ctx = bitbucket_test_ctx(Some("profile-workspace"), Some("profile-repo"));
        let resolved =
            bitbucket_repo_from_args(&ctx, Some("override-workspace"), Some("arg-repo"), "test")
                .unwrap();
        assert_eq!(resolved, ("override-workspace", "arg-repo"));
    }

    #[test]
    fn bitbucket_repo_from_args_owner_strips_profile_repo_workspace() {
        let ctx = bitbucket_test_ctx(
            Some("profile-workspace"),
            Some("stored-workspace/stored-repo"),
        );
        let resolved =
            bitbucket_repo_from_args(&ctx, Some("override-workspace"), None, "test").unwrap();
        assert_eq!(resolved, ("override-workspace", "stored-repo"));
    }

    #[test]
    fn bitbucket_repo_from_args_preserves_workspace_repo_arg_without_owner() {
        let ctx = bitbucket_test_ctx(Some("profile-workspace"), Some("profile-repo"));
        let resolved =
            bitbucket_repo_from_args(&ctx, None, Some("arg-workspace/arg-repo"), "test").unwrap();
        assert_eq!(resolved, ("arg-workspace", "arg-repo"));
    }

    #[test]
    fn bitbucket_repo_from_args_rejects_owner_with_workspace_repo_arg() {
        let ctx = bitbucket_test_ctx(Some("profile-workspace"), Some("profile-repo"));
        let err = bitbucket_repo_from_args(
            &ctx,
            Some("override-workspace"),
            Some("arg-workspace/arg-repo"),
            "test",
        )
        .unwrap_err();
        assert_eq!(err.code, "invalid_input");
    }

    #[test]
    fn name_contains_builds_bbql() {
        let bbql = branches_bbql_from_flags(&branch_list_args(Some("feature/"), None, None))
            .unwrap()
            .unwrap();
        assert_eq!(bbql, "name ~ \"feature/\"");
    }

    #[test]
    fn name_prefix_uses_substring_hint_for_server_filter() {
        // BBQL `~` is case-insensitive substring; the actual prefix match is
        // applied client-side by the paginator filter in `branches::List`.
        let bbql = branches_bbql_from_flags(&branch_list_args(None, Some("release-"), None))
            .unwrap()
            .unwrap();
        assert_eq!(bbql, "name ~ \"release-\"");
    }

    #[test]
    fn name_contains_escapes_quotes_and_backslashes() {
        let bbql = branches_bbql_from_flags(&branch_list_args(Some(r#"feat\"weird"#), None, None))
            .unwrap()
            .unwrap();
        assert_eq!(bbql, "name ~ \"feat\\\\\\\"weird\"");
    }

    #[test]
    fn query_takes_precedence_when_set() {
        let bbql =
            branches_bbql_from_flags(&branch_list_args(None, None, Some(r#"name ~ "exact""#)))
                .unwrap()
                .unwrap();
        assert_eq!(bbql, r#"name ~ "exact""#);
    }

    #[test]
    fn empty_name_contains_is_rejected() {
        let err = branches_bbql_from_flags(&branch_list_args(Some(""), None, None)).unwrap_err();
        assert_eq!(err.code, "invalid_input");
    }

    #[test]
    fn pagelen_for_clamps_to_max() {
        assert_eq!(pagelen_for(0), 1);
        assert_eq!(pagelen_for(50), 50);
        assert_eq!(pagelen_for(500), BITBUCKET_PAGELEN_MAX);
    }

    #[test]
    fn append_pagelen_uses_question_or_amp() {
        let mut a = "https://api.example.com/path".to_string();
        append_pagelen(&mut a, 25);
        assert!(a.ends_with("?pagelen=25"));

        let mut b = "https://api.example.com/path?q=foo".to_string();
        append_pagelen(&mut b, 25);
        assert!(b.ends_with("&pagelen=25"));
    }

    #[test]
    fn keep_inline_only_filters_correctly() {
        let inline = json!({ "id": 1, "inline": { "path": "src/lib.rs", "to": 10 } });
        let regular = json!({ "id": 2, "content": { "raw": "hi" } });
        assert!(keep_inline_only(&inline));
        assert!(!keep_inline_only(&regular));
    }

    #[test]
    fn aggregate_values_shape_matches_contract() {
        let collected = vec![json!({"id": 1}), json!({"id": 2})];
        let out = aggregate_values(collected.clone(), true);
        assert_eq!(out["values"].as_array().unwrap().len(), 2);
        assert_eq!(out["size"], 2);
        assert_eq!(out["truncated"], true);
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
