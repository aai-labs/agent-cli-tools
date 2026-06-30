use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "aai-cli")]
#[command(about = "Agent-friendly CLI wrappers for common work APIs")]
#[command(
    long_about = "Agent-friendly CLI wrappers for common work APIs.\n\nEvery successful service response includes `_aai.pagination` with pagination status, continuation values, and instructions for retrieving more results. Bare provider arrays are wrapped under `results` so this metadata is always visible."
)]
pub struct Cli {
    #[arg(long, global = true, env = "AAI_PROFILE")]
    pub profile: Option<String>,
    #[arg(long, global = true, env = "AAI_CONFIG")]
    pub config: Option<String>,
    #[arg(long, global = true, env = "AAI_SECRETS_FILE")]
    pub secrets_file: Option<String>,
    #[arg(long, global = true, env = "AAI_SECRET_KEY_FILE")]
    pub key_file: Option<String>,
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
    /// Manage Pipedrive CRM records, history, activities, notes, and synced email.
    Pipedrive(PipedriveCommand),
    /// Manage Apollo CRM, search, outreach, workflow, and analytics APIs.
    Apollo(ApolloCommand),
    /// Read and write Google Sheets spreadsheets and cell data.
    Sheets(SheetsCommand),
    /// Inspect and edit persistent profiles without exposing credentials.
    Config(ConfigCommand),
    Secrets(SecretsCommand),
}

#[derive(Debug, Args)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub resource: ConfigResource,
}

#[derive(Debug, Subcommand)]
pub enum ConfigResource {
    /// Inspect, create, update, validate, or remove profiles.
    Profiles(ConfigProfilesCommand),
    /// Inspect or change the configured default profile.
    DefaultProfile(ConfigDefaultProfileCommand),
}

#[derive(Debug, Args)]
pub struct ConfigProfilesCommand {
    #[command(subcommand)]
    pub action: ConfigProfilesAction,
}

#[derive(Debug, Subcommand)]
pub enum ConfigProfilesAction {
    List,
    Get(ConfigProfileName),
    Set(ConfigProfileSet),
    Remove(ConfigProfileName),
    Validate(ConfigProfileName),
}

#[derive(Debug, Args)]
pub struct ConfigProfileName {
    pub name: String,
}

#[derive(Debug, Args)]
pub struct ConfigProfileSet {
    pub name: String,
    /// JSON object, inline or from a path; use - to read stdin.
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub provider: Option<String>,
    #[arg(long)]
    pub auth_type: Option<String>,
    #[arg(long)]
    pub base_url: Option<String>,
    #[arg(long)]
    pub api_token_secret: Option<String>,
    #[arg(long)]
    pub token_secret: Option<String>,
    #[arg(long)]
    pub password_secret: Option<String>,
}

#[derive(Debug, Args)]
pub struct ConfigDefaultProfileCommand {
    #[command(subcommand)]
    pub action: ConfigDefaultProfileAction,
}

#[derive(Debug, Subcommand)]
pub enum ConfigDefaultProfileAction {
    Get,
    Set(ConfigProfileName),
}

#[derive(Debug, Args)]
pub struct SecretsCommand {
    #[command(subcommand)]
    pub action: SecretsAction,
}

#[derive(Debug, Subcommand)]
pub enum SecretsAction {
    Set(SecretSet),
    List,
    Remove(SecretKeyArg),
}

#[derive(Debug, Args)]
pub struct SecretSet {
    pub key: String,
    #[arg(long)]
    pub value: Option<String>,
}

#[derive(Debug, Args)]
pub struct SecretKeyArg {
    pub key: String,
}

