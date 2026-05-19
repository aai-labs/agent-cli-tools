use std::{
    env,
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::Value;

fn cli(profile_env: &str, args: &[&str]) -> Option<Value> {
    let config = match env::var("AAI_E2E_CONFIG") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("skipping live E2E: AAI_E2E_CONFIG is not set");
            return None;
        }
    };
    let profile = match env::var(profile_env) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("skipping live E2E: {profile_env} is not set");
            return None;
        }
    };

    let mut command = Command::new(env!("CARGO_BIN_EXE_aai-cli"));
    command
        .arg("--config")
        .arg(config)
        .arg("--profile")
        .arg(profile);
    command.args(args);
    Some(parse_success(
        command.output().expect("failed to execute aai-cli"),
        args,
    ))
}

fn cli_required(profile_env: &str, args: &[&str]) -> Value {
    cli(profile_env, args).unwrap_or_else(|| panic!("missing required E2E env for {profile_env}"))
}

fn parse_success(output: Output, args: &[&str]) -> Value {
    if !output.status.success() {
        panic!(
            "command failed: {:?}\nstatus: {}\nstdout: {}\nstderr: {}",
            args,
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&stdout)
            .unwrap_or_else(|err| panic!("invalid JSON stdout for {:?}: {err}\n{stdout}", args))
    }
}

fn env_or_skip(name: &str) -> Option<String> {
    match env::var(name) {
        Ok(value) if !value.trim().is_empty() => Some(value),
        _ => {
            eprintln!("skipping live E2E branch: {name} is not set");
            None
        }
    }
}

fn unique(prefix: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before epoch")
        .as_millis();
    format!("{prefix}-{millis}")
}

fn str_at<'a>(value: &'a Value, path: &[&str]) -> &'a str {
    let mut current = value;
    for segment in path {
        current = current
            .get(*segment)
            .unwrap_or_else(|| panic!("missing path {path:?} in {value:#}"));
    }
    current
        .as_str()
        .unwrap_or_else(|| panic!("path {path:?} is not string in {value:#}"))
}

fn u64_at(value: &Value, path: &[&str]) -> u64 {
    let mut current = value;
    for segment in path {
        current = current
            .get(*segment)
            .unwrap_or_else(|| panic!("missing path {path:?} in {value:#}"));
    }
    current
        .as_u64()
        .unwrap_or_else(|| panic!("path {path:?} is not u64 in {value:#}"))
}

fn find_string(value: &Value, keys: &[&str]) -> Option<String> {
    match value {
        Value::Object(map) => {
            for key in keys {
                if let Some(found) = map.get(*key).and_then(Value::as_str) {
                    return Some(found.to_string());
                }
            }
            map.values().find_map(|value| find_string(value, keys))
        }
        Value::Array(values) => values.iter().find_map(|value| find_string(value, keys)),
        _ => None,
    }
}

fn find_u64(value: &Value, keys: &[&str]) -> Option<u64> {
    match value {
        Value::Object(map) => {
            for key in keys {
                if let Some(found) = map.get(*key).and_then(Value::as_u64) {
                    return Some(found);
                }
            }
            map.values().find_map(|value| find_u64(value, keys))
        }
        Value::Array(values) => values.iter().find_map(|value| find_u64(value, keys)),
        _ => None,
    }
}

#[test]
#[ignore = "requires live Jira credentials and a disposable project"]
fn jira_issue_crud_and_projects() {
    let Some(project) = env_or_skip("AAI_E2E_JIRA_PROJECT") else {
        return;
    };
    let summary = unique("aai-e2e-jira");

    let created = cli_required(
        "AAI_E2E_JIRA_PROFILE",
        &[
            "jira",
            "issues",
            "create",
            "--project",
            &project,
            "--summary",
            &summary,
            "--description",
            "created by aai-cli live e2e",
        ],
    );
    let issue_key = str_at(&created, &["key"]).to_string();

    let _ = cli_required(
        "AAI_E2E_JIRA_PROFILE",
        &[
            "jira",
            "issues",
            "list",
            "--jql",
            &format!("key = {issue_key}"),
        ],
    );
    let fetched = cli_required(
        "AAI_E2E_JIRA_PROFILE",
        &["jira", "issues", "get", &issue_key],
    );
    assert_eq!(str_at(&fetched, &["key"]), issue_key);
    let _ = cli_required("AAI_E2E_JIRA_PROFILE", &["jira", "projects", "list"]);
    let _ = cli_required(
        "AAI_E2E_JIRA_PROFILE",
        &["jira", "projects", "get", &project],
    );
    let _ = cli_required(
        "AAI_E2E_JIRA_PROFILE",
        &["jira", "issues", "delete", &issue_key],
    );
}

