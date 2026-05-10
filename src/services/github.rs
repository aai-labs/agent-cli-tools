use reqwest::Method;
use serde_json::{json, Value};

use crate::{
    cli::*,
    config::Context,
    error::AppError,
    http::ApiClient,
    input,
    services::shared::{enc, github_base, github_repo, CtxProfile},
};

pub(crate) async fn dispatch(
    client: &ApiClient,
    ctx: &Context,
    command: GithubCommand,
) -> Result<Value, AppError> {
    match command.resource {
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
                let (owner, repo) = github_repo(
                    ctx.profile(),
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
                let (owner, repo) = github_repo(
                    ctx.profile(),
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
                let (owner, repo) = github_repo(
                    ctx.profile(),
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
                let (owner, repo) = github_repo(
                    ctx.profile(),
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
                let (owner, repo) = github_repo(
                    ctx.profile(),
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
                let (owner, repo) = github_repo(
                    ctx.profile(),
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
                let (owner, repo) = github_repo(
                    ctx.profile(),
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
                let (owner, repo) = github_repo(
                    ctx.profile(),
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
                let (owner, repo) = github_repo(
                    ctx.profile(),
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
                let (owner, repo) = github_repo(
                    ctx.profile(),
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
            GithubPullRequestAction::Comments(command) => pr_comments(client, ctx, command).await,
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
            let (owner, repo) = github_repo(
                ctx.profile(),
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
        BitbucketPrCommentAction::Get(args) => {
            let (owner, repo) = github_repo(
                ctx.profile(),
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
        BitbucketPrCommentAction::Create(args) => {
            let pr = args.pr;
            let owner_arg = args.owner.clone();
            let repo_arg = args.repo.clone();
            let (owner, repo) = github_repo(
                ctx.profile(),
                owner_arg.as_deref(),
                repo_arg.as_deref(),
                "pr-comments.create",
            )?;
            let body = github_pr_comment_body(args, "pr-comments.create")?;
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
        BitbucketPrCommentAction::Update(args) => {
            let comment = args.comment.ok_or_else(|| {
                AppError::invalid_input("github", "pr-comments.update", "--comment is required")
            })?;
            let owner_arg = args.owner.clone();
            let repo_arg = args.repo.clone();
            let (owner, repo) = github_repo(
                ctx.profile(),
                owner_arg.as_deref(),
                repo_arg.as_deref(),
                "pr-comments.update",
            )?;
            let body = github_pr_comment_body(args, "pr-comments.update")?;
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
        BitbucketPrCommentAction::Delete(args) => {
            let (owner, repo) = github_repo(
                ctx.profile(),
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

fn github_pr_comment_body(
    args: BitbucketPrCommentWrite,
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
}
