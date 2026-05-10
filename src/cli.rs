use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "aai-cli")]
#[command(about = "Agent-friendly CLI wrappers for common work APIs")]
pub struct Cli {
    #[arg(long, global = true, env = "AAI_PROFILE")]
    pub profile: Option<String>,
    #[arg(long, global = true, env = "AAI_CONFIG")]
    pub config: Option<String>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Jira(JiraCommand),
    Confluence(ConfluenceCommand),
    Bitbucket(BitbucketCommand),
    Github(GithubCommand),
    Email(EmailCommand),
    Calendar(CalendarCommand),
}

#[derive(Debug, Args)]
pub struct JiraCommand {
    #[command(subcommand)]
    pub resource: JiraResource,
}

#[derive(Debug, Subcommand)]
pub enum JiraResource {
    Issues(JiraIssuesCommand),
    Projects(JiraProjectsCommand),
}

#[derive(Debug, Args)]
pub struct JiraIssuesCommand {
    #[command(subcommand)]
    pub action: JiraIssuesAction,
}

#[derive(Debug, Subcommand)]
pub enum JiraIssuesAction {
    List(JiraIssueList),
    Get(IdArg),
    Create(JiraIssueCreate),
    Update(JiraIssueUpdate),
    Delete(IdArg),
}

#[derive(Debug, Args)]
pub struct JiraIssueList {
    #[arg(long)]
    pub jql: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct JiraIssueCreate {
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub project: Option<String>,
    #[arg(long)]
    pub issue_type: Option<String>,
    #[arg(long)]
    pub summary: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
}

#[derive(Debug, Args)]
pub struct JiraIssueUpdate {
    pub id: String,
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub summary: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
}

#[derive(Debug, Args)]
pub struct JiraProjectsCommand {
    #[command(subcommand)]
    pub action: ListGetAction,
}

#[derive(Debug, Args)]
pub struct ConfluenceCommand {
    #[command(subcommand)]
    pub resource: ConfluenceResource,
}

#[derive(Debug, Subcommand)]
pub enum ConfluenceResource {
    Spaces(ConfluenceSpacesCommand),
    Pages(ConfluencePagesCommand),
}

#[derive(Debug, Args)]
pub struct ConfluenceSpacesCommand {
    #[command(subcommand)]
    pub action: ListGetAction,
}

#[derive(Debug, Args)]
pub struct ConfluencePagesCommand {
    #[command(subcommand)]
    pub action: ConfluencePagesAction,
}

#[derive(Debug, Subcommand)]
pub enum ConfluencePagesAction {
    List(LimitArg),
    Get(IdArg),
    Create(ConfluencePageCreate),
    Update(ConfluencePageUpdate),
    Delete(IdArg),
}

#[derive(Debug, Args)]
pub struct ConfluencePageCreate {
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub space_id: Option<String>,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub body: Option<String>,
    #[arg(long)]
    pub parent_id: Option<String>,
}

#[derive(Debug, Args)]
pub struct ConfluencePageUpdate {
    pub id: String,
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub body: Option<String>,
    #[arg(long)]
    pub version: Option<u64>,
}

#[derive(Debug, Args)]
pub struct BitbucketCommand {
    #[command(subcommand)]
    pub resource: BitbucketResource,
}

#[derive(Debug, Subcommand)]
pub enum BitbucketResource {
    Repos(BitbucketReposCommand),
    Prs(BitbucketPrsCommand),
}

#[derive(Debug, Args)]
pub struct BitbucketReposCommand {
    #[command(subcommand)]
    pub action: ListGetAction,
}

#[derive(Debug, Args)]
pub struct BitbucketPrsCommand {
    #[command(subcommand)]
    pub action: BitbucketPullRequestAction,
}

#[derive(Debug, Subcommand)]
pub enum BitbucketPullRequestAction {
    List(RepoLimitArg),
    Get(NumberArg),
    Create(PullRequestCreate),
    Delete(NumberArg),
    Close(NumberArg),
    Decline(NumberArg),
    Comments(BitbucketPrCommentsCommand),
}

#[derive(Debug, Args)]
pub struct BitbucketPrCommentsCommand {
    #[command(subcommand)]
    pub action: BitbucketPrCommentAction,
}

#[derive(Debug, Subcommand)]
pub enum BitbucketPrCommentAction {
    List(BitbucketPrCommentList),
    Get(BitbucketPrCommentGet),
    Create(BitbucketPrCommentWrite),
    Update(BitbucketPrCommentWrite),
    Delete(BitbucketPrCommentGet),
}

#[derive(Debug, Args)]
pub struct BitbucketPrCommentList {
    pub pr: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct BitbucketPrCommentGet {
    pub pr: u64,
    pub comment: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
}

#[derive(Debug, Args)]
pub struct BitbucketPrCommentWrite {
    pub pr: u64,
    #[arg(long)]
    pub comment: Option<u64>,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub body: Option<String>,
}

#[derive(Debug, Args)]
pub struct GithubCommand {
    #[command(subcommand)]
    pub resource: GithubResource,
}

#[derive(Debug, Subcommand)]
pub enum GithubResource {
    Repos(GithubReposCommand),
    Issues(GithubIssuesCommand),
    Prs(GithubPrsCommand),
}

#[derive(Debug, Args)]
pub struct GithubReposCommand {
    #[command(subcommand)]
    pub action: GithubReposAction,
}

#[derive(Debug, Subcommand)]
pub enum GithubReposAction {
    List(LimitArg),
    Get(RepoArg),
}

#[derive(Debug, Args)]
pub struct GithubIssuesCommand {
    #[command(subcommand)]
    pub action: GithubIssuesAction,
}

#[derive(Debug, Subcommand)]
pub enum GithubIssuesAction {
    List(RepoLimitArg),
    Get(NumberArg),
    Create(GithubIssueCreate),
    Update(GithubIssueUpdate),
    Delete(NumberArg),
}

#[derive(Debug, Args)]
pub struct GithubIssueCreate {
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub body: Option<String>,
}

#[derive(Debug, Args)]
pub struct GithubIssueUpdate {
    pub number: u64,
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub body: Option<String>,
    #[arg(long)]
    pub state: Option<String>,
}

#[derive(Debug, Args)]
pub struct GithubPrsCommand {
    #[command(subcommand)]
    pub action: GithubPullRequestAction,
}

#[derive(Debug, Args)]
pub struct EmailCommand {
    #[command(subcommand)]
    pub resource: EmailResource,
}

#[derive(Debug, Subcommand)]
pub enum EmailResource {
    Messages(EmailMessagesCommand),
}

#[derive(Debug, Args)]
pub struct EmailMessagesCommand {
    #[command(subcommand)]
    pub action: EmailMessagesAction,
}

#[derive(Debug, Subcommand)]
pub enum EmailMessagesAction {
    List(LimitArg),
    Get(IdArg),
    Send(EmailSend),
    Delete(IdArg),
}

#[derive(Debug, Args)]
pub struct EmailSend {
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub to: Option<String>,
    #[arg(long)]
    pub subject: Option<String>,
    #[arg(long)]
    pub body: Option<String>,
}

#[derive(Debug, Args)]
pub struct CalendarCommand {
    #[command(subcommand)]
    pub resource: CalendarResource,
}

#[derive(Debug, Subcommand)]
pub enum CalendarResource {
    Events(CalendarEventsCommand),
}

#[derive(Debug, Args)]
pub struct CalendarEventsCommand {
    #[command(subcommand)]
    pub action: CalendarEventsAction,
}

#[derive(Debug, Subcommand)]
pub enum CalendarEventsAction {
    List(CalendarEventList),
    Get(IdArg),
    Create(CalendarEventCreate),
    Update(CalendarEventUpdate),
    Delete(CalendarEventDelete),
}

#[derive(Debug, Args)]
pub struct CalendarEventDelete {
    pub id: String,
    #[arg(long)]
    pub calendar_id: Option<String>,
}

#[derive(Debug, Args)]
pub struct CalendarEventList {
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
    #[arg(long)]
    pub calendar_id: Option<String>,
}

#[derive(Debug, Args)]
pub struct CalendarEventCreate {
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub calendar_id: Option<String>,
    #[arg(long)]
    pub summary: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub start: Option<String>,
    #[arg(long)]
    pub end: Option<String>,
}

#[derive(Debug, Args)]
pub struct CalendarEventUpdate {
    pub id: String,
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub calendar_id: Option<String>,
    #[arg(long)]
    pub summary: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub start: Option<String>,
    #[arg(long)]
    pub end: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum ListGetAction {
    List(LimitArg),
    Get(IdArg),
}

#[derive(Debug, Args)]
pub struct IdArg {
    pub id: String,
}

#[derive(Debug, Args)]
pub struct NumberArg {
    pub number: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
}

#[derive(Debug, Args)]
pub struct RepoArg {
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
}

#[derive(Debug, Args)]
pub struct LimitArg {
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct RepoLimitArg {
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Subcommand)]
pub enum PullRequestAction {
    List(RepoLimitArg),
    Get(NumberArg),
    Create(PullRequestCreate),
    Delete(NumberArg),
}

#[derive(Debug, Subcommand)]
pub enum GithubPullRequestAction {
    List(RepoLimitArg),
    Get(NumberArg),
    Create(PullRequestCreate),
    Delete(NumberArg),
    Close(NumberArg),
    Decline(NumberArg),
    Comments(BitbucketPrCommentsCommand),
}

#[derive(Debug, Args)]
pub struct PullRequestCreate {
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub body: Option<String>,
    #[arg(long)]
    pub source: Option<String>,
    #[arg(long)]
    pub destination: Option<String>,
    #[arg(long)]
    pub head: Option<String>,
    #[arg(long)]
    pub base: Option<String>,
}