#[test]
#[ignore = "requires live Confluence credentials and a disposable space"]
fn confluence_page_crud_and_spaces() {
    let Some(space_id) = env_or_skip("AAI_E2E_CONFLUENCE_SPACE_ID") else {
        return;
    };
    let title = unique("aai-e2e-page");

    let created = cli_required(
        "AAI_E2E_CONFLUENCE_PROFILE",
        &[
            "confluence",
            "pages",
            "create",
            "--space-id",
            &space_id,
            "--title",
            &title,
            "--body",
            "created by aai-cli live e2e",
        ],
    );
    let page_id = str_at(&created, &["id"]).to_string();

    let _ = cli_required(
        "AAI_E2E_CONFLUENCE_PROFILE",
        &["confluence", "pages", "list"],
    );
    let fetched = cli_required(
        "AAI_E2E_CONFLUENCE_PROFILE",
        &["confluence", "pages", "get", &page_id],
    );
    assert_eq!(str_at(&fetched, &["id"]), page_id);
    let _ = cli_required(
        "AAI_E2E_CONFLUENCE_PROFILE",
        &["confluence", "spaces", "list"],
    );
    let _ = cli_required(
        "AAI_E2E_CONFLUENCE_PROFILE",
        &["confluence", "spaces", "get", &space_id],
    );
    let _ = cli_required(
        "AAI_E2E_CONFLUENCE_PROFILE",
        &["confluence", "pages", "delete", &page_id],
    );
}

#[test]
#[ignore = "requires live GitHub credentials and a disposable repo"]
fn github_issue_crud_repos_and_optional_prs() {
    let created = cli_required(
        "AAI_E2E_GITHUB_PROFILE",
        &[
            "github",
            "issues",
            "create",
            "--title",
            &unique("aai-e2e-github"),
            "--body",
            "created by aai-cli live e2e",
        ],
    );
    let number = u64_at(&created, &["number"]).to_string();

    let _ = cli_required("AAI_E2E_GITHUB_PROFILE", &["github", "issues", "list"]);
    let fetched = cli_required(
        "AAI_E2E_GITHUB_PROFILE",
        &["github", "issues", "get", &number],
    );
    assert_eq!(u64_at(&fetched, &["number"]).to_string(), number);
    let _ = cli_required("AAI_E2E_GITHUB_PROFILE", &["github", "repos", "list"]);
    let _ = cli_required("AAI_E2E_GITHUB_PROFILE", &["github", "repos", "get"]);
    let _ = cli_required(
        "AAI_E2E_GITHUB_PROFILE",
        &["github", "issues", "delete", &number],
    );

    let Some(head) = env_or_skip("AAI_E2E_GITHUB_PR_HEAD") else {
        return;
    };
    let Some(base) = env_or_skip("AAI_E2E_GITHUB_PR_BASE") else {
        return;
    };
    let created_pr = cli_required(
        "AAI_E2E_GITHUB_PROFILE",
        &[
            "github",
            "prs",
            "create",
            "--title",
            &unique("aai-e2e-pr"),
            "--head",
            &head,
            "--base",
            &base,
            "--body",
            "created by aai-cli live e2e",
        ],
    );
    let pr_number = u64_at(&created_pr, &["number"]).to_string();
    let _ = cli_required("AAI_E2E_GITHUB_PROFILE", &["github", "prs", "list"]);
    let _ = cli_required(
        "AAI_E2E_GITHUB_PROFILE",
        &["github", "prs", "get", &pr_number],
    );
    let comment = cli_required(
        "AAI_E2E_GITHUB_PROFILE",
        &[
            "github",
            "prs",
            "comments",
            "create",
            &pr_number,
            "--body",
            "created by aai-cli live e2e",
        ],
    );
    let comment_id = u64_at(&comment, &["id"]).to_string();
    let _ = cli_required(
        "AAI_E2E_GITHUB_PROFILE",
        &["github", "prs", "comments", "list", &pr_number],
    );
    let _ = cli_required(
        "AAI_E2E_GITHUB_PROFILE",
        &["github", "prs", "comments", "get", &pr_number, &comment_id],
    );
    let _ = cli_required(
        "AAI_E2E_GITHUB_PROFILE",
        &["github", "prs", "close", &pr_number],
    );
}

