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
        shared::{enc, github_base, github_repo, write_download, CtxProfile},
    },
};

const GITHUB_PER_PAGE_MAX: u32 = 100;
const GITHUB_DIFF_ACCEPT: &str = "application/vnd.github.v3.diff";
const GITHUB_RAW_ACCEPT: &str = "application/vnd.github.v3.raw";

pub(crate) async fn dispatch(
    client: &ApiClient,
    ctx: &Context,
    command: GithubCommand,
) -> Result<Value, AppError> {
    match command.resource {
        GithubResource::Request(args) => {
            generic_request::dispatch(client, ctx, "github", github_base(ctx.profile()), args).await
        }
        GithubResource::Repos(command) => match command.action {
            GithubReposAction::List(args) => {
                let url = if let Some(org) = ctx.profile().org.as_deref() {
                    format!(
                        "{}/orgs/{}/repos?per_page={}",
                        github_base(ctx.profile()),
                        enc(org),
                        args.limit
                    )
                } else {
                    format!(
                        "{}/user/repos?per_page={}",
                        github_base(ctx.profile()),
                        args.limit
                    )
                };
                client
                    .request(
                        "github",
                        "repos.list",
                        ctx.profile(),
                        Method::GET,
                        url,
                        None,
                    )
                    .await
            }
            GithubReposAction::Get(args) => {
                let (owner, repo) = github_repo_from_args(
                    ctx,
                    args.owner.as_deref(),
                    args.repo.as_deref(),
                    "repos.get",
                )?;
                let url = format!(
                    "{}/repos/{}/{}",
                    github_base(ctx.profile()),
                    enc(owner),
                    enc(repo)
                );
                client
                    .request("github", "repos.get", ctx.profile(), Method::GET, url, None)
                    .await
            }
        },
        GithubResource::Issues(command) => match command.action {
            GithubIssuesAction::List(args) => {
                let (owner, repo) = github_repo_from_args(
                    ctx,
                    args.owner.as_deref(),
                    args.repo.as_deref(),
                    "issues.list",
                )?;
                let url = format!(
                    "{}/repos/{}/{}/issues?per_page={}",
                    github_base(ctx.profile()),
                    enc(owner),
                    enc(repo),
                    args.limit
                );
                client
                    .request(
                        "github",
                        "issues.list",
                        ctx.profile(),
                        Method::GET,
                        url,
                        None,
                    )
                    .await
            }
            GithubIssuesAction::Get(args) => {
                let (owner, repo) = github_repo_from_args(
                    ctx,
                    args.owner.as_deref(),
                    args.repo.as_deref(),
                    "issues.get",
                )?;
                let url = format!(
                    "{}/repos/{}/{}/issues/{}",
                    github_base(ctx.profile()),
                    enc(owner),
                    enc(repo),
                    args.number
                );
                client
                    .request(
                        "github",
                        "issues.get",
                        ctx.profile(),
                        Method::GET,
                        url,
                        None,
                    )
                    .await
            }
            GithubIssuesAction::Create(args) => {
                let owner_arg = args.owner.clone();
                let repo_arg = args.repo.clone();
                let (owner, repo) = github_repo_from_args(
                    ctx,
                    owner_arg.as_deref(),
                    repo_arg.as_deref(),
                    "issues.create",
                )?;
                let body = issue_create_body(args)?;
                let url = format!(
                    "{}/repos/{}/{}/issues",
                    github_base(ctx.profile()),
                    enc(owner),
                    enc(repo)
                );
                client
                    .request(
                        "github",
                        "issues.create",
                        ctx.profile(),
                        Method::POST,
                        url,
                        Some(body),
                    )
                    .await
            }
            GithubIssuesAction::Update(args) => {
                let owner_arg = args.owner.clone();
                let repo_arg = args.repo.clone();
                let (owner, repo) = github_repo_from_args(
                    ctx,
                    owner_arg.as_deref(),
                    repo_arg.as_deref(),
                    "issues.update",
                )?;
                let number = args.number;
                let body = issue_update_body(args)?;
                let url = format!(
                    "{}/repos/{}/{}/issues/{}",
                    github_base(ctx.profile()),
                    enc(owner),
                    enc(repo),
                    number
                );
                client
                    .request(
                        "github",
                        "issues.update",
                        ctx.profile(),
                        Method::PATCH,
                        url,
                        Some(body),
                    )
                    .await
            }
            GithubIssuesAction::Delete(args) => {
                let (owner, repo) = github_repo_from_args(
                    ctx,
                    args.owner.as_deref(),
                    args.repo.as_deref(),
                    "issues.delete",
                )?;
                let url = format!(
                    "{}/repos/{}/{}/issues/{}",
                    github_base(ctx.profile()),
                    enc(owner),
                    enc(repo),
                    args.number
                );
                client
                    .request(
                        "github",
                        "issues.delete",
                        ctx.profile(),
                        Method::PATCH,
                        url,
                        Some(json!({ "state": "closed" })),
                    )
                    .await
            }
        },
        GithubResource::Prs(command) => match command.action {
            GithubPullRequestAction::List(args) => {
                let (owner, repo) = github_repo_from_args(
                    ctx,
                    args.owner.as_deref(),
                    args.repo.as_deref(),
                    "prs.list",
                )?;
                let url = format!(
                    "{}/repos/{}/{}/pulls?per_page={}",
                    github_base(ctx.profile()),
                    enc(owner),
                    enc(repo),
                    args.limit
                );
                client
                    .request("github", "prs.list", ctx.profile(), Method::GET, url, None)
                    .await
            }
            GithubPullRequestAction::Get(args) => {
                let (owner, repo) = github_repo_from_args(
                    ctx,
                    args.owner.as_deref(),
                    args.repo.as_deref(),
                    "prs.get",
                )?;
                let url = format!(
                    "{}/repos/{}/{}/pulls/{}",
                    github_base(ctx.profile()),
                    enc(owner),
                    enc(repo),
                    args.number
                );
                client
                    .request("github", "prs.get", ctx.profile(), Method::GET, url, None)
                    .await
            }
            GithubPullRequestAction::Create(args) => {
                let owner_arg = args.owner.clone();
                let repo_arg = args.repo.clone();
                let (owner, repo) = github_repo_from_args(
                    ctx,
                    owner_arg.as_deref(),
                    repo_arg.as_deref(),
                    "prs.create",
                )?;
                let body = pr_create_body(args)?;
                let url = format!(
                    "{}/repos/{}/{}/pulls",
                    github_base(ctx.profile()),
                    enc(owner),
                    enc(repo)
                );
                client
                    .request(
                        "github",
                        "prs.create",
                        ctx.profile(),
                        Method::POST,
                        url,
                        Some(body),
                    )
                    .await
            }
            GithubPullRequestAction::Delete(args)
            | GithubPullRequestAction::Close(args)
            | GithubPullRequestAction::Decline(args) => {
                let (owner, repo) = github_repo_from_args(
                    ctx,
                    args.owner.as_deref(),
                    args.repo.as_deref(),
                    "prs.delete",
                )?;
                let url = format!(
                    "{}/repos/{}/{}/pulls/{}",
                    github_base(ctx.profile()),
                    enc(owner),
                    enc(repo),
                    args.number
                );
                client
                    .request(
                        "github",
                        "prs.delete",
                        ctx.profile(),
                        Method::PATCH,
                        url,
                        Some(json!({ "state": "closed" })),
                    )
                    .await
            }
            GithubPullRequestAction::Diff(args) => pr_diff(client, ctx, args).await,
            GithubPullRequestAction::Files(args) => pr_files(client, ctx, args).await,
            GithubPullRequestAction::Commits(args) => pr_commits(client, ctx, args).await,
            GithubPullRequestAction::Timeline(args) => pr_timeline(client, ctx, args).await,
            GithubPullRequestAction::Comments(command) => pr_comments(client, ctx, command).await,
            GithubPullRequestAction::ReviewComments(command) => {
                pr_review_comments(client, ctx, command).await
            }
            GithubPullRequestAction::Reviews(command) => pr_reviews(client, ctx, command).await,
        },
        GithubResource::Actions(command) => actions(client, ctx, command).await,
        GithubResource::Branches(command) => branches(client, ctx, command).await,
        GithubResource::Source(command) => source(client, ctx, command).await,
    }
}

