use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "aai-cli")]
#[command(about = "Agent-friendly CLI wrappers for common work APIs")]
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
    Pipedrive(PipedriveCommand),
    Secrets(SecretsCommand),
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
    Search(JiraIssueSearch),
    Get(IdArg),
    Create(JiraIssueCreate),
    Update(JiraIssueUpdate),
    Delete(IdArg),
}

#[derive(Debug, Args)]
pub struct JiraIssueList {
    #[arg(long)]
    pub jql: Option<String>,
    #[arg(long)]
    pub fields: Option<String>,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct JiraIssueSearch {
    #[arg(long)]
    pub jql: String,
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
pub struct ConfluenceCommand {
    #[command(subcommand)]
    pub resource: ConfluenceResource,
}

#[derive(Debug, Subcommand)]
pub enum ConfluenceResource {
    Spaces(ConfluenceSpacesCommand),
    Pages(ConfluencePagesCommand),
    Search(ConfluenceSearch),
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
    Move(ConfluencePageMove),
    Delete(IdArg),
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
pub struct ConfluenceSearch {
    #[arg(long, conflicts_with = "query")]
    pub cql: Option<String>,
    #[arg(long, conflicts_with = "cql")]
    pub query: Option<String>,
    #[arg(long, default_value_t = 25)]
    pub limit: u32,
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
    Pipelines(BitbucketPipelinesCommand),
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

#[derive(Debug, Args)]
pub struct PipedriveCommand {
    #[command(subcommand)]
    pub resource: PipedriveResource,
}

#[derive(Debug, Subcommand)]
pub enum PipedriveResource {
    Leads(PipedriveLeadsCommand),
    Persons(PipedrivePersonsCommand),
    Organizations(PipedriveOrganizationsCommand),
    Deals(PipedriveDealsCommand),
    Labels(PipedriveLabelsCommand),
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
    List(PipedrivePersonList),
    Search(PipedrivePersonSearch),
    Get(PipedriveGetWithLabels),
    Create(PipedrivePersonWrite),
    Update(PipedrivePersonUpdate),
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
    List(PipedriveOrganizationList),
    Search(PipedriveOrganizationSearch),
    Get(PipedriveGetWithLabels),
    Create(PipedriveOrganizationWrite),
    Update(PipedriveOrganizationUpdate),
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
    List(PipedriveDealList),
    Search(PipedriveDealSearch),
    Get(PipedriveGetWithLabels),
    Create(PipedriveDealWrite),
    Update(PipedriveDealUpdate),
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