#[test]
#[ignore = "requires live Bitbucket credentials and a disposable repo"]
fn bitbucket_repos_and_optional_prs() {
    let Some(repo) = env_or_skip("AAI_E2E_BITBUCKET_REPO") else {
        return;
    };
    let _ = cli_required("AAI_E2E_BITBUCKET_PROFILE", &["bitbucket", "repos", "list"]);
    let _ = cli_required(
        "AAI_E2E_BITBUCKET_PROFILE",
        &["bitbucket", "repos", "get", &repo],
    );

    let Some(source) = env_or_skip("AAI_E2E_BITBUCKET_PR_SOURCE") else {
        return;
    };
    let Some(destination) = env_or_skip("AAI_E2E_BITBUCKET_PR_DESTINATION") else {
        return;
    };
    let created_pr = cli_required(
        "AAI_E2E_BITBUCKET_PROFILE",
        &[
            "bitbucket",
            "prs",
            "create",
            "--repo",
            &repo,
            "--title",
            &unique("aai-e2e-bb-pr"),
            "--source",
            &source,
            "--destination",
            &destination,
            "--body",
            "created by aai-cli live e2e",
        ],
    );
    let pr_number = find_u64(&created_pr, &["id"]).expect("Bitbucket PR response missing id");
    let pr_number = pr_number.to_string();
    let _ = cli_required(
        "AAI_E2E_BITBUCKET_PROFILE",
        &["bitbucket", "prs", "list", "--repo", &repo],
    );
    let _ = cli_required(
        "AAI_E2E_BITBUCKET_PROFILE",
        &["bitbucket", "prs", "get", &pr_number, "--repo", &repo],
    );
    let _ = cli_required(
        "AAI_E2E_BITBUCKET_PROFILE",
        &["bitbucket", "prs", "diff", &pr_number, "--repo", &repo],
    );
    let _ = cli_required(
        "AAI_E2E_BITBUCKET_PROFILE",
        &["bitbucket", "prs", "diffstat", &pr_number, "--repo", &repo],
    );
    let _ = cli_required(
        "AAI_E2E_BITBUCKET_PROFILE",
        &["bitbucket", "prs", "commits", &pr_number, "--repo", &repo],
    );
    let _ = cli_required(
        "AAI_E2E_BITBUCKET_PROFILE",
        &["bitbucket", "prs", "activity", &pr_number, "--repo", &repo],
    );
    let comment = cli_required(
        "AAI_E2E_BITBUCKET_PROFILE",
        &[
            "bitbucket",
            "prs",
            "comments",
            "create",
            &pr_number,
            "--repo",
            &repo,
            "--body",
            "created by aai-cli live e2e",
        ],
    );
    let comment_id = find_u64(&comment, &["id"]).expect("Bitbucket PR comment missing id");
    let comment_id = comment_id.to_string();
    let _ = cli_required(
        "AAI_E2E_BITBUCKET_PROFILE",
        &[
            "bitbucket",
            "prs",
            "comments",
            "list",
            &pr_number,
            "--repo",
            &repo,
        ],
    );
    let _ = cli_required(
        "AAI_E2E_BITBUCKET_PROFILE",
        &[
            "bitbucket",
            "prs",
            "comments",
            "get",
            &pr_number,
            &comment_id,
            "--repo",
            &repo,
        ],
    );
    if let Some(inline_path) = env_or_skip("AAI_E2E_BITBUCKET_SOURCE_PATH") {
        let _ = cli_required(
            "AAI_E2E_BITBUCKET_PROFILE",
            &[
                "bitbucket",
                "prs",
                "comments",
                "create",
                &pr_number,
                "--repo",
                &repo,
                "--body",
                "inline comment from aai-cli live e2e",
                "--inline-path",
                &inline_path,
                "--inline-to",
                "1",
            ],
        );
        let _ = cli_required(
            "AAI_E2E_BITBUCKET_PROFILE",
            &[
                "bitbucket",
                "prs",
                "comments",
                "list",
                &pr_number,
                "--repo",
                &repo,
                "--inline-only",
            ],
        );
    }
    let _ = cli_required(
        "AAI_E2E_BITBUCKET_PROFILE",
        &["bitbucket", "prs", "decline", &pr_number, "--repo", &repo],
    );
}