async fn actions(
    client: &ApiClient,
    ctx: &Context,
    command: GithubActionsCommand,
) -> Result<Value, AppError> {
    match command.resource {
        GithubActionsResource::Runs(command) => match command.action {
            GithubActionsRunsAction::List(args) => {
                let (owner, repo) = github_repo_from_args(
                    ctx,
                    args.owner.as_deref(),
                    args.repo.as_deref(),
                    "actions.runs.list",
                )?;
                let mut url = format!(
                    "{}/repos/{}/{}/actions/runs?per_page={}",
                    github_base(ctx.profile()),
                    enc(owner),
                    enc(repo),
                    args.limit.clamp(1, 100)
                );
                append_query(&mut url, "branch", args.branch.as_deref());
                append_query(&mut url, "status", args.status.as_deref());
                append_query(&mut url, "event", args.event.as_deref());
                client
                    .request(
                        "github",
                        "actions.runs.list",
                        ctx.profile(),
                        Method::GET,
                        url,
                        None,
                    )
                    .await
            }
            GithubActionsRunsAction::Get(args) => {
                let (owner, repo) = github_repo_from_args(
                    ctx,
                    args.owner.as_deref(),
                    args.repo.as_deref(),
                    "actions.runs.get",
                )?;
                let url = format!(
                    "{}/repos/{}/{}/actions/runs/{}",
                    github_base(ctx.profile()),
                    enc(owner),
                    enc(repo),
                    args.run
                );
                client
                    .request(
                        "github",
                        "actions.runs.get",
                        ctx.profile(),
                        Method::GET,
                        url,
                        None,
                    )
                    .await
            }
            GithubActionsRunsAction::Logs(command) => match command.action {
                GithubActionsRunLogsAction::Download(args) => {
                    let (owner, repo) = github_repo_from_args(
                        ctx,
                        args.owner.as_deref(),
                        args.repo.as_deref(),
                        "actions.runs.logs.download",
                    )?;
                    let url = format!(
                        "{}/repos/{}/{}/actions/runs/{}/logs",
                        github_base(ctx.profile()),
                        enc(owner),
                        enc(repo),
                        args.run
                    );
                    let bytes = client
                        .download("github", "actions.runs.logs.download", ctx.profile(), url)
                        .await?;
                    write_download("github", "actions.runs.logs.download", &args.output, &bytes)
                }
            },
        },
        GithubActionsResource::Jobs(command) => match command.action {
            GithubActionsJobsAction::List(args) => {
                let (owner, repo) = github_repo_from_args(
                    ctx,
                    args.owner.as_deref(),
                    args.repo.as_deref(),
                    "actions.jobs.list",
                )?;
                let filter = if args.all_attempts { "all" } else { "latest" };
                let url = format!(
                    "{}/repos/{}/{}/actions/runs/{}/jobs?per_page={}&filter={}",
                    github_base(ctx.profile()),
                    enc(owner),
                    enc(repo),
                    args.run,
                    args.limit.clamp(1, 100),
                    filter
                );
                client
                    .request(
                        "github",
                        "actions.jobs.list",
                        ctx.profile(),
                        Method::GET,
                        url,
                        None,
                    )
                    .await
            }
            GithubActionsJobsAction::Get(args) => {
                let (owner, repo) = github_repo_from_args(
                    ctx,
                    args.owner.as_deref(),
                    args.repo.as_deref(),
                    "actions.jobs.get",
                )?;
                let url = format!(
                    "{}/repos/{}/{}/actions/jobs/{}",
                    github_base(ctx.profile()),
                    enc(owner),
                    enc(repo),
                    args.job
                );
                client
                    .request(
                        "github",
                        "actions.jobs.get",
                        ctx.profile(),
                        Method::GET,
                        url,
                        None,
                    )
                    .await
            }
            GithubActionsJobsAction::Logs(command) => match command.action {
                GithubActionsJobLogsAction::Download(args) => {
                    let (owner, repo) = github_repo_from_args(
                        ctx,
                        args.owner.as_deref(),
                        args.repo.as_deref(),
                        "actions.jobs.logs.download",
                    )?;
                    let url = format!(
                        "{}/repos/{}/{}/actions/jobs/{}/logs",
                        github_base(ctx.profile()),
                        enc(owner),
                        enc(repo),
                        args.job
                    );
                    let bytes = client
                        .download("github", "actions.jobs.logs.download", ctx.profile(), url)
                        .await?;
                    write_download("github", "actions.jobs.logs.download", &args.output, &bytes)
                }
            },
        },
    }
}