#[derive(Debug, Args)]
pub struct GenericRequest {
    #[arg(value_enum)]
    pub method: GenericHttpMethod,
    /// Relative provider endpoint path. Absolute URLs are rejected.
    pub path: String,
    /// Query parameter as key=value. Repeat for multiple parameters.
    #[arg(long = "query")]
    pub query: Vec<String>,
    /// JSON body, inline or from a path; use - to read stdin.
    #[arg(long)]
    pub json: Option<String>,
    /// Required for POST, PUT, PATCH, and DELETE.
    #[arg(long)]
    pub allow_write: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum GenericHttpMethod {
    Get,
    Head,
    Post,
    Put,
    Patch,
    Delete,
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
    Sprints(JiraSprintsCommand),
    Boards(JiraBoardsCommand),
    Users(JiraUsersCommand),
    /// Call an uncommon Jira REST endpoint with profile authentication.
    Request(GenericRequest),
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
    Comments(JiraIssueCommentsCommand),
    Attachments(JiraIssueAttachmentsCommand),
}

#[derive(Debug, Args)]
pub struct JiraIssueCommentsCommand {
    #[command(subcommand)]
    pub action: JiraIssueCommentsAction,
}

#[derive(Debug, Subcommand)]
pub enum JiraIssueCommentsAction {
    List(JiraIssueCommentsList),
    Get(JiraIssueCommentsGet),
    Create(JiraIssueCommentsCreate),
}

#[derive(Debug, Args)]
pub struct JiraIssueCommentsList {
    pub issue: String,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct JiraIssueCommentsGet {
    pub issue: String,
    pub comment: String,
}

#[derive(Debug, Args)]
pub struct JiraIssueCommentsCreate {
    pub issue: String,
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub body: Option<String>,
}

#[derive(Debug, Args)]
pub struct JiraIssueAttachmentsCommand {
    #[command(subcommand)]
    pub action: JiraIssueAttachmentsAction,
}

#[derive(Debug, Subcommand)]
pub enum JiraIssueAttachmentsAction {
    List(JiraIssueAttachmentsList),
    Download(JiraAttachmentDownload),
    Upload(JiraAttachmentUpload),
}

#[derive(Debug, Args)]
pub struct JiraIssueAttachmentsList {
    pub issue: String,
}

#[derive(Debug, Args)]
pub struct JiraAttachmentDownload {
    pub attachment_id: String,
    #[arg(long)]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct JiraAttachmentUpload {
    pub issue: String,
    #[arg(long)]
    pub file: String,
}

#[derive(Debug, Args)]
pub struct JiraIssueList {
    #[arg(long)]
    pub project: Option<String>,
    #[arg(long)]
    pub status: Option<String>,
    #[arg(long)]
    pub assignee: Option<String>,
    #[arg(long = "type")]
    pub issue_type: Option<String>,
    #[arg(long)]
    pub sprint: Option<String>,
    #[arg(long)]
    pub text: Option<String>,
    #[arg(long = "updated-since")]
    pub updated_since: Option<String>,
    #[arg(long)]
    pub fields: Option<String>,
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
pub struct JiraSprintsCommand {
    #[command(subcommand)]
    pub action: JiraSprintsAction,
}

#[derive(Debug, Subcommand)]
pub enum JiraSprintsAction {
    List(JiraSprintsList),
    Get(JiraSprintsGet),
    Create(JiraSprintsCreate),
    Issues(JiraSprintsIssuesCommand),
}

#[derive(Debug, Args)]
pub struct JiraSprintsIssuesCommand {
    #[command(subcommand)]
    pub action: JiraSprintsIssuesAction,
}

#[derive(Debug, Subcommand)]
pub enum JiraSprintsIssuesAction {
    Add(JiraSprintsIssuesAdd),
}

#[derive(Debug, Args)]
pub struct JiraSprintsIssuesAdd {
    pub sprint: u64,
    #[arg(long)]
    pub issues: String,
}

#[derive(Debug, Args)]
pub struct JiraSprintsList {
    #[arg(long)]
    pub board: u64,
    #[arg(long)]
    pub state: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct JiraSprintsGet {
    pub id: u64,
}

#[derive(Debug, Args)]
pub struct JiraSprintsCreate {
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub board: Option<u64>,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub goal: Option<String>,
    #[arg(long = "start-date")]
    pub start_date: Option<String>,
    #[arg(long = "end-date")]
    pub end_date: Option<String>,
}

#[derive(Debug, Args)]
pub struct JiraBoardsCommand {
    #[command(subcommand)]
    pub action: JiraBoardsAction,
}

#[derive(Debug, Subcommand)]
pub enum JiraBoardsAction {
    List(JiraBoardsList),
    Get(JiraBoardsGet),
}

#[derive(Debug, Args)]
pub struct JiraBoardsList {
    #[arg(long = "type")]
    pub board_type: Option<String>,
    #[arg(long)]
    pub project: Option<String>,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct JiraBoardsGet {
    pub id: u64,
}

#[derive(Debug, Args)]
pub struct JiraUsersCommand {
    #[command(subcommand)]
    pub action: JiraUsersAction,
}

#[derive(Debug, Subcommand)]
pub enum JiraUsersAction {
    Get(JiraUsersGet),
}

#[derive(Debug, Args)]
pub struct JiraUsersGet {
    pub account_id: String,
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
    /// Call an uncommon Confluence REST endpoint with profile authentication.
    Request(GenericRequest),
}

#[derive(Debug, Args)]
pub struct ConfluenceSpacesCommand {
    #[command(subcommand)]
    pub action: ConfluenceSpacesAction,
}

#[derive(Debug, Subcommand)]
pub enum ConfluenceSpacesAction {
    List(ConfluenceSpacesList),
    Get(IdArg),
}

#[derive(Debug, Args)]
pub struct ConfluenceSpacesList {
    #[arg(long = "type")]
    pub space_type: Option<String>,
    #[arg(long)]
    pub status: Option<String>,
    #[arg(long)]
    pub key: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct ConfluencePagesCommand {
    #[command(subcommand)]
    pub action: ConfluencePagesAction,
}

#[derive(Debug, Subcommand)]
pub enum ConfluencePagesAction {
    List(ConfluencePagesList),
    Get(IdArg),
    Create(ConfluencePageCreate),
    Update(ConfluencePageUpdate),
    Move(ConfluencePageMove),
    Delete(IdArg),
    Comments(ConfluencePageCommentsCommand),
    Attachments(ConfluencePageAttachmentsCommand),
}

#[derive(Debug, Args)]
pub struct ConfluencePageCommentsCommand {
    #[command(subcommand)]
    pub action: ConfluencePageCommentsAction,
}

#[derive(Debug, Subcommand)]
pub enum ConfluencePageCommentsAction {
    List(ConfluencePageCommentsList),
    Create(ConfluencePageCommentsCreate),
}

#[derive(Debug, Args)]
pub struct ConfluencePageCommentsList {
    pub page_id: String,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct ConfluencePageCommentsCreate {
    pub page_id: String,
    #[arg(long)]
    pub body: Option<String>,
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long = "reply-to")]
    pub reply_to: Option<String>,
}

#[derive(Debug, Args)]
pub struct ConfluencePageAttachmentsCommand {
    #[command(subcommand)]
    pub action: ConfluencePageAttachmentsAction,
}

#[derive(Debug, Subcommand)]
pub enum ConfluencePageAttachmentsAction {
    List(ConfluencePageAttachmentsList),
    Download(ConfluencePageAttachmentsDownload),
    Upload(ConfluencePageAttachmentsUpload),
}

#[derive(Debug, Args)]
pub struct ConfluencePageAttachmentsList {
    pub page_id: String,
    #[arg(long, default_value_t = 25)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct ConfluencePageAttachmentsDownload {
    pub page_id: String,
    pub attachment_id: String,
    #[arg(long)]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct ConfluencePageAttachmentsUpload {
    pub page_id: String,
    #[arg(long)]
    pub file: String,
    #[arg(long)]
    pub comment: Option<String>,
}

#[derive(Debug, Args)]
pub struct ConfluencePagesList {
    #[arg(long)]
    pub space: Option<String>,
    #[arg(long)]
    pub status: Option<String>,
    #[arg(long)]
    pub parent: Option<String>,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct ConfluencePageCreate {
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub space_id: Option<String>,
    #[arg(long)]
    pub space_key: Option<String>,
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
pub struct ConfluencePageMove {
    pub id: String,
    #[arg(long)]
    pub target_id: String,
    #[arg(long, value_enum, default_value_t = ConfluenceMovePosition::Append)]
    pub position: ConfluenceMovePosition,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum ConfluenceMovePosition {
    Before,
    After,
    Append,
}

impl ConfluenceMovePosition {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Before => "before",
            Self::After => "after",
            Self::Append => "append",
        }
    }
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
    Branches(BitbucketBranchesCommand),
    Commits(BitbucketCommitsCommand),
    Source(BitbucketSourceCommand),
    Pipelines(BitbucketPipelinesCommand),
    /// Call an uncommon Bitbucket REST endpoint with profile authentication.
    Request(GenericRequest),
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
    Diff(BitbucketPrDiff),
    Diffstat(BitbucketPrDiffstat),
    Commits(BitbucketPrCommits),
    Activity(BitbucketPrActivity),
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
    #[arg(long)]
    pub inline_only: bool,
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
    #[arg(long)]
    pub inline_path: Option<String>,
    #[arg(long)]
    pub inline_from: Option<u64>,
    #[arg(long)]
    pub inline_to: Option<u64>,
    #[arg(long)]
    pub parent_id: Option<u64>,
}

#[derive(Debug, Args)]
pub struct BitbucketPrDiff {
    pub pr: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub output: Option<String>,
}

#[derive(Debug, Args)]
pub struct BitbucketPrDiffstat {
    pub pr: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct BitbucketPrCommits {
    pub pr: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct BitbucketPrActivity {
    pub pr: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct BitbucketBranchesCommand {
    #[command(subcommand)]
    pub action: BitbucketBranchesAction,
}

#[derive(Debug, Subcommand)]
pub enum BitbucketBranchesAction {
    List(BitbucketBranchList),
    Get(BitbucketBranchGet),
}

#[derive(Debug, Args)]
pub struct BitbucketBranchList {
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
    #[arg(long, conflicts_with_all = ["name_prefix", "query"])]
    pub name_contains: Option<String>,
    #[arg(long, conflicts_with_all = ["name_contains", "query"])]
    pub name_prefix: Option<String>,
    #[arg(
        long,
        hide = true,
        help = "Advanced escape hatch: raw Bitbucket BBQL expression for the ?q= filter (e.g. 'name ~ \"^feature/\"'). Prefer --name-contains or --name-prefix."
    )]
    pub query: Option<String>,
}

#[derive(Debug, Args)]
pub struct BitbucketBranchGet {
    pub name: String,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
}

#[derive(Debug, Args)]
pub struct BitbucketCommitsCommand {
    #[command(subcommand)]
    pub action: BitbucketCommitsAction,
}

#[derive(Debug, Subcommand)]
pub enum BitbucketCommitsAction {
    List(BitbucketCommitList),
    Get(BitbucketCommitGet),
}

#[derive(Debug, Args)]
pub struct BitbucketCommitList {
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
    #[arg(long)]
    pub branch: Option<String>,
    #[arg(long)]
    pub include: Option<String>,
    #[arg(long)]
    pub exclude: Option<String>,
}

#[derive(Debug, Args)]
pub struct BitbucketCommitGet {
    pub sha: String,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
}

#[derive(Debug, Args)]
pub struct BitbucketSourceCommand {
    #[command(subcommand)]
    pub action: BitbucketSourceAction,
}

#[derive(Debug, Subcommand)]
pub enum BitbucketSourceAction {
    Get(BitbucketSourceGet),
    History(BitbucketSourceHistory),
}

#[derive(Debug, Args)]
pub struct BitbucketSourceGet {
    pub commit: String,
    pub path: String,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, conflicts_with = "meta")]
    pub output: Option<String>,
    #[arg(long, conflicts_with = "output")]
    pub meta: bool,
}

#[derive(Debug, Args)]
pub struct BitbucketSourceHistory {
    pub commit: String,
    pub path: String,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct BitbucketPipelinesCommand {
    #[command(subcommand)]
    pub action: BitbucketPipelinesAction,
}

#[derive(Debug, Subcommand)]
pub enum BitbucketPipelinesAction {
    List(BitbucketPipelineList),
    Get(BitbucketPipelineGet),
    Steps(BitbucketPipelineStepsCommand),
}

#[derive(Debug, Args)]
pub struct BitbucketPipelineList {
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
    #[arg(long)]
    pub branch: Option<String>,
    #[arg(long)]
    pub status: Option<String>,
    #[arg(long)]
    pub sort: Option<String>,
}

#[derive(Debug, Args)]
pub struct BitbucketPipelineGet {
    pub pipeline: String,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
}

#[derive(Debug, Args)]
pub struct BitbucketPipelineStepsCommand {
    #[command(subcommand)]
    pub action: BitbucketPipelineStepsAction,
}

#[derive(Debug, Subcommand)]
pub enum BitbucketPipelineStepsAction {
    List(BitbucketPipelineStepList),
    Get(BitbucketPipelineStepGet),
    Logs(BitbucketPipelineStepLogsCommand),
}

#[derive(Debug, Args)]
pub struct BitbucketPipelineStepList {
    pub pipeline: String,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
}

#[derive(Debug, Args)]
pub struct BitbucketPipelineStepGet {
    pub pipeline: String,
    pub step: String,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
}

#[derive(Debug, Args)]
pub struct BitbucketPipelineStepLogsCommand {
    #[command(subcommand)]
    pub action: BitbucketPipelineStepLogsAction,
}

#[derive(Debug, Subcommand)]
pub enum BitbucketPipelineStepLogsAction {
    Download(BitbucketPipelineStepLogDownload),
}

#[derive(Debug, Args)]
pub struct BitbucketPipelineStepLogDownload {
    pub pipeline: String,
    pub step: String,
    #[arg(long)]
    pub log: Option<String>,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub output: String,
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
    Actions(GithubActionsCommand),
    Branches(GithubBranchesCommand),
    Source(GithubSourceCommand),
    /// Call an uncommon GitHub REST endpoint with profile authentication.
    Request(GenericRequest),
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
pub struct GithubActionsCommand {
    #[command(subcommand)]
    pub resource: GithubActionsResource,
}

#[derive(Debug, Subcommand)]
pub enum GithubActionsResource {
    Runs(GithubActionsRunsCommand),
    Jobs(GithubActionsJobsCommand),
}

#[derive(Debug, Args)]
pub struct GithubActionsRunsCommand {
    #[command(subcommand)]
    pub action: GithubActionsRunsAction,
}

#[derive(Debug, Subcommand)]
pub enum GithubActionsRunsAction {
    List(GithubActionsRunList),
    Get(GithubActionsRunGet),
    Logs(GithubActionsRunLogsCommand),
}

#[derive(Debug, Args)]
pub struct GithubActionsRunList {
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
    #[arg(long)]
    pub branch: Option<String>,
    #[arg(long)]
    pub status: Option<String>,
    #[arg(long)]
    pub event: Option<String>,
}

#[derive(Debug, Args)]
pub struct GithubActionsRunGet {
    pub run: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
}

#[derive(Debug, Args)]
pub struct GithubActionsRunLogsCommand {
    #[command(subcommand)]
    pub action: GithubActionsRunLogsAction,
}

#[derive(Debug, Subcommand)]
pub enum GithubActionsRunLogsAction {
    Download(GithubActionsRunLogDownload),
}

#[derive(Debug, Args)]
pub struct GithubActionsRunLogDownload {
    pub run: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct GithubActionsJobsCommand {
    #[command(subcommand)]
    pub action: GithubActionsJobsAction,
}

#[derive(Debug, Subcommand)]
pub enum GithubActionsJobsAction {
    List(GithubActionsJobList),
    Get(GithubActionsJobGet),
    Logs(GithubActionsJobLogsCommand),
}

#[derive(Debug, Args)]
pub struct GithubActionsJobList {
    pub run: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
    #[arg(long)]
    pub all_attempts: bool,
}

#[derive(Debug, Args)]
pub struct GithubActionsJobGet {
    pub job: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
}

#[derive(Debug, Args)]
pub struct GithubActionsJobLogsCommand {
    #[command(subcommand)]
    pub action: GithubActionsJobLogsAction,
}

#[derive(Debug, Subcommand)]
pub enum GithubActionsJobLogsAction {
    Download(GithubActionsJobLogDownload),
}

#[derive(Debug, Args)]
pub struct GithubActionsJobLogDownload {
    pub job: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct EmailCommand {
    #[command(subcommand)]
    pub resource: EmailResource,
}

#[derive(Debug, Subcommand)]
pub enum EmailResource {
    Messages(EmailMessagesCommand),
    /// Call an uncommon REST email endpoint. SMTP/IMAP profiles are rejected.
    Request(GenericRequest),
}

#[derive(Debug, Args)]
pub struct EmailMessagesCommand {
    #[command(subcommand)]
    pub action: EmailMessagesAction,
}

#[derive(Debug, Subcommand)]
pub enum EmailMessagesAction {
    List(EmailMessageList),
    Get(IdArg),
    Send(EmailSend),
    Delete(IdArg),
}

#[derive(Debug, Args)]
pub struct EmailMessageList {
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
    #[arg(long)]
    pub received_after: Option<String>,
    #[arg(long)]
    pub received_before: Option<String>,
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
    /// Call an uncommon REST calendar endpoint. CalDAV profiles are rejected.
    Request(GenericRequest),
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

#[derive(Debug, Args)]
#[command(
    about = "Manage Pipedrive CRM records and communication history",
    long_about = "Manage Pipedrive leads, persons, organizations, deals, labels, activities, notes, and synced email history.\n\nUse `deals view`, `persons view`, or `organizations view` for a combined record, activities, and notes response. Add `--include-mail` to include associated email history.",
    after_help = "Examples:\n  aai-cli pipedrive deals view 123 --include-mail\n  aai-cli pipedrive persons activities 456 --limit 25\n  aai-cli pipedrive notes list --deal-id 123\n  aai-cli pipedrive mailbox messages get 789 --include-body"
)]
pub struct PipedriveCommand {
    #[command(subcommand)]
    pub resource: PipedriveResource,
}

#[derive(Debug, Subcommand)]
pub enum PipedriveResource {
    /// Manage leads in the Leads Inbox.
    Leads(PipedriveLeadsCommand),
    /// Manage persons (contacts) and inspect their full CRM history.
    Persons(PipedrivePersonsCommand),
    /// Manage organizations and inspect their full CRM history.
    Organizations(PipedriveOrganizationsCommand),
    /// Manage deals and inspect their full CRM history.
    Deals(PipedriveDealsCommand),
    /// List and manage CRM labels.
    Labels(PipedriveLabelsCommand),
    /// List or get activities across CRM records.
    Activities(PipedriveActivitiesCommand),
    /// List or get notes across CRM records.
    Notes(PipedriveNotesCommand),
    /// Inspect synced email messages and threads.
    Mailbox(PipedriveMailboxCommand),
    /// Call an uncommon Pipedrive REST endpoint with profile authentication.
    Request(GenericRequest),
}

#[derive(Debug, Args)]
pub struct PipedriveLeadsCommand {
    #[command(subcommand)]
    pub action: PipedriveLeadsAction,
}

#[derive(Debug, Subcommand)]
pub enum PipedriveLeadsAction {
    List(PipedriveLeadList),
    Search(PipedriveLeadSearch),
    Get(PipedriveIdArg),
    Create(PipedriveLeadWrite),
    Update(PipedriveLeadUpdate),
    Delete(PipedriveIdArg),
    Convert(PipedriveLeadConvert),
}

#[derive(Debug, Args)]
pub struct PipedriveLeadList {
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
    #[arg(long)]
    pub owner_id: Option<String>,
    #[arg(long)]
    pub person_id: Option<String>,
    #[arg(long)]
    pub organization_id: Option<String>,
    #[arg(long)]
    pub filter_id: Option<String>,
    #[arg(long)]
    pub updated_since: Option<String>,
    #[arg(long)]
    pub sort: Option<String>,
    #[arg(long)]
    pub archived: bool,
}

#[derive(Debug, Args)]
pub struct PipedriveLeadSearch {
    #[arg(long)]
    pub term: String,
    #[arg(long)]
    pub fields: Option<String>,
    #[arg(long)]
    pub exact_match: bool,
    #[arg(long)]
    pub person_id: Option<String>,
    #[arg(long)]
    pub organization_id: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct PipedriveLeadWrite {
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub title: String,
    #[arg(long)]
    pub person_id: Option<String>,
    #[arg(long)]
    pub organization_id: Option<String>,
    #[arg(long)]
    pub label_ids: Option<String>,
}

#[derive(Debug, Args)]
pub struct PipedriveLeadUpdate {
    pub id: String,
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub person_id: Option<String>,
    #[arg(long)]
    pub organization_id: Option<String>,
    #[arg(long)]
    pub label_ids: Option<String>,
}

#[derive(Debug, Args)]
pub struct PipedriveLeadConvert {
    pub id: String,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Debug, Args)]
pub struct PipedrivePersonsCommand {
    #[command(subcommand)]
    pub action: PipedrivePersonsAction,
}

#[derive(Debug, Subcommand)]
pub enum PipedrivePersonsAction {
    /// List persons with optional filters.
    List(PipedrivePersonList),
    /// Search persons by text.
    Search(PipedrivePersonSearch),
    /// Get one person record.
    Get(PipedriveGetWithLabels),
    /// Get a person with activities, notes, and optional associated email.
    View(PipedriveAssociatedView),
    /// List activities associated with a person.
    Activities(PipedriveAssociatedList),
    /// List notes associated with a person.
    Notes(PipedriveAssociatedList),
    /// List synced email messages associated with a person.
    MailMessages(PipedriveAssociatedList),
    /// Create a person from flags and/or JSON.
    Create(PipedrivePersonWrite),
    /// Update a person from flags and/or JSON.
    Update(PipedrivePersonUpdate),
    /// Delete a person.
    Delete(PipedriveIdArg),
}

#[derive(Debug, Args)]
pub struct PipedrivePersonList {
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
    #[arg(long)]
    pub filter_id: Option<String>,
    #[arg(long)]
    pub ids: Option<String>,
    #[arg(long)]
    pub owner_id: Option<String>,
    #[arg(long)]
    pub org_id: Option<String>,
    #[arg(long)]
    pub deal_id: Option<String>,
    #[arg(long)]
    pub updated_since: Option<String>,
    #[arg(long)]
    pub updated_until: Option<String>,
    #[arg(long)]
    pub sort_by: Option<String>,
    #[arg(long, value_enum)]
    pub sort_direction: Option<PipedriveSortDirection>,
    #[arg(long)]
    pub include_labels: bool,
}

#[derive(Debug, Args)]
pub struct PipedrivePersonSearch {
    #[arg(long)]
    pub term: String,
    #[arg(long)]
    pub fields: Option<String>,
    #[arg(long)]
    pub exact_match: bool,
    #[arg(long)]
    pub organization_id: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct PipedrivePersonWrite {
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub org_id: Option<String>,
    #[arg(long)]
    pub email: Option<String>,
    #[arg(long)]
    pub phone: Option<String>,
    #[arg(long)]
    pub label_ids: Option<String>,
}

#[derive(Debug, Args)]
pub struct PipedrivePersonUpdate {
    pub id: String,
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub org_id: Option<String>,
    #[arg(long)]
    pub email: Option<String>,
    #[arg(long)]
    pub phone: Option<String>,
    #[arg(long)]
    pub label_ids: Option<String>,
}

#[derive(Debug, Args)]
pub struct PipedriveOrganizationsCommand {
    #[command(subcommand)]
    pub action: PipedriveOrganizationsAction,
}

#[derive(Debug, Subcommand)]
pub enum PipedriveOrganizationsAction {
    /// List organizations with optional filters.
    List(PipedriveOrganizationList),
    /// Search organizations by text.
    Search(PipedriveOrganizationSearch),
    /// Get one organization record.
    Get(PipedriveGetWithLabels),
    /// Get an organization with activities, notes, and optional associated email.
    View(PipedriveAssociatedView),
    /// List activities associated with an organization.
    Activities(PipedriveAssociatedList),
    /// List notes associated with an organization.
    Notes(PipedriveAssociatedList),
    /// List synced email messages associated with an organization.
    MailMessages(PipedriveAssociatedList),
    /// Create an organization from flags and/or JSON.
    Create(PipedriveOrganizationWrite),
    /// Update an organization from flags and/or JSON.
    Update(PipedriveOrganizationUpdate),
    /// Delete an organization.
    Delete(PipedriveIdArg),
}

#[derive(Debug, Args)]
pub struct PipedriveOrganizationList {
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
    #[arg(long)]
    pub filter_id: Option<String>,
    #[arg(long)]
    pub ids: Option<String>,
    #[arg(long)]
    pub owner_id: Option<String>,
    #[arg(long)]
    pub updated_since: Option<String>,
    #[arg(long)]
    pub updated_until: Option<String>,
    #[arg(long)]
    pub sort_by: Option<String>,
    #[arg(long, value_enum)]
    pub sort_direction: Option<PipedriveSortDirection>,
    #[arg(long)]
    pub include_labels: bool,
}

#[derive(Debug, Args)]
pub struct PipedriveOrganizationSearch {
    #[arg(long)]
    pub term: String,
    #[arg(long)]
    pub fields: Option<String>,
    #[arg(long)]
    pub exact_match: bool,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct PipedriveOrganizationWrite {
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub address: Option<String>,
    #[arg(long)]
    pub label_ids: Option<String>,
}

#[derive(Debug, Args)]
pub struct PipedriveOrganizationUpdate {
    pub id: String,
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub address: Option<String>,
    #[arg(long)]
    pub label_ids: Option<String>,
}

#[derive(Debug, Args)]
pub struct PipedriveDealsCommand {
    #[command(subcommand)]
    pub action: PipedriveDealsAction,
}

#[derive(Debug, Subcommand)]
pub enum PipedriveDealsAction {
    /// List deals with optional filters.
    List(PipedriveDealList),
    /// Search deals by text.
    Search(PipedriveDealSearch),
    /// Get one deal record.
    Get(PipedriveGetWithLabels),
    /// Get a deal with activities, notes, and optional associated email.
    View(PipedriveAssociatedView),
    /// List activities associated with a deal.
    Activities(PipedriveAssociatedList),
    /// List notes associated with a deal.
    Notes(PipedriveAssociatedList),
    /// List synced email messages associated with a deal.
    MailMessages(PipedriveAssociatedList),
    /// Create a deal from flags and/or JSON.
    Create(PipedriveDealWrite),
    /// Update a deal from flags and/or JSON.
    Update(PipedriveDealUpdate),
    /// Delete a deal.
    Delete(PipedriveIdArg),
}

#[derive(Debug, Args)]
pub struct PipedriveDealList {
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
    #[arg(long)]
    pub filter_id: Option<String>,
    #[arg(long)]
    pub ids: Option<String>,
    #[arg(long)]
    pub owner_id: Option<String>,
    #[arg(long)]
    pub person_id: Option<String>,
    #[arg(long)]
    pub org_id: Option<String>,
    #[arg(long)]
    pub pipeline_id: Option<String>,
    #[arg(long)]
    pub stage_id: Option<String>,
    #[arg(long, value_enum)]
    pub status: Option<PipedriveDealStatus>,
    #[arg(long)]
    pub updated_since: Option<String>,
    #[arg(long)]
    pub updated_until: Option<String>,
    #[arg(long)]
    pub sort_by: Option<String>,
    #[arg(long, value_enum)]
    pub sort_direction: Option<PipedriveSortDirection>,
    #[arg(long)]
    pub include_labels: bool,
}

#[derive(Debug, Args)]
pub struct PipedriveDealSearch {
    #[arg(long)]
    pub term: String,
    #[arg(long)]
    pub fields: Option<String>,
    #[arg(long)]
    pub exact_match: bool,
    #[arg(long)]
    pub person_id: Option<String>,
    #[arg(long)]
    pub organization_id: Option<String>,
    #[arg(long, value_enum)]
    pub status: Option<PipedriveSearchDealStatus>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct PipedriveDealWrite {
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub title: String,
    #[arg(long)]
    pub person_id: Option<String>,
    #[arg(long)]
    pub org_id: Option<String>,
    #[arg(long)]
    pub value: Option<f64>,
    #[arg(long)]
    pub currency: Option<String>,
    #[arg(long)]
    pub pipeline_id: Option<String>,
    #[arg(long)]
    pub stage_id: Option<String>,
    #[arg(long)]
    pub label_ids: Option<String>,
}

#[derive(Debug, Args)]
pub struct PipedriveDealUpdate {
    pub id: String,
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub person_id: Option<String>,
    #[arg(long)]
    pub org_id: Option<String>,
    #[arg(long)]
    pub value: Option<f64>,
    #[arg(long)]
    pub currency: Option<String>,
    #[arg(long)]
    pub pipeline_id: Option<String>,
    #[arg(long)]
    pub stage_id: Option<String>,
    #[arg(long)]
    pub label_ids: Option<String>,
}

#[derive(Debug, Args)]
pub struct PipedriveLabelsCommand {
    #[command(subcommand)]
    pub resource: PipedriveLabelResource,
}

#[derive(Debug, Subcommand)]
pub enum PipedriveLabelResource {
    Leads(PipedriveLeadLabelsCommand),
    Deals(PipedriveLabelListCommand),
    Persons(PipedriveLabelListCommand),
    Organizations(PipedriveLabelListCommand),
}

#[derive(Debug, Args)]
pub struct PipedriveLeadLabelsCommand {
    #[command(subcommand)]
    pub action: PipedriveLeadLabelsAction,
}

#[derive(Debug, Subcommand)]
pub enum PipedriveLeadLabelsAction {
    List,
    Create(PipedriveLabelCreate),
    Update(PipedriveLabelUpdate),
    Delete(PipedriveIdArg),
}

#[derive(Debug, Args)]
pub struct PipedriveLabelListCommand {
    #[command(subcommand)]
    pub action: PipedriveLabelListAction,
}

#[derive(Debug, Subcommand)]
pub enum PipedriveLabelListAction {
    List,
}

#[derive(Debug, Args)]
pub struct PipedriveLabelCreate {
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub color: String,
}

#[derive(Debug, Args)]
pub struct PipedriveLabelUpdate {
    pub id: String,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub color: Option<String>,
}

#[derive(Debug, Args)]
pub struct PipedriveIdArg {
    pub id: String,
}

#[derive(Debug, Args)]
pub struct PipedriveAssociatedList {
    /// Pipedrive record ID.
    pub id: String,
    /// Maximum associated records to aggregate.
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
#[command(
    about = "Manage Apollo CRM, search, outreach, workflow, and analytics APIs",
    long_about = "Manage Apollo people, organizations, contacts, accounts, deals, tasks, calls, sequences, emails, conversations, and analytics.\n\nCreate, update, search, and bulk commands accept `--json <inline|path|->`. Typed flags set common fields and override matching JSON fields.",
    after_help = "Examples:\n  aai-cli apollo people search --title CEO --location Berlin --limit 10\n  aai-cli apollo contacts create --email ada@example.com --first-name Ada --last-name Lovelace\n  aai-cli apollo users me\n  aai-cli apollo request get /users/api_profile"
)]
pub struct ApolloCommand {
    #[command(subcommand)]
    pub resource: ApolloResource,
}

#[derive(Debug, Subcommand)]
pub enum ApolloResource {
    /// Test Apollo API-key authentication.
    Health,
    /// Manage people search and enrichment.
    People(ApolloPeopleCommand),
    /// Manage organization search and enrichment.
    Organizations(ApolloOrganizationsCommand),
    /// Manage Apollo contacts.
    Contacts(ApolloContactsCommand),
    /// Manage Apollo accounts.
    Accounts(ApolloAccountsCommand),
    /// Manage Apollo deals.
    Deals(ApolloDealsCommand),
    /// Manage Apollo tasks.
    Tasks(ApolloTasksCommand),
    /// Manage Apollo call records.
    Calls(ApolloCallsCommand),
    /// List Apollo notes.
    Notes(ApolloNotesCommand),
    /// List users and current API profile.
    Users(ApolloUsersCommand),
    /// List Apollo labels.
    Labels(ApolloListCommand),
    /// List or create Apollo fields.
    Fields(ApolloFieldsCommand),
    /// List Apollo typed custom fields.
    CustomFields(ApolloListCommand),
    /// Inspect Apollo API usage.
    Usage(ApolloUsageCommand),
    /// Poll Apollo webhook results.
    Webhooks(ApolloWebhooksCommand),
    /// Query Apollo analytics reports.
    Analytics(ApolloAnalyticsCommand),
    /// Manage Apollo sequences.
    Sequences(ApolloSequencesCommand),
    /// Manage Apollo outreach emails.
    Emails(ApolloEmailsCommand),
    /// Search Apollo news articles.
    News(ApolloNewsCommand),
    /// Search, inspect, and export Apollo conversations.
    Conversations(ApolloConversationsCommand),
    /// Call an Apollo REST endpoint with profile authentication.
    Request(GenericRequest),
}

#[derive(Debug, Args)]
pub struct ApolloPeopleCommand {
    #[command(subcommand)]
    pub action: ApolloPeopleAction,
}

#[derive(Debug, Subcommand)]
pub enum ApolloPeopleAction {
    Search(ApolloSearchArgs),
    Get(ApolloIdArg),
    Enrich(ApolloPeopleEnrich),
    BulkEnrich(ApolloJsonArgs),
}

#[derive(Debug, Args)]
pub struct ApolloPeopleEnrich {
    #[command(flatten)]
    pub json: ApolloJsonArgs,
    #[arg(long)]
    pub first_name: Option<String>,
    #[arg(long)]
    pub last_name: Option<String>,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub email: Option<String>,
    #[arg(long)]
    pub organization_name: Option<String>,
    #[arg(long)]
    pub domain: Option<String>,
    #[arg(long)]
    pub id: Option<String>,
    #[arg(long)]
    pub linkedin_url: Option<String>,
    #[arg(long)]
    pub reveal_personal_emails: bool,
    #[arg(long)]
    pub reveal_phone_number: bool,
}

#[derive(Debug, Args)]
pub struct ApolloOrganizationsCommand {
    #[command(subcommand)]
    pub action: ApolloOrganizationsAction,
}

#[derive(Debug, Subcommand)]
pub enum ApolloOrganizationsAction {
    Search(ApolloSearchArgs),
    Get(ApolloIdArg),
    Enrich(ApolloOrganizationEnrich),
    BulkEnrich(ApolloJsonArgs),
    JobPostings(ApolloPagedIdArg),
}

#[derive(Debug, Args)]
pub struct ApolloOrganizationEnrich {
    #[arg(long)]
    pub domain: Option<String>,
    #[arg(long)]
    pub linkedin_url: Option<String>,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub website: Option<String>,
}

#[derive(Debug, Args)]
pub struct ApolloContactsCommand {
    #[command(subcommand)]
    pub action: ApolloContactsAction,
}

#[derive(Debug, Subcommand)]
pub enum ApolloContactsAction {
    Create(ApolloContactWrite),
    Get(ApolloIdArg),
    Search(ApolloSearchArgs),
    Update(ApolloContactUpdate),
    BulkCreate(ApolloJsonArgs),
    BulkUpdate(ApolloJsonArgs),
    UpdateStages(ApolloIdsUpdate),
    UpdateOwners(ApolloIdsUpdate),
    Deals(ApolloIdJsonArgs),
}

#[derive(Debug, Args)]
pub struct ApolloAccountsCommand {
    #[command(subcommand)]
    pub action: ApolloAccountsAction,
}

#[derive(Debug, Subcommand)]
pub enum ApolloAccountsAction {
    Create(ApolloAccountWrite),
    Get(ApolloIdArg),
    Search(ApolloSearchArgs),
    Update(ApolloAccountUpdate),
    BulkCreate(ApolloJsonArgs),
    BulkUpdate(ApolloJsonArgs),
    UpdateOwners(ApolloIdsUpdate),
    Stages,
}

#[derive(Debug, Args)]
pub struct ApolloDealsCommand {
    #[command(subcommand)]
    pub action: ApolloDealsAction,
}

#[derive(Debug, Subcommand)]
pub enum ApolloDealsAction {
    Create(ApolloDealWrite),
    List(ApolloSearchArgs),
    Get(ApolloIdArg),
    Update(ApolloDealUpdate),
    Stages,
}

#[derive(Debug, Args)]
pub struct ApolloTasksCommand {
    #[command(subcommand)]
    pub action: ApolloTasksAction,
}

#[derive(Debug, Subcommand)]
pub enum ApolloTasksAction {
    Create(ApolloTaskWrite),
    BulkCreate(ApolloJsonArgs),
    Search(ApolloSearchArgs),
}

#[derive(Debug, Args)]
pub struct ApolloCallsCommand {
    #[command(subcommand)]
    pub action: ApolloCallsAction,
}

#[derive(Debug, Subcommand)]
pub enum ApolloCallsAction {
    Create(ApolloCallWrite),
    Search(ApolloSearchArgs),
    Update(ApolloCallUpdate),
}

#[derive(Debug, Args)]
pub struct ApolloNotesCommand {
    #[command(subcommand)]
    pub action: ApolloNotesAction,
}

#[derive(Debug, Subcommand)]
pub enum ApolloNotesAction {
    List(ApolloNotesList),
}

#[derive(Debug, Args)]
pub struct ApolloUsersCommand {
    #[command(subcommand)]
    pub action: ApolloUsersAction,
}

#[derive(Debug, Subcommand)]
pub enum ApolloUsersAction {
    List(ApolloSearchArgs),
    Me(ApolloMeArgs),
}

#[derive(Debug, Args)]
pub struct ApolloListCommand {
    #[command(subcommand)]
    pub action: ApolloListAction,
}

#[derive(Debug, Subcommand)]
pub enum ApolloListAction {
    List,
}

#[derive(Debug, Args)]
pub struct ApolloFieldsCommand {
    #[command(subcommand)]
    pub action: ApolloFieldsAction,
}

#[derive(Debug, Subcommand)]
pub enum ApolloFieldsAction {
    List(ApolloFieldsList),
    Create(ApolloFieldWrite),
}

#[derive(Debug, Args)]
pub struct ApolloUsageCommand {
    #[command(subcommand)]
    pub action: ApolloUsageAction,
}

#[derive(Debug, Subcommand)]
pub enum ApolloUsageAction {
    Stats,
}

#[derive(Debug, Args)]
pub struct ApolloWebhooksCommand {
    #[command(subcommand)]
    pub action: ApolloWebhooksAction,
}

#[derive(Debug, Subcommand)]
pub enum ApolloWebhooksAction {
    Result(ApolloIdArg),
}

#[derive(Debug, Args)]
pub struct ApolloAnalyticsCommand {
    #[command(subcommand)]
    pub action: ApolloAnalyticsAction,
}

#[derive(Debug, Subcommand)]
pub enum ApolloAnalyticsAction {
    Report(ApolloJsonArgs),
}

#[derive(Debug, Args)]
pub struct ApolloSequencesCommand {
    #[command(subcommand)]
    pub action: ApolloSequencesAction,
}

#[derive(Debug, Subcommand)]
pub enum ApolloSequencesAction {
    Search(ApolloSearchArgs),
    Create(ApolloSequenceWrite),
    Update(ApolloSequenceUpdate),
    AddContacts(ApolloSequenceContacts),
    UpdateContactStatus(ApolloSequenceStatus),
    Activate(ApolloIdArg),
    Deactivate(ApolloIdArg),
    Archive(ApolloIdArg),
}

#[derive(Debug, Args)]
pub struct ApolloEmailsCommand {
    #[command(subcommand)]
    pub action: ApolloEmailsAction,
}

#[derive(Debug, Subcommand)]
pub enum ApolloEmailsAction {
    Draft(ApolloEmailDraft),
    SendNow(ApolloSendNow),
    SendStatus(ApolloJsonArgs),
    Search(ApolloSearchArgs),
    Stats(ApolloIdArg),
    Accounts,
}

#[derive(Debug, Args)]
pub struct ApolloNewsCommand {
    #[command(subcommand)]
    pub action: ApolloNewsAction,
}

#[derive(Debug, Subcommand)]
pub enum ApolloNewsAction {
    Search(ApolloSearchArgs),
}

#[derive(Debug, Args)]
pub struct ApolloConversationsCommand {
    #[command(subcommand)]
    pub action: ApolloConversationsAction,
}

#[derive(Debug, Subcommand)]
pub enum ApolloConversationsAction {
    Search(ApolloConversationSearch),
    Get(ApolloIdArg),
    Export(ApolloJsonArgs),
    GetExport(ApolloIdArg),
}

#[derive(Debug, Args)]
pub struct ApolloJsonArgs {
    /// JSON object, inline or from a path; use - to read stdin.
    #[arg(long)]
    pub json: Option<String>,
    /// Extra query parameter as key=value. Repeat for multiple parameters.
    #[arg(long = "query")]
    pub query: Vec<String>,
}

#[derive(Debug, Args)]
pub struct ApolloSearchArgs {
    #[command(flatten)]
    pub json: ApolloJsonArgs,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
    #[arg(long)]
    pub q_keywords: Option<String>,
    #[arg(long)]
    pub q_name: Option<String>,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub location: Option<String>,
    #[arg(long)]
    pub domain: Option<String>,
    #[arg(long)]
    pub sort_by_field: Option<String>,
    #[arg(long)]
    pub sort_ascending: Option<bool>,
}

#[derive(Debug, Args)]
pub struct ApolloIdArg {
    pub id: String,
}

#[derive(Debug, Args)]
pub struct ApolloPagedIdArg {
    pub id: String,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
    #[arg(long = "query")]
    pub query: Vec<String>,
}

#[derive(Debug, Args)]
pub struct ApolloIdJsonArgs {
    pub id: String,
    #[command(flatten)]
    pub json: ApolloJsonArgs,
}

#[derive(Debug, Args)]
pub struct ApolloIdsUpdate {
    /// Comma-separated Apollo IDs.
    #[arg(long)]
    pub ids: String,
    #[arg(long)]
    pub owner_id: Option<String>,
    #[arg(long)]
    pub stage_id: Option<String>,
}

#[derive(Debug, Args)]
pub struct ApolloContactWrite {
    #[command(flatten)]
    pub json: ApolloJsonArgs,
    #[arg(long)]
    pub first_name: Option<String>,
    #[arg(long)]
    pub last_name: Option<String>,
    #[arg(long)]
    pub organization_name: Option<String>,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub account_id: Option<String>,
    #[arg(long)]
    pub email: Option<String>,
    #[arg(long)]
    pub website_url: Option<String>,
    #[arg(long)]
    pub contact_stage_id: Option<String>,
}

#[derive(Debug, Args)]
pub struct ApolloContactUpdate {
    pub id: String,
    #[command(flatten)]
    pub write: ApolloContactWrite,
}

#[derive(Debug, Args)]
pub struct ApolloAccountWrite {
    #[command(flatten)]
    pub json: ApolloJsonArgs,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub domain: Option<String>,
    #[arg(long)]
    pub owner_id: Option<String>,
    #[arg(long)]
    pub account_stage_id: Option<String>,
    #[arg(long)]
    pub phone: Option<String>,
    #[arg(long)]
    pub raw_address: Option<String>,
}

#[derive(Debug, Args)]
pub struct ApolloAccountUpdate {
    pub id: String,
    #[command(flatten)]
    pub write: ApolloAccountWrite,
}

#[derive(Debug, Args)]
pub struct ApolloDealWrite {
    #[command(flatten)]
    pub json: ApolloJsonArgs,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub owner_id: Option<String>,
    #[arg(long)]
    pub account_id: Option<String>,
    #[arg(long)]
    pub amount: Option<f64>,
    #[arg(long)]
    pub opportunity_stage_id: Option<String>,
    #[arg(long)]
    pub closed_date: Option<String>,
}

#[derive(Debug, Args)]
pub struct ApolloDealUpdate {
    pub id: String,
    #[command(flatten)]
    pub write: ApolloDealWrite,
}

#[derive(Debug, Args)]
pub struct ApolloTaskWrite {
    #[command(flatten)]
    pub json: ApolloJsonArgs,
    #[arg(long)]
    pub user_id: Option<String>,
    #[arg(long)]
    pub contact_id: Option<String>,
    #[arg(long = "type")]
    pub task_type: Option<String>,
    #[arg(long)]
    pub priority: Option<String>,
    #[arg(long)]
    pub status: Option<String>,
    #[arg(long)]
    pub due_at: Option<String>,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub note: Option<String>,
}

#[derive(Debug, Args)]
pub struct ApolloCallWrite {
    #[command(flatten)]
    pub json: ApolloJsonArgs,
    #[arg(long)]
    pub contact_id: Option<String>,
    #[arg(long)]
    pub account_id: Option<String>,
    #[arg(long)]
    pub to_number: Option<String>,
    #[arg(long)]
    pub from_number: Option<String>,
    #[arg(long)]
    pub status: Option<String>,
    #[arg(long)]
    pub start_time: Option<String>,
    #[arg(long)]
    pub end_time: Option<String>,
    #[arg(long)]
    pub duration: Option<u64>,
    #[arg(long)]
    pub note: Option<String>,
}

#[derive(Debug, Args)]
pub struct ApolloCallUpdate {
    pub id: String,
    #[command(flatten)]
    pub write: ApolloCallWrite,
}

#[derive(Debug, Args)]
pub struct ApolloNotesList {
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
    #[arg(long)]
    pub contact_id: Option<String>,
    #[arg(long)]
    pub account_id: Option<String>,
    #[arg(long)]
    pub opportunity_id: Option<String>,
    #[arg(long)]
    pub start_date: Option<String>,
    #[arg(long)]
    pub sort_by_field: Option<String>,
    #[arg(long)]
    pub sort_direction: Option<String>,
}

#[derive(Debug, Args)]
pub struct ApolloMeArgs {
    #[arg(long)]
    pub include_credit_usage: bool,
}

#[derive(Debug, Args)]
pub struct ApolloFieldsList {
    #[arg(long)]
    pub source: Option<String>,
}

#[derive(Debug, Args)]
pub struct ApolloFieldWrite {
    #[command(flatten)]
    pub json: ApolloJsonArgs,
    #[arg(long)]
    pub label: Option<String>,
    #[arg(long)]
    pub modality: Option<String>,
    #[arg(long = "type")]
    pub field_type: Option<String>,
}

#[derive(Debug, Args)]
pub struct ApolloSequenceWrite {
    #[command(flatten)]
    pub json: ApolloJsonArgs,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub active: Option<bool>,
    #[arg(long)]
    pub user_id: Option<String>,
}

#[derive(Debug, Args)]
pub struct ApolloSequenceUpdate {
    pub id: String,
    #[command(flatten)]
    pub write: ApolloSequenceWrite,
}

#[derive(Debug, Args)]
pub struct ApolloSequenceContacts {
    pub id: String,
    #[arg(long)]
    pub contact_ids: String,
    #[arg(long)]
    pub status: Option<String>,
    #[arg(long)]
    pub email_account_id: Option<String>,
    #[arg(long)]
    pub email_address: Option<String>,
}

#[derive(Debug, Args)]
pub struct ApolloSequenceStatus {
    #[arg(long)]
    pub sequence_ids: String,
    #[arg(long)]
    pub contact_ids: String,
    #[arg(long)]
    pub mode: String,
}

#[derive(Debug, Args)]
pub struct ApolloEmailDraft {
    #[command(flatten)]
    pub json: ApolloJsonArgs,
    #[arg(long)]
    pub contact_id: Option<String>,
    #[arg(long)]
    pub subject: Option<String>,
    #[arg(long)]
    pub body_html: Option<String>,
}

#[derive(Debug, Args)]
pub struct ApolloSendNow {
    pub id: String,
    #[command(flatten)]
    pub json: ApolloJsonArgs,
    #[arg(long)]
    pub surface: Option<String>,
}

#[derive(Debug, Args)]
pub struct ApolloConversationSearch {
    #[command(flatten)]
    pub json: ApolloJsonArgs,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
    #[arg(long)]
    pub conversation_type: Option<String>,
    #[arg(long)]
    pub account_id: Option<String>,
    #[arg(long)]
    pub sort_by_field: Option<String>,
}

#[derive(Debug, Args)]
#[command(
    about = "Get a complete CRM view of a record",
    long_about = "Returns one JSON object containing the CRM record, associated activities, and associated notes. Use --include-mail to also include synced email history."
)]
pub struct PipedriveAssociatedView {
    /// Pipedrive record ID.
    pub id: String,
    /// Maximum records to aggregate for each associated history category.
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
    /// Include synced email messages associated with the record.
    #[arg(long)]
    pub include_mail: bool,
    /// Include label details on the primary CRM record.
    #[arg(long)]
    pub include_labels: bool,
}

#[derive(Debug, Args)]
pub struct PipedriveActivitiesCommand {
    #[command(subcommand)]
    pub action: PipedriveActivitiesAction,
}

#[derive(Debug, Subcommand)]
pub enum PipedriveActivitiesAction {
    /// List activities, optionally filtered by linked CRM records.
    List(Box<PipedriveActivityList>),
    /// Get one activity.
    Get(PipedriveIdArg),
}

#[derive(Debug, Args)]
pub struct PipedriveActivityList {
    /// Maximum activities to aggregate.
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
    #[arg(long)]
    pub filter_id: Option<String>,
    #[arg(long)]
    pub ids: Option<String>,
    #[arg(long)]
    pub owner_id: Option<String>,
    #[arg(long)]
    pub deal_id: Option<String>,
    #[arg(long)]
    pub lead_id: Option<String>,
    #[arg(long)]
    pub person_id: Option<String>,
    #[arg(long)]
    pub org_id: Option<String>,
    /// Filter by completion state.
    #[arg(long)]
    pub done: Option<bool>,
    #[arg(long)]
    pub updated_since: Option<String>,
    #[arg(long)]
    pub updated_until: Option<String>,
    #[arg(long)]
    pub sort_by: Option<String>,
    #[arg(long, value_enum)]
    pub sort_direction: Option<PipedriveSortDirection>,
    /// Include activity attendees in responses.
    #[arg(long)]
    pub include_attendees: bool,
}

#[derive(Debug, Args)]
pub struct PipedriveNotesCommand {
    #[command(subcommand)]
    pub action: PipedriveNotesAction,
}

#[derive(Debug, Subcommand)]
pub enum PipedriveNotesAction {
    /// List notes, optionally filtered by linked CRM records.
    List(PipedriveNoteList),
    /// Get one note.
    Get(PipedriveIdArg),
}

#[derive(Debug, Args)]
pub struct PipedriveNoteList {
    /// Maximum notes to aggregate.
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
    #[arg(long)]
    pub user_id: Option<String>,
    #[arg(long)]
    pub lead_id: Option<String>,
    #[arg(long)]
    pub deal_id: Option<String>,
    #[arg(long)]
    pub person_id: Option<String>,
    #[arg(long)]
    pub org_id: Option<String>,
    #[arg(long)]
    pub sort: Option<String>,
    #[arg(long)]
    pub start_date: Option<String>,
    #[arg(long)]
    pub end_date: Option<String>,
    #[arg(long)]
    pub updated_since: Option<String>,
}

#[derive(Debug, Args)]
pub struct PipedriveMailboxCommand {
    #[command(subcommand)]
    pub resource: PipedriveMailboxResource,
}

#[derive(Debug, Subcommand)]
pub enum PipedriveMailboxResource {
    /// Get individual synced email messages.
    Messages(PipedriveMailboxMessagesCommand),
    /// List and inspect synced email threads.
    Threads(PipedriveMailboxThreadsCommand),
}

#[derive(Debug, Args)]
pub struct PipedriveMailboxMessagesCommand {
    #[command(subcommand)]
    pub action: PipedriveMailboxMessagesAction,
}

#[derive(Debug, Subcommand)]
pub enum PipedriveMailboxMessagesAction {
    /// Get one synced email message, optionally including its full body.
    Get(PipedriveMailMessageGet),
}

#[derive(Debug, Args)]
pub struct PipedriveMailMessageGet {
    /// Pipedrive mail message ID.
    pub id: String,
    /// Include the full message body, not only metadata and snippet.
    #[arg(long)]
    pub include_body: bool,
}

#[derive(Debug, Args)]
pub struct PipedriveMailboxThreadsCommand {
    #[command(subcommand)]
    pub action: PipedriveMailboxThreadsAction,
}

#[derive(Debug, Subcommand)]
pub enum PipedriveMailboxThreadsAction {
    /// List synced email threads in a mailbox folder.
    List(PipedriveMailThreadList),
    /// Get one synced email thread.
    Get(PipedriveIdArg),
    /// List every message in a synced email thread.
    Messages(PipedriveIdArg),
}

#[derive(Debug, Args)]
pub struct PipedriveMailThreadList {
    /// Mailbox folder to list.
    #[arg(long, value_enum, default_value_t = PipedriveMailFolder::Inbox)]
    pub folder: PipedriveMailFolder,
    /// Maximum threads to aggregate.
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum PipedriveMailFolder {
    Inbox,
    Drafts,
    Sent,
    Archive,
}

#[derive(Debug, Args)]
pub struct PipedriveGetWithLabels {
    pub id: String,
    #[arg(long)]
    pub include_labels: bool,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum PipedriveSortDirection {
    Asc,
    Desc,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum PipedriveDealStatus {
    Open,
    Won,
    Lost,
    Deleted,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum PipedriveSearchDealStatus {
    Open,
    Won,
    Lost,
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn pipedrive_history_commands_are_discoverable_in_help() {
        let mut command = Cli::command();
        let pipedrive = command
            .find_subcommand_mut("pipedrive")
            .expect("pipedrive command");
        let mut help = Vec::new();
        pipedrive.write_long_help(&mut help).unwrap();
        let help = String::from_utf8(help).unwrap();

        assert!(help.contains("synced email"));
        assert!(help.contains("deals view"));
        assert!(help.contains("mailbox messages get"));
        assert!(help.contains("activities"));
        assert!(help.contains("notes"));
    }

    #[test]
    fn config_profile_commands_are_discoverable_in_help() {
        let mut command = Cli::command();
        let config = command
            .find_subcommand_mut("config")
            .expect("config command");
        let mut help = Vec::new();
        config.write_long_help(&mut help).unwrap();
        let help = String::from_utf8(help).unwrap();

        assert!(help.contains("profiles"));
        assert!(help.contains("default-profile"));
        assert!(help.contains("without exposing credentials"));
    }

    #[test]
    fn generic_request_is_discoverable_for_http_services() {
        for service in [
            "jira",
            "confluence",
            "bitbucket",
            "github",
            "email",
            "calendar",
            "pipedrive",
        ] {
            let mut command = Cli::command();
            let service_command = command
                .find_subcommand_mut(service)
                .expect("service command");
            let mut help = Vec::new();
            service_command.write_long_help(&mut help).unwrap();
            let help = String::from_utf8(help).unwrap();
            assert!(help.contains("request"), "{service} help lacks request");
        }
    }

    #[test]
    fn pagination_contract_is_discoverable_in_top_level_help() {
        let mut help = Vec::new();
        Cli::command().write_long_help(&mut help).unwrap();
        let help = String::from_utf8(help).unwrap();
        assert!(help.contains("_aai.pagination"));
        assert!(help.contains("retrieving more results"));
    }
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
    Diff(GithubPrDiff),
    Files(GithubPrFiles),
    Commits(GithubPrCommits),
    Timeline(GithubPrTimeline),
    Comments(GithubPrCommentsCommand),
    #[command(name = "review-comments")]
    ReviewComments(GithubPrReviewCommentsCommand),
    Reviews(GithubPrReviewsCommand),
}

#[derive(Debug, Args)]
pub struct GithubPrDiff {
    pub pr: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub output: Option<String>,
}

#[derive(Debug, Args)]
pub struct GithubPrFiles {
    pub pr: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct GithubPrCommits {
    pub pr: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct GithubPrTimeline {
    pub pr: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct GithubPrCommentsCommand {
    #[command(subcommand)]
    pub action: GithubPrCommentAction,
}

#[derive(Debug, Subcommand)]
pub enum GithubPrCommentAction {
    List(GithubPrCommentList),
    Get(GithubPrCommentGet),
    Create(GithubPrCommentWrite),
    Update(GithubPrCommentWrite),
    Delete(GithubPrCommentGet),
}

#[derive(Debug, Args)]
pub struct GithubPrCommentList {
    pub pr: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct GithubPrCommentGet {
    pub pr: u64,
    pub comment: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
}

#[derive(Debug, Args)]
pub struct GithubPrCommentWrite {
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
pub struct GithubPrReviewCommentsCommand {
    #[command(subcommand)]
    pub action: GithubPrReviewCommentAction,
}

#[derive(Debug, Subcommand)]
pub enum GithubPrReviewCommentAction {
    List(GithubPrReviewCommentList),
    Get(GithubPrReviewCommentGet),
    Create(GithubPrReviewCommentCreate),
    Update(GithubPrReviewCommentUpdate),
    Delete(GithubPrReviewCommentGet),
}

#[derive(Debug, Args)]
pub struct GithubPrReviewCommentList {
    pub pr: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct GithubPrReviewCommentGet {
    pub pr: u64,
    pub comment: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
}

#[derive(Debug, Args)]
pub struct GithubPrReviewCommentCreate {
    pub pr: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub body: Option<String>,
    #[arg(long)]
    pub path: Option<String>,
    #[arg(long)]
    pub line: Option<u64>,
    #[arg(long)]
    pub side: Option<String>,
    #[arg(long = "start-line")]
    pub start_line: Option<u64>,
    #[arg(long = "start-side")]
    pub start_side: Option<String>,
    #[arg(long = "commit-id")]
    pub commit_id: Option<String>,
    #[arg(long = "in-reply-to")]
    pub in_reply_to: Option<u64>,
}

#[derive(Debug, Args)]
pub struct GithubPrReviewCommentUpdate {
    pub pr: u64,
    #[arg(long)]
    pub comment: u64,
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
pub struct GithubPrReviewsCommand {
    #[command(subcommand)]
    pub action: GithubPrReviewAction,
}

#[derive(Debug, Subcommand)]
pub enum GithubPrReviewAction {
    List(GithubPrReviewList),
    Get(GithubPrReviewGet),
    Create(GithubPrReviewCreate),
}

#[derive(Debug, Args)]
pub struct GithubPrReviewList {
    pub pr: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct GithubPrReviewGet {
    pub pr: u64,
    pub review: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
}

#[derive(Debug, Args)]
pub struct GithubPrReviewCreate {
    pub pr: u64,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub body: Option<String>,
    #[arg(long)]
    pub event: Option<String>,
    #[arg(long = "commit-id")]
    pub commit_id: Option<String>,
    #[arg(long = "comments-json")]
    pub comments_json: Option<String>,
}

#[derive(Debug, Args)]
pub struct GithubBranchesCommand {
    #[command(subcommand)]
    pub action: GithubBranchesAction,
}

#[derive(Debug, Subcommand)]
pub enum GithubBranchesAction {
    List(GithubBranchList),
    Get(GithubBranchGet),
}

#[derive(Debug, Args)]
pub struct GithubBranchList {
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
    #[arg(long, conflicts_with = "name_prefix")]
    pub name_contains: Option<String>,
    #[arg(long, conflicts_with = "name_contains")]
    pub name_prefix: Option<String>,
    #[arg(long)]
    pub protected: Option<bool>,
}

#[derive(Debug, Args)]
pub struct GithubBranchGet {
    pub name: String,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
}

#[derive(Debug, Args)]
pub struct GithubSourceCommand {
    #[command(subcommand)]
    pub action: GithubSourceAction,
}

#[derive(Debug, Subcommand)]
pub enum GithubSourceAction {
    Get(GithubSourceGet),
    History(GithubSourceHistory),
}

#[derive(Debug, Args)]
pub struct GithubSourceGet {
    pub commit: String,
    pub path: String,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, conflicts_with = "meta")]
    pub output: Option<String>,
    #[arg(long, conflicts_with = "output")]
    pub meta: bool,
}

#[derive(Debug, Args)]
pub struct GithubSourceHistory {
    pub commit: String,
    pub path: String,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct SheetsCommand {
    #[command(subcommand)]
    pub resource: SheetsResource,
}

#[derive(Debug, Subcommand)]
pub enum SheetsResource {
    /// Discover spreadsheets in Drive, or get metadata (sheet tabs) for a specific spreadsheet.
    Spreadsheets(SpreadsheetsCommand),
    /// Read, write, or clear cell values in a spreadsheet range.
    Values(ValuesCommand),
}

#[derive(Debug, Args)]
pub struct SpreadsheetsCommand {
    #[command(subcommand)]
    pub action: SpreadsheetsAction,
}

#[derive(Debug, Subcommand)]
pub enum SpreadsheetsAction {
    /// List all Google Sheets spreadsheets in Drive.
    List(SpreadsheetsListArgs),
    /// Get spreadsheet metadata including all sheet tab names and IDs.
    Get(SpreadsheetsGetArgs),
}

#[derive(Debug, Args)]
pub struct SpreadsheetsListArgs {
    /// Pagination token from a previous response to fetch the next page.
    #[arg(long)]
    pub page_token: Option<String>,
}

#[derive(Debug, Args)]
pub struct SpreadsheetsGetArgs {
    pub spreadsheet_id: String,
}

#[derive(Debug, Args)]
pub struct ValuesCommand {
    #[command(subcommand)]
    pub action: ValuesAction,
}

#[derive(Debug, Subcommand)]
pub enum ValuesAction {
    /// Read cell values from a range (e.g. 'Sheet1'!A1:D5).
    Get(ValuesGetArgs),
    /// Write cell values to a range.
    Update(ValuesUpdateArgs),
    /// Clear cell values from a range (formatting is preserved).
    Clear(ValuesClearArgs),
}

#[derive(Debug, Args)]
pub struct ValuesGetArgs {
    pub spreadsheet_id: String,
    /// A1 notation range, e.g. 'Sheet1'!A1:D5 or 'Inventory'!2:2
    pub range: String,
}

#[derive(Debug, Args)]
pub struct ValuesUpdateArgs {
    pub spreadsheet_id: String,
    /// A1 notation range, e.g. 'Sheet1'!A1:D5
    pub range: String,
    /// JSON array of arrays: [["A1","B1"],["A2","B2"]]
    #[arg(long)]
    pub values: String,
}

#[derive(Debug, Args)]
pub struct ValuesClearArgs {
    pub spreadsheet_id: String,
    /// A1 notation range, e.g. 'Sheet1'!A1:D5
    pub range: String,
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