#[test]
#[ignore = "requires live Bitbucket credentials and read-only repo metadata"]
fn bitbucket_read_only_endpoints() {
    let Some(repo) = env_or_skip("AAI_E2E_BITBUCKET_REPO") else {
        return;
    };
    let branches = cli_required(
        "AAI_E2E_BITBUCKET_PROFILE",
        &["bitbucket", "branches", "list", "--repo", &repo],
    );
    let branch_name = env_or_skip("AAI_E2E_BITBUCKET_BRANCH").or_else(|| {
        branches
            .get("values")
            .and_then(Value::as_array)
            .and_then(|values| values.first())
            .and_then(|branch| branch.get("name"))
            .and_then(Value::as_str)
            .map(str::to_string)
    });
    let Some(branch_name) = branch_name else {
        eprintln!("skipping live E2E branch: no branch name available");
        return;
    };
    let _ = cli_required(
        "AAI_E2E_BITBUCKET_PROFILE",
        &[
            "bitbucket",
            "branches",
            "get",
            &branch_name,
            "--repo",
            &repo,
        ],
    );
    let commits = cli_required(
        "AAI_E2E_BITBUCKET_PROFILE",
        &[
            "bitbucket",
            "commits",
            "list",
            "--repo",
            &repo,
            "--branch",
            &branch_name,
        ],
    );
    let commit_sha = env_or_skip("AAI_E2E_BITBUCKET_COMMIT_SHA").or_else(|| {
        commits
            .get("values")
            .and_then(Value::as_array)
            .and_then(|values| values.first())
            .and_then(|commit| commit.get("hash"))
            .and_then(Value::as_str)
            .map(str::to_string)
    });
    let Some(commit_sha) = commit_sha else {
        eprintln!("skipping live E2E branch: no commit SHA available");
        return;
    };
    let _ = cli_required(
        "AAI_E2E_BITBUCKET_PROFILE",
        &["bitbucket", "commits", "get", &commit_sha, "--repo", &repo],
    );
    let Some(source_path) = env_or_skip("AAI_E2E_BITBUCKET_SOURCE_PATH") else {
        return;
    };
    let _ = cli_required(
        "AAI_E2E_BITBUCKET_PROFILE",
        &[
            "bitbucket",
            "source",
            "get",
            &commit_sha,
            &source_path,
            "--repo",
            &repo,
        ],
    );
    let _ = cli_required(
        "AAI_E2E_BITBUCKET_PROFILE",
        &[
            "bitbucket",
            "source",
            "history",
            &commit_sha,
            &source_path,
            "--repo",
            &repo,
        ],
    );
}