fn github_repo_from_args<'a>(
    ctx: &'a Context,
    owner: Option<&'a str>,
    repo: Option<&'a str>,
    operation: &'static str,
) -> Result<(&'a str, &'a str), AppError> {
    github_repo(ctx.profile(), owner, repo, operation)
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

fn per_page_for(limit: u32) -> u32 {
    limit.clamp(1, GITHUB_PER_PAGE_MAX)
}

fn json_kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn aggregate_values(values: Vec<Value>, truncated: bool) -> Value {
    let size = values.len();
    json!({
        "values": values,
        "size": size,
        "truncated": truncated,
    })
}

fn keep_all(_value: &Value) -> bool {
    true
}

/// Walks GitHub paginated endpoints by incrementing `page=K` until either
/// the accumulated set reaches `limit` (then `truncated=true`) or the
/// provider returns a short page. Returns `{ values, size, truncated }`.
async fn paginate_github<F>(
    client: &ApiClient,
    operation: &'static str,
    ctx: &Context,
    base_url: String,
    limit: u32,
    filter: F,
) -> Result<Value, AppError>
where
    F: Fn(&Value) -> bool,
{
    let per_page = per_page_for(limit);
    let limit_usize = limit as usize;
    let mut accumulated: Vec<Value> = Vec::new();
    let mut truncated = false;
    let mut page: u32 = 1;
    'outer: loop {
        let separator = if base_url.contains('?') { '&' } else { '?' };
        let url = format!("{base_url}{separator}per_page={per_page}&page={page}");
        let response = client
            .request("github", operation, ctx.profile(), Method::GET, url, None)
            .await?;
        let values_opt = response
            .as_array()
            .cloned()
            .or_else(|| response.get("items").and_then(Value::as_array).cloned());
        let Some(values) = values_opt else {
            // Every paginated GitHub endpoint we wrap returns either a bare
            // array or an object with an `items` array. Anything else is an
            // unexpected provider response — surface it as an internal error
            // instead of silently dropping the documented `{ values, size,
            // truncated }` envelope and confusing downstream callers.
            return Err(AppError::internal(
                "github",
                operation,
                format!(
                    "expected an array or {{\"items\": [...]}} from GitHub, got {}",
                    json_kind(&response)
                ),
            ));
        };
        let page_len = values.len();
        for value in values {
            if !filter(&value) {
                continue;
            }
            accumulated.push(value);
            if accumulated.len() >= limit_usize {
                truncated = true;
                break 'outer;
            }
        }
        if (page_len as u32) < per_page {
            break 'outer;
        }
        page += 1;
    }
    Ok(aggregate_values(accumulated, truncated))
}

async fn pr_diff(client: &ApiClient, ctx: &Context, args: GithubPrDiff) -> Result<Value, AppError> {
    let (owner, repo) =
        github_repo_from_args(ctx, args.owner.as_deref(), args.repo.as_deref(), "prs.diff")?;
    let url = format!(
        "{}/repos/{}/{}/pulls/{}",
        github_base(ctx.profile()),
        enc(owner),
        enc(repo),
        args.pr
    );
    let bytes = client
        .download_with_accept("github", "prs.diff", ctx.profile(), url, GITHUB_DIFF_ACCEPT)
        .await?;
    if let Some(output) = args.output {
        return write_download("github", "prs.diff", &output, &bytes);
    }
    let text = String::from_utf8(bytes).map_err(|err| {
        AppError::internal(
            "github",
            "prs.diff",
            format!("diff response was not valid UTF-8: {err}"),
        )
    })?;
    Ok(Value::String(text))
}

async fn pr_files(
    client: &ApiClient,
    ctx: &Context,
    args: GithubPrFiles,
) -> Result<Value, AppError> {
    let (owner, repo) = github_repo_from_args(
        ctx,
        args.owner.as_deref(),
        args.repo.as_deref(),
        "prs.files",
    )?;
    let url = format!(
        "{}/repos/{}/{}/pulls/{}/files",
        github_base(ctx.profile()),
        enc(owner),
        enc(repo),
        args.pr
    );
    paginate_github(client, "prs.files", ctx, url, args.limit, keep_all).await
}

async fn pr_commits(
    client: &ApiClient,
    ctx: &Context,
    args: GithubPrCommits,
) -> Result<Value, AppError> {
    let (owner, repo) = github_repo_from_args(
        ctx,
        args.owner.as_deref(),
        args.repo.as_deref(),
        "prs.commits",
    )?;
    let url = format!(
        "{}/repos/{}/{}/pulls/{}/commits",
        github_base(ctx.profile()),
        enc(owner),
        enc(repo),
        args.pr
    );
    paginate_github(client, "prs.commits", ctx, url, args.limit, keep_all).await
}

async fn pr_timeline(
    client: &ApiClient,
    ctx: &Context,
    args: GithubPrTimeline,
) -> Result<Value, AppError> {
    let (owner, repo) = github_repo_from_args(
        ctx,
        args.owner.as_deref(),
        args.repo.as_deref(),
        "prs.timeline",
    )?;
    let url = format!(
        "{}/repos/{}/{}/issues/{}/timeline",
        github_base(ctx.profile()),
        enc(owner),
        enc(repo),
        args.pr
    );
    paginate_github(client, "prs.timeline", ctx, url, args.limit, keep_all).await
}