#[test]
#[ignore = "requires live Google Gmail credentials and a test recipient"]
fn gmail_message_send_list_get_delete() {
    let Some(to) = env_or_skip("AAI_E2E_EMAIL_TO") else {
        return;
    };
    let sent = cli_required(
        "AAI_E2E_GMAIL_PROFILE",
        &[
            "email",
            "messages",
            "send",
            "--to",
            &to,
            "--subject",
            &unique("aai-e2e-gmail"),
            "--body",
            "created by aai-cli live e2e",
        ],
    );
    let id = str_at(&sent, &["id"]).to_string();
    let _ = cli_required("AAI_E2E_GMAIL_PROFILE", &["email", "messages", "list"]);
    let _ = cli_required("AAI_E2E_GMAIL_PROFILE", &["email", "messages", "get", &id]);
    let _ = cli_required(
        "AAI_E2E_GMAIL_PROFILE",
        &["email", "messages", "delete", &id],
    );
}

#[test]
#[ignore = "requires live Zoho Mail credentials and a test recipient"]
fn zoho_mail_message_send_list_get_delete() {
    let Some(to) = env_or_skip("AAI_E2E_EMAIL_TO") else {
        return;
    };
    let sent = cli_required(
        "AAI_E2E_ZOHO_MAIL_PROFILE",
        &[
            "email",
            "messages",
            "send",
            "--to",
            &to,
            "--subject",
            &unique("aai-e2e-zoho-mail"),
            "--body",
            "created by aai-cli live e2e",
        ],
    );
    let id = find_string(&sent, &["messageId", "messageIdString", "id"])
        .or_else(|| find_u64(&sent, &["id"]).map(|id| id.to_string()))
        .expect("Zoho Mail send response missing message id");
    let _ = cli_required("AAI_E2E_ZOHO_MAIL_PROFILE", &["email", "messages", "list"]);
    let _ = cli_required(
        "AAI_E2E_ZOHO_MAIL_PROFILE",
        &["email", "messages", "get", &id],
    );
    let _ = cli_required(
        "AAI_E2E_ZOHO_MAIL_PROFILE",
        &["email", "messages", "delete", &id],
    );
}

#[test]
#[ignore = "requires live Google Calendar credentials"]
fn google_calendar_event_crud() {
    let created = cli_required(
        "AAI_E2E_GOOGLE_CALENDAR_PROFILE",
        &[
            "calendar",
            "events",
            "create",
            "--summary",
            &unique("aai-e2e-google-calendar"),
            "--description",
            "created by aai-cli live e2e",
            "--start",
            "2030-01-01T10:00:00Z",
            "--end",
            "2030-01-01T10:30:00Z",
        ],
    );
    let id = str_at(&created, &["id"]).to_string();
    let _ = cli_required(
        "AAI_E2E_GOOGLE_CALENDAR_PROFILE",
        &["calendar", "events", "list"],
    );
    let _ = cli_required(
        "AAI_E2E_GOOGLE_CALENDAR_PROFILE",
        &["calendar", "events", "get", &id],
    );
    let _ = cli_required(
        "AAI_E2E_GOOGLE_CALENDAR_PROFILE",
        &["calendar", "events", "delete", &id],
    );
}

#[test]
#[ignore = "requires live Zoho Calendar credentials"]
fn zoho_calendar_event_crud() {
    let created = cli_required(
        "AAI_E2E_ZOHO_CALENDAR_PROFILE",
        &[
            "calendar",
            "events",
            "create",
            "--summary",
            &unique("aai-e2e-zoho-calendar"),
            "--description",
            "created by aai-cli live e2e",
            "--start",
            "20300101T100000Z",
            "--end",
            "20300101T103000Z",
        ],
    );
    let id = find_string(&created, &["eventId", "uid", "id"])
        .expect("Zoho Calendar response missing event id");
    let _ = cli_required(
        "AAI_E2E_ZOHO_CALENDAR_PROFILE",
        &["calendar", "events", "list"],
    );
    let _ = cli_required(
        "AAI_E2E_ZOHO_CALENDAR_PROFILE",
        &["calendar", "events", "get", &id],
    );
    let _ = cli_required(
        "AAI_E2E_ZOHO_CALENDAR_PROFILE",
        &["calendar", "events", "delete", &id],
    );
}