async fn pr_comments(
    client: &ApiClient,
    ctx: &Context,
    command: GithubPrCommentsCommand,
) -> Result<Value, AppError> {
    match command.action {
        GithubPrCommentAction::List(args) => {
            let (owner, repo) = github_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "pr-comments.list",
            )?;
            let url = format!(
                "{}/repos/{}/{}/issues/{}/comments?per_page={}",
                github_base(ctx.profile()),
                enc(owner),
                enc(repo),
                args.pr,
                args.limit
            );
            client
                .request(
                    "github",
                    "pr-comments.list",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
        }
        GithubPrCommentAction::Get(args) => {
            let (owner, repo) = github_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "pr-comments.get",
            )?;
            let url = format!(
                "{}/repos/{}/{}/issues/comments/{}",
                github_base(ctx.profile()),
                enc(owner),
                enc(repo),
                args.comment
            );
            client
                .request(
                    "github",
                    "pr-comments.get",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
        }
        GithubPrCommentAction::Create(args) => {
            let pr = args.pr;
            let owner_arg = args.owner.clone();
            let repo_arg = args.repo.clone();
            let (owner, repo) = github_repo_from_args(
                ctx,
                owner_arg.as_deref(),
                repo_arg.as_deref(),
                "pr-comments.create",
            )?;
            let body = pr_comment_body(args, "pr-comments.create")?;
            let url = format!(
                "{}/repos/{}/{}/issues/{}/comments",
                github_base(ctx.profile()),
                enc(owner),
                enc(repo),
                pr
            );
            client
                .request(
                    "github",
                    "pr-comments.create",
                    ctx.profile(),
                    Method::POST,
                    url,
                    Some(body),
                )
                .await
        }
        GithubPrCommentAction::Update(args) => {
            let comment = args.comment.ok_or_else(|| {
                AppError::invalid_input("github", "pr-comments.update", "--comment is required")
            })?;
            let owner_arg = args.owner.clone();
            let repo_arg = args.repo.clone();
            let (owner, repo) = github_repo_from_args(
                ctx,
                owner_arg.as_deref(),
                repo_arg.as_deref(),
                "pr-comments.update",
            )?;
            let body = pr_comment_body(args, "pr-comments.update")?;
            let url = format!(
                "{}/repos/{}/{}/issues/comments/{}",
                github_base(ctx.profile()),
                enc(owner),
                enc(repo),
                comment
            );
            client
                .request(
                    "github",
                    "pr-comments.update",
                    ctx.profile(),
                    Method::PATCH,
                    url,
                    Some(body),
                )
                .await
        }
        GithubPrCommentAction::Delete(args) => {
            let (owner, repo) = github_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "pr-comments.delete",
            )?;
            let url = format!(
                "{}/repos/{}/{}/issues/comments/{}",
                github_base(ctx.profile()),
                enc(owner),
                enc(repo),
                args.comment
            );
            client
                .request(
                    "github",
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

async fn pr_review_comments(
    client: &ApiClient,
    ctx: &Context,
    command: GithubPrReviewCommentsCommand,
) -> Result<Value, AppError> {
    match command.action {
        GithubPrReviewCommentAction::List(args) => {
            let (owner, repo) = github_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "pr-review-comments.list",
            )?;
            let url = format!(
                "{}/repos/{}/{}/pulls/{}/comments",
                github_base(ctx.profile()),
                enc(owner),
                enc(repo),
                args.pr
            );
            paginate_github(
                client,
                "pr-review-comments.list",
                ctx,
                url,
                args.limit,
                keep_all,
            )
            .await
        }
        GithubPrReviewCommentAction::Get(args) => {
            let (owner, repo) = github_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "pr-review-comments.get",
            )?;
            let url = format!(
                "{}/repos/{}/{}/pulls/comments/{}",
                github_base(ctx.profile()),
                enc(owner),
                enc(repo),
                args.comment
            );
            client
                .request(
                    "github",
                    "pr-review-comments.get",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
        }
        GithubPrReviewCommentAction::Create(args) => {
            let pr = args.pr;
            let in_reply_to = args.in_reply_to;
            let owner_arg = args.owner.clone();
            let repo_arg = args.repo.clone();
            let (owner, repo) = github_repo_from_args(
                ctx,
                owner_arg.as_deref(),
                repo_arg.as_deref(),
                "pr-review-comments.create",
            )?;
            if let Some(reply_to) = in_reply_to {
                let body = pr_review_comment_reply_body(args, "pr-review-comments.create")?;
                let url = format!(
                    "{}/repos/{}/{}/pulls/{}/comments/{}/replies",
                    github_base(ctx.profile()),
                    enc(owner),
                    enc(repo),
                    pr,
                    reply_to
                );
                client
                    .request(
                        "github",
                        "pr-review-comments.create",
                        ctx.profile(),
                        Method::POST,
                        url,
                        Some(body),
                    )
                    .await
            } else {
                let body = pr_review_comment_create_body(args, "pr-review-comments.create")?;
                let url = format!(
                    "{}/repos/{}/{}/pulls/{}/comments",
                    github_base(ctx.profile()),
                    enc(owner),
                    enc(repo),
                    pr
                );
                client
                    .request(
                        "github",
                        "pr-review-comments.create",
                        ctx.profile(),
                        Method::POST,
                        url,
                        Some(body),
                    )
                    .await
            }
        }
        GithubPrReviewCommentAction::Update(args) => {
            let comment = args.comment;
            let owner_arg = args.owner.clone();
            let repo_arg = args.repo.clone();
            let (owner, repo) = github_repo_from_args(
                ctx,
                owner_arg.as_deref(),
                repo_arg.as_deref(),
                "pr-review-comments.update",
            )?;
            let body = pr_review_comment_update_body(args, "pr-review-comments.update")?;
            let url = format!(
                "{}/repos/{}/{}/pulls/comments/{}",
                github_base(ctx.profile()),
                enc(owner),
                enc(repo),
                comment
            );
            client
                .request(
                    "github",
                    "pr-review-comments.update",
                    ctx.profile(),
                    Method::PATCH,
                    url,
                    Some(body),
                )
                .await
        }
        GithubPrReviewCommentAction::Delete(args) => {
            let (owner, repo) = github_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "pr-review-comments.delete",
            )?;
            let url = format!(
                "{}/repos/{}/{}/pulls/comments/{}",
                github_base(ctx.profile()),
                enc(owner),
                enc(repo),
                args.comment
            );
            client
                .request(
                    "github",
                    "pr-review-comments.delete",
                    ctx.profile(),
                    Method::DELETE,
                    url,
                    None,
                )
                .await
        }
    }
}

async fn pr_reviews(
    client: &ApiClient,
    ctx: &Context,
    command: GithubPrReviewsCommand,
) -> Result<Value, AppError> {
    match command.action {
        GithubPrReviewAction::List(args) => {
            let (owner, repo) = github_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "pr-reviews.list",
            )?;
            let url = format!(
                "{}/repos/{}/{}/pulls/{}/reviews",
                github_base(ctx.profile()),
                enc(owner),
                enc(repo),
                args.pr
            );
            paginate_github(client, "pr-reviews.list", ctx, url, args.limit, keep_all).await
        }
        GithubPrReviewAction::Get(args) => {
            let (owner, repo) = github_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "pr-reviews.get",
            )?;
            let url = format!(
                "{}/repos/{}/{}/pulls/{}/reviews/{}",
                github_base(ctx.profile()),
                enc(owner),
                enc(repo),
                args.pr,
                args.review
            );
            client
                .request(
                    "github",
                    "pr-reviews.get",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
        }
        GithubPrReviewAction::Create(args) => {
            let pr = args.pr;
            let owner_arg = args.owner.clone();
            let repo_arg = args.repo.clone();
            let (owner, repo) = github_repo_from_args(
                ctx,
                owner_arg.as_deref(),
                repo_arg.as_deref(),
                "pr-reviews.create",
            )?;
            let body = pr_review_create_body(args, "pr-reviews.create")?;
            let url = format!(
                "{}/repos/{}/{}/pulls/{}/reviews",
                github_base(ctx.profile()),
                enc(owner),
                enc(repo),
                pr
            );
            client
                .request(
                    "github",
                    "pr-reviews.create",
                    ctx.profile(),
                    Method::POST,
                    url,
                    Some(body),
                )
                .await
        }
    }
}

async fn branches(
    client: &ApiClient,
    ctx: &Context,
    command: GithubBranchesCommand,
) -> Result<Value, AppError> {
    match command.action {
        GithubBranchesAction::List(args) => {
            let (owner, repo) = github_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "branches.list",
            )?;
            let mut url = format!(
                "{}/repos/{}/{}/branches",
                github_base(ctx.profile()),
                enc(owner),
                enc(repo)
            );
            if let Some(protected) = args.protected {
                url.push('?');
                url.push_str("protected=");
                url.push_str(if protected { "true" } else { "false" });
            }
            let name_contains = args.name_contains.clone();
            let name_prefix = args.name_prefix.clone();
            let filter = move |value: &Value| -> bool {
                let Some(name) = value.get("name").and_then(Value::as_str) else {
                    return true;
                };
                if let Some(prefix) = name_prefix.as_deref() {
                    return name.starts_with(prefix);
                }
                if let Some(needle) = name_contains.as_deref() {
                    let needle_lower = needle.to_lowercase();
                    return name.to_lowercase().contains(&needle_lower);
                }
                true
            };
            paginate_github(client, "branches.list", ctx, url, args.limit, filter).await
        }
        GithubBranchesAction::Get(args) => {
            let (owner, repo) = github_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "branches.get",
            )?;
            let url = format!(
                "{}/repos/{}/{}/branches/{}",
                github_base(ctx.profile()),
                enc(owner),
                enc(repo),
                enc(&args.name)
            );
            client
                .request(
                    "github",
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

async fn source(
    client: &ApiClient,
    ctx: &Context,
    command: GithubSourceCommand,
) -> Result<Value, AppError> {
    match command.action {
        GithubSourceAction::Get(args) => {
            let (owner, repo) = github_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "source.get",
            )?;
            let path = enc_path(&args.path);
            let url = format!(
                "{}/repos/{}/{}/contents/{}?ref={}",
                github_base(ctx.profile()),
                enc(owner),
                enc(repo),
                path,
                enc(&args.commit)
            );
            if args.meta {
                return client
                    .request(
                        "github",
                        "source.get",
                        ctx.profile(),
                        Method::GET,
                        url,
                        None,
                    )
                    .await;
            }
            let bytes = client
                .download_with_accept(
                    "github",
                    "source.get",
                    ctx.profile(),
                    url,
                    GITHUB_RAW_ACCEPT,
                )
                .await?;
            if let Some(output) = args.output {
                return write_download("github", "source.get", &output, &bytes);
            }
            let text = String::from_utf8(bytes).map_err(|err| {
                AppError::internal(
                    "github",
                    "source.get",
                    format!(
                        "source response was not valid UTF-8 (use --output for binary files): {err}"
                    ),
                )
            })?;
            Ok(Value::String(text))
        }
        GithubSourceAction::History(args) => {
            let (owner, repo) = github_repo_from_args(
                ctx,
                args.owner.as_deref(),
                args.repo.as_deref(),
                "source.history",
            )?;
            // GitHub's /commits?path= filter expects the literal repo-relative path
            // with `/` preserved between segments; using enc() would percent-encode
            // them as %2F, which the API treats as "no match" and silently returns
            // an empty list. enc_path() encodes only special chars within each
            // segment, matching the behavior of source.get.
            let url = format!(
                "{}/repos/{}/{}/commits?path={}&sha={}",
                github_base(ctx.profile()),
                enc(owner),
                enc(repo),
                enc_path(&args.path),
                enc(&args.commit)
            );
            paginate_github(client, "source.history", ctx, url, args.limit, keep_all).await
        }
    }
}

fn issue_create_body(args: GithubIssueCreate) -> Result<Value, AppError> {
    let mut body = input::read_json_arg("github", "issues.create", args.json.as_deref())?;
    input::set_string(&mut body, "title", &args.title);
    input::set_string(&mut body, "body", &args.body);
    Ok(body)
}

fn issue_update_body(args: GithubIssueUpdate) -> Result<Value, AppError> {
    let mut body = input::read_json_arg("github", "issues.update", args.json.as_deref())?;
    input::set_string(&mut body, "title", &args.title);
    input::set_string(&mut body, "body", &args.body);
    input::set_string(&mut body, "state", &args.state);
    Ok(body)
}

fn pr_create_body(args: PullRequestCreate) -> Result<Value, AppError> {
    let mut body = input::read_json_arg("github", "prs.create", args.json.as_deref())?;
    input::set_string(&mut body, "title", &args.title);
    input::set_string(&mut body, "body", &args.body);
    input::set_string(&mut body, "head", &args.head.or(args.source));
    input::set_string(&mut body, "base", &args.base.or(args.destination));
    Ok(body)
}

fn pr_comment_body(args: GithubPrCommentWrite, operation: &'static str) -> Result<Value, AppError> {
    let mut body = input::read_json_arg("github", operation, args.json.as_deref())?;
    input::set_string(&mut body, "body", &args.body);
    if body.get("body").is_none() {
        return Err(AppError::invalid_input(
            "github",
            operation,
            "--body or JSON body is required",
        ));
    }
    Ok(body)
}

fn pr_review_comment_create_body(
    args: GithubPrReviewCommentCreate,
    operation: &'static str,
) -> Result<Value, AppError> {
    let mut body = input::read_json_arg("github", operation, args.json.as_deref())?;
    input::set_string(&mut body, "body", &args.body);
    input::set_string(&mut body, "commit_id", &args.commit_id);
    input::set_string(&mut body, "path", &args.path);
    input::set_u64(&mut body, "line", args.line);
    input::set_string(&mut body, "side", &args.side);
    input::set_u64(&mut body, "start_line", args.start_line);
    input::set_string(&mut body, "start_side", &args.start_side);

    if body.get("body").is_none() {
        return Err(AppError::invalid_input(
            "github",
            operation,
            "--body or JSON body is required",
        ));
    }
    if body.get("commit_id").is_none() {
        return Err(AppError::invalid_input(
            "github",
            operation,
            "--commit-id or JSON commit_id is required for inline review comments",
        ));
    }
    if body.get("path").is_none() {
        return Err(AppError::invalid_input(
            "github",
            operation,
            "--path or JSON path is required for inline review comments",
        ));
    }
    Ok(body)
}

fn pr_review_comment_reply_body(
    args: GithubPrReviewCommentCreate,
    operation: &'static str,
) -> Result<Value, AppError> {
    let mut body = input::read_json_arg("github", operation, args.json.as_deref())?;
    input::set_string(&mut body, "body", &args.body);
    if body.get("body").is_none() {
        return Err(AppError::invalid_input(
            "github",
            operation,
            "--body or JSON body is required for replies",
        ));
    }
    Ok(body)
}

fn pr_review_comment_update_body(
    args: GithubPrReviewCommentUpdate,
    operation: &'static str,
) -> Result<Value, AppError> {
    let mut body = input::read_json_arg("github", operation, args.json.as_deref())?;
    input::set_string(&mut body, "body", &args.body);
    if body.get("body").is_none() {
        return Err(AppError::invalid_input(
            "github",
            operation,
            "--body or JSON body is required",
        ));
    }
    Ok(body)
}

fn pr_review_create_body(
    args: GithubPrReviewCreate,
    operation: &'static str,
) -> Result<Value, AppError> {
    let mut body = input::read_json_arg("github", operation, args.json.as_deref())?;
    input::set_string(&mut body, "body", &args.body);
    input::set_string(&mut body, "event", &args.event);
    input::set_string(&mut body, "commit_id", &args.commit_id);
    if let Some(comments_json) = args.comments_json.as_deref() {
        let comments = input::read_json_arg("github", operation, Some(comments_json))?;
        if !comments.is_array() {
            return Err(AppError::invalid_input(
                "github",
                operation,
                "--comments-json must be a JSON array of review comments",
            ));
        }
        input::ensure_object(&mut body).insert("comments".to_string(), comments);
    }

    // GitHub's POST /pulls/{n}/reviews accepts event values APPROVE,
    // REQUEST_CHANGES, COMMENT, or omits the field entirely to create a
    // PENDING (draft) review. The literal "PENDING" string is rejected by
    // the API with HTTP 422, so when callers pass --event PENDING we accept
    // it as a friendly alias and strip the field from the body.
    let pending_event = match body.get("event").and_then(Value::as_str) {
        Some("APPROVE") | Some("REQUEST_CHANGES") | Some("COMMENT") => false,
        Some("PENDING") => true,
        Some(other) => {
            return Err(AppError::invalid_input(
                "github",
                operation,
                format!(
                    "--event must be one of APPROVE, REQUEST_CHANGES, COMMENT, PENDING (got '{other}')"
                ),
            ));
        }
        None => false,
    };
    if pending_event {
        if let Some(obj) = body.as_object_mut() {
            obj.remove("event");
        }
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_override_json() {
        let body = issue_create_body(GithubIssueCreate {
            json: Some(r#"{"title":"old","labels":["bug"]}"#.to_string()),
            owner: None,
            repo: None,
            title: Some("new".to_string()),
            body: None,
        })
        .unwrap();
        assert_eq!(body["title"], "new");
        assert_eq!(body["labels"][0], "bug");
    }

    fn review_comment_args(
        body: Option<&str>,
        path: Option<&str>,
        commit_id: Option<&str>,
        line: Option<u64>,
        in_reply_to: Option<u64>,
        json: Option<&str>,
    ) -> GithubPrReviewCommentCreate {
        GithubPrReviewCommentCreate {
            pr: 1,
            owner: None,
            repo: None,
            json: json.map(str::to_string),
            body: body.map(str::to_string),
            path: path.map(str::to_string),
            line,
            side: None,
            start_line: None,
            start_side: None,
            commit_id: commit_id.map(str::to_string),
            in_reply_to,
        }
    }

    #[test]
    fn review_comment_create_requires_path_and_commit() {
        let err = pr_review_comment_create_body(
            review_comment_args(Some("LGTM"), None, Some("abc123"), Some(10), None, None),
            "pr-review-comments.create",
        )
        .unwrap_err();
        assert_eq!(err.code, "invalid_input");
    }

    #[test]
    fn review_comment_create_assembles_body() {
        let body = pr_review_comment_create_body(
            review_comment_args(
                Some("please rename"),
                Some("src/lib.rs"),
                Some("abc123"),
                Some(120),
                None,
                None,
            ),
            "pr-review-comments.create",
        )
        .unwrap();
        assert_eq!(body["body"], "please rename");
        assert_eq!(body["path"], "src/lib.rs");
        assert_eq!(body["commit_id"], "abc123");
        assert_eq!(body["line"], 120);
    }

    #[test]
    fn review_comment_reply_only_requires_body() {
        let body = pr_review_comment_reply_body(
            review_comment_args(Some("agreed"), None, None, None, Some(7), None),
            "pr-review-comments.create",
        )
        .unwrap();
        assert_eq!(body["body"], "agreed");
    }

    #[test]
    fn review_create_merges_comments_array() {
        let body = pr_review_create_body(
            GithubPrReviewCreate {
                pr: 1,
                owner: None,
                repo: None,
                json: None,
                body: Some("Looks good with one nit".to_string()),
                event: Some("REQUEST_CHANGES".to_string()),
                commit_id: Some("abc123".to_string()),
                comments_json: Some(
                    r#"[{"path":"src/lib.rs","line":10,"body":"nit: rename"}]"#.to_string(),
                ),
            },
            "pr-reviews.create",
        )
        .unwrap();
        assert_eq!(body["event"], "REQUEST_CHANGES");
        assert_eq!(body["body"], "Looks good with one nit");
        assert_eq!(body["commit_id"], "abc123");
        assert_eq!(body["comments"][0]["path"], "src/lib.rs");
        assert_eq!(body["comments"][0]["line"], 10);
        assert_eq!(body["comments"][0]["body"], "nit: rename");
    }

    #[test]
    fn review_create_rejects_unknown_event() {
        let err = pr_review_create_body(
            GithubPrReviewCreate {
                pr: 1,
                owner: None,
                repo: None,
                json: None,
                body: Some("nope".to_string()),
                event: Some("REJECT".to_string()),
                commit_id: None,
                comments_json: None,
            },
            "pr-reviews.create",
        )
        .unwrap_err();
        assert_eq!(err.code, "invalid_input");
    }

    #[test]
    fn review_create_rejects_non_array_comments_json() {
        let err = pr_review_create_body(
            GithubPrReviewCreate {
                pr: 1,
                owner: None,
                repo: None,
                json: None,
                body: None,
                event: Some("COMMENT".to_string()),
                commit_id: None,
                comments_json: Some(r#"{"path":"x"}"#.to_string()),
            },
            "pr-reviews.create",
        )
        .unwrap_err();
        assert_eq!(err.code, "invalid_input");
    }

    #[test]
    fn aggregate_values_shape_matches_contract() {
        let collected = vec![json!({"id": 1}), json!({"id": 2})];
        let out = aggregate_values(collected, true);
        assert_eq!(out["values"].as_array().unwrap().len(), 2);
        assert_eq!(out["size"], 2);
        assert_eq!(out["truncated"], true);
    }

    #[test]
    fn per_page_for_clamps_to_max() {
        assert_eq!(per_page_for(0), 1);
        assert_eq!(per_page_for(50), 50);
        assert_eq!(per_page_for(500), GITHUB_PER_PAGE_MAX);
    }

    #[test]
    fn pr_comment_body_requires_body() {
        let err = pr_comment_body(
            GithubPrCommentWrite {
                pr: 1,
                comment: None,
                owner: None,
                repo: None,
                json: None,
                body: None,
            },
            "pr-comments.create",
        )
        .unwrap_err();
        assert_eq!(err.code, "invalid_input");
    }

    #[test]
    fn enc_path_strips_leading_slash_and_encodes_segments() {
        assert_eq!(enc_path("/src/main.rs"), "src/main.rs");
        assert_eq!(
            enc_path("src/with space/file.txt"),
            "src/with%20space/file.txt"
        );
    }

    #[test]
    fn review_create_strips_pending_event() {
        let body = pr_review_create_body(
            GithubPrReviewCreate {
                pr: 1,
                owner: None,
                repo: None,
                json: None,
                body: Some("draft notes".to_string()),
                event: Some("PENDING".to_string()),
                commit_id: Some("deadbeef".to_string()),
                comments_json: None,
            },
            "pr-reviews.create",
        )
        .unwrap();
        assert!(
            body.get("event").is_none(),
            "event must be omitted for PENDING reviews; GitHub rejects literal \"PENDING\""
        );
        assert_eq!(body["body"], "draft notes");
        assert_eq!(body["commit_id"], "deadbeef");
    }

    #[test]
    fn review_create_keeps_real_events() {
        for event in ["APPROVE", "REQUEST_CHANGES", "COMMENT"] {
            let body = pr_review_create_body(
                GithubPrReviewCreate {
                    pr: 1,
                    owner: None,
                    repo: None,
                    json: None,
                    body: Some("hi".to_string()),
                    event: Some(event.to_string()),
                    commit_id: None,
                    comments_json: None,
                },
                "pr-reviews.create",
            )
            .unwrap();
            assert_eq!(body["event"], event);
        }
    }

    #[test]
    fn json_kind_labels_each_variant() {
        assert_eq!(json_kind(&Value::Null), "null");
        assert_eq!(json_kind(&json!(true)), "bool");
        assert_eq!(json_kind(&json!(7)), "number");
        assert_eq!(json_kind(&json!("hi")), "string");
        assert_eq!(json_kind(&json!([1, 2])), "array");
        assert_eq!(json_kind(&json!({"a": 1})), "object");
    }
}
