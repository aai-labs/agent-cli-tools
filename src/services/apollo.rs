use reqwest::Method;
use serde_json::{json, Number, Value};

use crate::{
    cli::*,
    config::Context,
    error::AppError,
    http::ApiClient,
    input,
    services::{
        generic_request,
        shared::{apollo_base, enc, CtxProfile},
    },
};

const SERVICE: &str = "apollo";
const HEALTH_URL: &str = "https://api.apollo.io/v1/auth/health";
const PAGE_ARRAY_KEYS: &[&str] = &[
    "people",
    "organizations",
    "accounts",
    "contacts",
    "opportunities",
    "emailer_campaigns",
    "tasks",
    "phone_calls",
    "notes",
    "news_articles",
    "conversations",
    "users",
    "job_postings",
];

pub(crate) async fn dispatch(
    client: &ApiClient,
    ctx: &Context,
    command: ApolloCommand,
) -> Result<Value, AppError> {
    match command.resource {
        ApolloResource::Health => {
            request(
                client,
                ctx,
                "health",
                Method::GET,
                HEALTH_URL.to_string(),
                None,
            )
            .await
        }
        ApolloResource::People(command) => people(client, ctx, command).await,
        ApolloResource::Organizations(command) => organizations(client, ctx, command).await,
        ApolloResource::Contacts(command) => contacts(client, ctx, command).await,
        ApolloResource::Accounts(command) => accounts(client, ctx, command).await,
        ApolloResource::Deals(command) => deals(client, ctx, command).await,
        ApolloResource::Tasks(command) => tasks(client, ctx, command).await,
        ApolloResource::Calls(command) => calls(client, ctx, command).await,
        ApolloResource::Notes(command) => notes(client, ctx, command).await,
        ApolloResource::Users(command) => users(client, ctx, command).await,
        ApolloResource::Labels(command) => match command.action {
            ApolloListAction::List => {
                get(client, ctx, "labels.list", "/labels", Query::new()).await
            }
        },
        ApolloResource::Fields(command) => fields(client, ctx, command).await,
        ApolloResource::CustomFields(command) => match command.action {
            ApolloListAction::List => {
                get(
                    client,
                    ctx,
                    "custom-fields.list",
                    "/typed_custom_fields",
                    Query::new(),
                )
                .await
            }
        },
        ApolloResource::Usage(command) => match command.action {
            ApolloUsageAction::Stats => {
                post(
                    client,
                    ctx,
                    "usage.stats",
                    "/usage_stats/api_usage_stats",
                    None,
                )
                .await
            }
        },
        ApolloResource::Webhooks(command) => match command.action {
            ApolloWebhooksAction::Result(args) => {
                get(
                    client,
                    ctx,
                    "webhooks.result",
                    &format!("/webhook_result/{}", enc(&args.id)),
                    Query::new(),
                )
                .await
            }
        },
        ApolloResource::Analytics(command) => match command.action {
            ApolloAnalyticsAction::Report(args) => {
                post_json_args(
                    client,
                    ctx,
                    "analytics.report",
                    "/reports/sync_report",
                    args,
                )
                .await
            }
        },
        ApolloResource::Sequences(command) => sequences(client, ctx, command).await,
        ApolloResource::Emails(command) => emails(client, ctx, command).await,
        ApolloResource::News(command) => match command.action {
            ApolloNewsAction::Search(args) => {
                paginate_query(
                    client,
                    ctx,
                    "news.search",
                    "/news_articles/search",
                    Method::POST,
                    search_query(args)?,
                    50,
                )
                .await
            }
        },
        ApolloResource::Conversations(command) => conversations(client, ctx, command).await,
        ApolloResource::Request(args) => {
            generic_request::dispatch(client, ctx, SERVICE, apollo_base(ctx.profile()), args).await
        }
    }
}

async fn people(
    client: &ApiClient,
    ctx: &Context,
    command: ApolloPeopleCommand,
) -> Result<Value, AppError> {
    match command.action {
        ApolloPeopleAction::Search(args) => {
            paginate_query(
                client,
                ctx,
                "people.search",
                "/mixed_people/api_search",
                Method::POST,
                search_query(args)?,
                100,
            )
            .await
        }
        ApolloPeopleAction::Get(args) => {
            get(
                client,
                ctx,
                "people.get",
                &format!("/people/{}", enc(&args.id)),
                Query::new(),
            )
            .await
        }
        ApolloPeopleAction::Enrich(args) => {
            let mut query = Query::from_pairs(args.json.query)?;
            query.push("first_name", args.first_name.as_deref());
            query.push("last_name", args.last_name.as_deref());
            query.push("name", args.name.as_deref());
            query.push("email", args.email.as_deref());
            query.push("organization_name", args.organization_name.as_deref());
            query.push("domain", args.domain.as_deref());
            query.push("id", args.id.as_deref());
            query.push("linkedin_url", args.linkedin_url.as_deref());
            query.push_bool("reveal_personal_emails", args.reveal_personal_emails);
            query.push_bool("reveal_phone_number", args.reveal_phone_number);
            post_with_query(client, ctx, "people.enrich", "/people/match", query, None).await
        }
        ApolloPeopleAction::BulkEnrich(args) => {
            post_json_args(
                client,
                ctx,
                "people.bulk-enrich",
                "/people/bulk_match",
                args,
            )
            .await
        }
    }
}

async fn organizations(
    client: &ApiClient,
    ctx: &Context,
    command: ApolloOrganizationsCommand,
) -> Result<Value, AppError> {
    match command.action {
        ApolloOrganizationsAction::Search(args) => {
            paginate_query(
                client,
                ctx,
                "organizations.search",
                "/mixed_companies/search",
                Method::POST,
                search_query(args)?,
                100,
            )
            .await
        }
        ApolloOrganizationsAction::Get(args) => {
            get(
                client,
                ctx,
                "organizations.get",
                &format!("/organizations/{}", enc(&args.id)),
                Query::new(),
            )
            .await
        }
        ApolloOrganizationsAction::Enrich(args) => {
            let mut query = Query::new();
            query.push("domain", args.domain.as_deref());
            query.push("linkedin_url", args.linkedin_url.as_deref());
            query.push("name", args.name.as_deref());
            query.push("website", args.website.as_deref());
            get(
                client,
                ctx,
                "organizations.enrich",
                "/organizations/enrich",
                query,
            )
            .await
        }
        ApolloOrganizationsAction::BulkEnrich(args) => {
            post_json_args(
                client,
                ctx,
                "organizations.bulk-enrich",
                "/organizations/bulk_enrich",
                args,
            )
            .await
        }
        ApolloOrganizationsAction::JobPostings(args) => {
            let query = Query::from_pairs(args.query)?;
            paginate_query(
                client,
                ctx,
                "organizations.job-postings",
                &format!("/organizations/{}/job_postings", enc(&args.id)),
                Method::GET,
                (query, args.limit),
                100,
            )
            .await
        }
    }
}

async fn contacts(
    client: &ApiClient,
    ctx: &Context,
    command: ApolloContactsCommand,
) -> Result<Value, AppError> {
    match command.action {
        ApolloContactsAction::Create(args) => {
            post(
                client,
                ctx,
                "contacts.create",
                "/contacts",
                Some(contact_body(args)?),
            )
            .await
        }
        ApolloContactsAction::Get(args) => {
            get(
                client,
                ctx,
                "contacts.get",
                &format!("/contacts/{}", enc(&args.id)),
                Query::new(),
            )
            .await
        }
        ApolloContactsAction::Search(args) => {
            let (body, limit) = search_body(args)?;
            paginate_body(
                client,
                ctx,
                "contacts.search",
                "/contacts/search",
                body,
                limit,
                100,
            )
            .await
        }
        ApolloContactsAction::Update(args) => {
            patch(
                client,
                ctx,
                "contacts.update",
                &format!("/contacts/{}", enc(&args.id)),
                Some(contact_body(args.write)?),
            )
            .await
        }
        ApolloContactsAction::BulkCreate(args) => {
            post_json_args(
                client,
                ctx,
                "contacts.bulk-create",
                "/contacts/bulk_create",
                args,
            )
            .await
        }
        ApolloContactsAction::BulkUpdate(args) => {
            post_json_args(
                client,
                ctx,
                "contacts.bulk-update",
                "/contacts/bulk_update",
                args,
            )
            .await
        }
        ApolloContactsAction::UpdateStages(args) => {
            ids_update(
                client,
                ctx,
                "contacts.update-stages",
                "/contacts/update_stages",
                "contact_ids[]",
                "contact_stage_id",
                args,
            )
            .await
        }
        ApolloContactsAction::UpdateOwners(args) => {
            ids_update(
                client,
                ctx,
                "contacts.update-owners",
                "/contacts/update_owners",
                "contact_ids[]",
                "owner_id",
                args,
            )
            .await
        }
        ApolloContactsAction::Deals(args) => {
            post_json_args(
                client,
                ctx,
                "contacts.deals",
                &format!("/contacts/{}/opportunities", enc(&args.id)),
                args.json,
            )
            .await
        }
    }
}

async fn accounts(
    client: &ApiClient,
    ctx: &Context,
    command: ApolloAccountsCommand,
) -> Result<Value, AppError> {
    match command.action {
        ApolloAccountsAction::Create(args) => {
            post(
                client,
                ctx,
                "accounts.create",
                "/accounts",
                Some(account_body(args)?),
            )
            .await
        }
        ApolloAccountsAction::Get(args) => {
            get(
                client,
                ctx,
                "accounts.get",
                &format!("/accounts/{}", enc(&args.id)),
                Query::new(),
            )
            .await
        }
        ApolloAccountsAction::Search(args) => {
            let (body, limit) = search_body(args)?;
            paginate_body(
                client,
                ctx,
                "accounts.search",
                "/accounts/search",
                body,
                limit,
                100,
            )
            .await
        }
        ApolloAccountsAction::Update(args) => {
            patch(
                client,
                ctx,
                "accounts.update",
                &format!("/accounts/{}", enc(&args.id)),
                Some(account_body(args.write)?),
            )
            .await
        }
        ApolloAccountsAction::BulkCreate(args) => {
            post_json_args(
                client,
                ctx,
                "accounts.bulk-create",
                "/accounts/bulk_create",
                args,
            )
            .await
        }
        ApolloAccountsAction::BulkUpdate(args) => {
            post_json_args(
                client,
                ctx,
                "accounts.bulk-update",
                "/accounts/bulk_update",
                args,
            )
            .await
        }
        ApolloAccountsAction::UpdateOwners(args) => {
            ids_update(
                client,
                ctx,
                "accounts.update-owners",
                "/accounts/update_owners",
                "account_ids[]",
                "owner_id",
                args,
            )
            .await
        }
        ApolloAccountsAction::Stages => {
            get(
                client,
                ctx,
                "accounts.stages",
                "/account_stages",
                Query::new(),
            )
            .await
        }
    }
}

async fn deals(
    client: &ApiClient,
    ctx: &Context,
    command: ApolloDealsCommand,
) -> Result<Value, AppError> {
    match command.action {
        ApolloDealsAction::Create(args) => {
            post(
                client,
                ctx,
                "deals.create",
                "/opportunities",
                Some(deal_body(args)?),
            )
            .await
        }
        ApolloDealsAction::List(args) => {
            paginate_query(
                client,
                ctx,
                "deals.list",
                "/opportunities/search",
                Method::GET,
                search_query(args)?,
                100,
            )
            .await
        }
        ApolloDealsAction::Get(args) => {
            get(
                client,
                ctx,
                "deals.get",
                &format!("/opportunities/{}", enc(&args.id)),
                Query::new(),
            )
            .await
        }
        ApolloDealsAction::Update(args) => {
            patch(
                client,
                ctx,
                "deals.update",
                &format!("/opportunities/{}", enc(&args.id)),
                Some(deal_body(args.write)?),
            )
            .await
        }
        ApolloDealsAction::Stages => {
            get(
                client,
                ctx,
                "deals.stages",
                "/opportunity_stages",
                Query::new(),
            )
            .await
        }
    }
}

async fn tasks(
    client: &ApiClient,
    ctx: &Context,
    command: ApolloTasksCommand,
) -> Result<Value, AppError> {
    match command.action {
        ApolloTasksAction::Create(args) => {
            post(
                client,
                ctx,
                "tasks.create",
                "/tasks",
                Some(task_body(args)?),
            )
            .await
        }
        ApolloTasksAction::BulkCreate(args) => {
            post_json_args(client, ctx, "tasks.bulk-create", "/tasks/bulk_create", args).await
        }
        ApolloTasksAction::Search(args) => {
            paginate_query(
                client,
                ctx,
                "tasks.search",
                "/tasks/search",
                Method::POST,
                search_query(args)?,
                100,
            )
            .await
        }
    }
}

async fn calls(
    client: &ApiClient,
    ctx: &Context,
    command: ApolloCallsCommand,
) -> Result<Value, AppError> {
    match command.action {
        ApolloCallsAction::Create(args) => {
            let query = call_query(args)?;
            post_with_query(client, ctx, "calls.create", "/phone_calls", query, None).await
        }
        ApolloCallsAction::Search(args) => {
            paginate_query(
                client,
                ctx,
                "calls.search",
                "/phone_calls/search",
                Method::GET,
                search_query(args)?,
                100,
            )
            .await
        }
        ApolloCallsAction::Update(args) => {
            let query = call_query(args.write)?;
            put_with_query(
                client,
                ctx,
                "calls.update",
                &format!("/phone_calls/{}", enc(&args.id)),
                query,
                None,
            )
            .await
        }
    }
}

async fn notes(
    client: &ApiClient,
    ctx: &Context,
    command: ApolloNotesCommand,
) -> Result<Value, AppError> {
    match command.action {
        ApolloNotesAction::List(args) => {
            let mut query = Query::new();
            query.push("contact_id", args.contact_id.as_deref());
            query.push("account_id", args.account_id.as_deref());
            query.push("opportunity_id", args.opportunity_id.as_deref());
            query.push("start_date", args.start_date.as_deref());
            query.push("sort_by_field", args.sort_by_field.as_deref());
            query.push("sort_direction", args.sort_direction.as_deref());
            paginate_skip(client, ctx, "notes.list", "/notes", query, args.limit, 100).await
        }
    }
}

async fn users(
    client: &ApiClient,
    ctx: &Context,
    command: ApolloUsersCommand,
) -> Result<Value, AppError> {
    match command.action {
        ApolloUsersAction::List(args) => {
            paginate_query(
                client,
                ctx,
                "users.list",
                "/users/search",
                Method::GET,
                search_query(args)?,
                100,
            )
            .await
        }
        ApolloUsersAction::Me(args) => {
            let mut query = Query::new();
            query.push_bool("include_credit_usage", args.include_credit_usage);
            get(client, ctx, "users.me", "/users/api_profile", query).await
        }
    }
}

async fn fields(
    client: &ApiClient,
    ctx: &Context,
    command: ApolloFieldsCommand,
) -> Result<Value, AppError> {
    match command.action {
        ApolloFieldsAction::List(args) => {
            let mut query = Query::new();
            query.push("source", args.source.as_deref());
            get(client, ctx, "fields.list", "/fields", query).await
        }
        ApolloFieldsAction::Create(args) => {
            post(
                client,
                ctx,
                "fields.create",
                "/fields",
                Some(field_body(args)?),
            )
            .await
        }
    }
}

async fn sequences(
    client: &ApiClient,
    ctx: &Context,
    command: ApolloSequencesCommand,
) -> Result<Value, AppError> {
    match command.action {
        ApolloSequencesAction::Search(args) => {
            paginate_query(
                client,
                ctx,
                "sequences.search",
                "/emailer_campaigns/search",
                Method::POST,
                search_query(args)?,
                100,
            )
            .await
        }
        ApolloSequencesAction::Create(args) => {
            post(
                client,
                ctx,
                "sequences.create",
                "/sequences",
                Some(sequence_body(args)?),
            )
            .await
        }
        ApolloSequencesAction::Update(args) => {
            put(
                client,
                ctx,
                "sequences.update",
                &format!("/sequences/{}", enc(&args.id)),
                Some(sequence_body(args.write)?),
            )
            .await
        }
        ApolloSequencesAction::AddContacts(args) => {
            let mut query = Query::new();
            query.push_value("emailer_campaign_id", &args.id);
            query.push_list("contact_ids[]", &args.contact_ids);
            query.push("status", args.status.as_deref());
            query.push(
                "send_email_from_email_account_id",
                args.email_account_id.as_deref(),
            );
            query.push(
                "send_email_from_email_address",
                args.email_address.as_deref(),
            );
            post_with_query(
                client,
                ctx,
                "sequences.add-contacts",
                &format!("/emailer_campaigns/{}/add_contact_ids", enc(&args.id)),
                query,
                None,
            )
            .await
        }
        ApolloSequencesAction::UpdateContactStatus(args) => {
            let mut query = Query::new();
            query.push_list("emailer_campaign_ids[]", &args.sequence_ids);
            query.push_list("contact_ids[]", &args.contact_ids);
            query.push_value("mode", args.mode);
            post_with_query(
                client,
                ctx,
                "sequences.update-contact-status",
                "/emailer_campaigns/remove_or_stop_contact_ids",
                query,
                None,
            )
            .await
        }
        ApolloSequencesAction::Activate(args) => {
            post(
                client,
                ctx,
                "sequences.activate",
                &format!("/emailer_campaigns/{}/approve", enc(&args.id)),
                None,
            )
            .await
        }
        ApolloSequencesAction::Deactivate(args) => {
            post(
                client,
                ctx,
                "sequences.deactivate",
                &format!("/emailer_campaigns/{}/abort", enc(&args.id)),
                None,
            )
            .await
        }
        ApolloSequencesAction::Archive(args) => {
            post(
                client,
                ctx,
                "sequences.archive",
                &format!("/emailer_campaigns/{}/archive", enc(&args.id)),
                None,
            )
            .await
        }
    }
}

async fn emails(
    client: &ApiClient,
    ctx: &Context,
    command: ApolloEmailsCommand,
) -> Result<Value, AppError> {
    match command.action {
        ApolloEmailsAction::Draft(args) => {
            post(
                client,
                ctx,
                "emails.draft",
                "/emailer_messages",
                Some(email_body(args)?),
            )
            .await
        }
        ApolloEmailsAction::SendNow(args) => {
            let mut body = json_body("emails.send-now", args.json)?;
            set_string(&mut body, "surface", args.surface);
            post(
                client,
                ctx,
                "emails.send-now",
                &format!("/emailer_messages/{}/send_now", enc(&args.id)),
                Some(body),
            )
            .await
        }
        ApolloEmailsAction::SendStatus(args) => {
            post_json_args(
                client,
                ctx,
                "emails.send-status",
                "/emailer_messages/email_send_status",
                args,
            )
            .await
        }
        ApolloEmailsAction::Search(args) => {
            paginate_query(
                client,
                ctx,
                "emails.search",
                "/emailer_messages/search",
                Method::GET,
                search_query(args)?,
                100,
            )
            .await
        }
        ApolloEmailsAction::Stats(args) => {
            get(
                client,
                ctx,
                "emails.stats",
                &format!("/emailer_messages/{}/activities", enc(&args.id)),
                Query::new(),
            )
            .await
        }
        ApolloEmailsAction::Accounts => {
            get(
                client,
                ctx,
                "emails.accounts",
                "/email_accounts",
                Query::new(),
            )
            .await
        }
    }
}

async fn conversations(
    client: &ApiClient,
    ctx: &Context,
    command: ApolloConversationsCommand,
) -> Result<Value, AppError> {
    match command.action {
        ApolloConversationsAction::Search(args) => {
            let mut body = json_body("conversations.search", args.json)?;
            set_string(&mut body, "conversation_type", args.conversation_type);
            set_string(&mut body, "account_id", args.account_id);
            set_string(&mut body, "sort_by_field", args.sort_by_field);
            paginate_body_page_only(
                client,
                ctx,
                "conversations.search",
                "/conversations/search",
                body,
                args.limit,
                100,
            )
            .await
        }
        ApolloConversationsAction::Get(args) => {
            get(
                client,
                ctx,
                "conversations.get",
                &format!("/conversations/{}", enc(&args.id)),
                Query::new(),
            )
            .await
        }
        ApolloConversationsAction::Export(args) => {
            post_json_args(
                client,
                ctx,
                "conversations.export",
                "/conversations/export",
                args,
            )
            .await
        }
        ApolloConversationsAction::GetExport(args) => {
            get(
                client,
                ctx,
                "conversations.get-export",
                &format!("/conversations/export/{}", enc(&args.id)),
                Query::new(),
            )
            .await
        }
    }
}

fn search_query(args: ApolloSearchArgs) -> Result<(Query, u32), AppError> {
    let mut query = Query::from_pairs(args.json.query)?;
    query.push("q_keywords", args.q_keywords.as_deref());
    query.push("q_name", args.q_name.as_deref());
    query.push("q_organization_name", args.q_name.as_deref());
    query.push("sort_by_field", args.sort_by_field.as_deref());
    query.push_bool_opt("sort_ascending", args.sort_ascending);
    if let Some(title) = args.title.as_deref() {
        query.push_value("person_titles[]", title);
    }
    if let Some(location) = args.location.as_deref() {
        query.push_value("person_locations[]", location);
        query.push_value("organization_locations[]", location);
    }
    if let Some(domain) = args.domain.as_deref() {
        query.push_value("q_organization_domains_list[]", domain);
    }
    Ok((query, args.limit))
}

fn search_body(args: ApolloSearchArgs) -> Result<(Value, u32), AppError> {
    let limit = args.limit;
    let mut body = json_body("search", args.json)?;
    set_string(&mut body, "q_keywords", args.q_keywords);
    set_string(&mut body, "q_organization_name", args.q_name);
    set_string(&mut body, "sort_by_field", args.sort_by_field);
    set_bool(&mut body, "sort_ascending", args.sort_ascending);
    Ok((body, limit))
}

fn contact_body(args: ApolloContactWrite) -> Result<Value, AppError> {
    let mut body = json_body("contacts.write", args.json)?;
    set_string(&mut body, "first_name", args.first_name);
    set_string(&mut body, "last_name", args.last_name);
    set_string(&mut body, "organization_name", args.organization_name);
    set_string(&mut body, "title", args.title);
    set_string(&mut body, "account_id", args.account_id);
    set_string(&mut body, "email", args.email);
    set_string(&mut body, "website_url", args.website_url);
    set_string(&mut body, "contact_stage_id", args.contact_stage_id);
    Ok(body)
}

fn account_body(args: ApolloAccountWrite) -> Result<Value, AppError> {
    let mut body = json_body("accounts.write", args.json)?;
    set_string(&mut body, "name", args.name);
    set_string(&mut body, "domain", args.domain);
    set_string(&mut body, "owner_id", args.owner_id);
    set_string(&mut body, "account_stage_id", args.account_stage_id);
    set_string(&mut body, "phone", args.phone);
    set_string(&mut body, "raw_address", args.raw_address);
    Ok(body)
}

fn deal_body(args: ApolloDealWrite) -> Result<Value, AppError> {
    let mut body = json_body("deals.write", args.json)?;
    set_string(&mut body, "name", args.name);
    set_string(&mut body, "owner_id", args.owner_id);
    set_string(&mut body, "account_id", args.account_id);
    set_f64(&mut body, "amount", args.amount);
    set_string(&mut body, "opportunity_stage_id", args.opportunity_stage_id);
    set_string(&mut body, "closed_date", args.closed_date);
    Ok(body)
}

fn task_body(args: ApolloTaskWrite) -> Result<Value, AppError> {
    let mut body = json_body("tasks.write", args.json)?;
    set_string(&mut body, "user_id", args.user_id);
    set_string(&mut body, "contact_id", args.contact_id);
    set_string(&mut body, "type", args.task_type);
    set_string(&mut body, "priority", args.priority);
    set_string(&mut body, "status", args.status);
    set_string(&mut body, "due_at", args.due_at);
    set_string(&mut body, "title", args.title);
    set_string(&mut body, "note", args.note);
    Ok(body)
}

fn call_query(args: ApolloCallWrite) -> Result<Query, AppError> {
    let mut query = Query::from_pairs(args.json.query)?;
    query.push("contact_id", args.contact_id.as_deref());
    query.push("account_id", args.account_id.as_deref());
    query.push("to_number", args.to_number.as_deref());
    query.push("from_number", args.from_number.as_deref());
    query.push("status", args.status.as_deref());
    query.push("start_time", args.start_time.as_deref());
    query.push("end_time", args.end_time.as_deref());
    query.push_u64("duration", args.duration);
    query.push("note", args.note.as_deref());
    Ok(query)
}

fn field_body(args: ApolloFieldWrite) -> Result<Value, AppError> {
    let mut body = json_body("fields.create", args.json)?;
    set_string(&mut body, "label", args.label);
    set_string(&mut body, "modality", args.modality);
    set_string(&mut body, "type", args.field_type);
    Ok(body)
}

fn sequence_body(args: ApolloSequenceWrite) -> Result<Value, AppError> {
    let mut body = json_body("sequences.write", args.json)?;
    set_string(&mut body, "name", args.name);
    set_bool(&mut body, "active", args.active);
    set_string(&mut body, "user_id", args.user_id);
    Ok(body)
}

fn email_body(args: ApolloEmailDraft) -> Result<Value, AppError> {
    let mut body = json_body("emails.draft", args.json)?;
    set_string(&mut body, "contact_id", args.contact_id);
    set_string(&mut body, "subject", args.subject);
    set_string(&mut body, "body_html", args.body_html);
    Ok(body)
}

async fn ids_update(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    path: &str,
    ids_key: &str,
    value_key: &str,
    args: ApolloIdsUpdate,
) -> Result<Value, AppError> {
    let mut query = Query::new();
    query.push_list(ids_key, &args.ids);
    let value = args.owner_id.or(args.stage_id).ok_or_else(|| {
        AppError::invalid_input(
            SERVICE,
            operation,
            format!("--{} is required", value_key.replace('_', "-")),
        )
    })?;
    query.push_value(value_key, value);
    post_with_query(client, ctx, operation, path, query, None).await
}

async fn get(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    path: &str,
    query: Query,
) -> Result<Value, AppError> {
    let mut url = format!("{}{}", apollo_base(ctx.profile()), path);
    query.append_to(&mut url);
    request(client, ctx, operation, Method::GET, url, None).await
}

async fn post_json_args(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    path: &str,
    args: ApolloJsonArgs,
) -> Result<Value, AppError> {
    let query = Query::from_pairs(args.query.clone())?;
    let body = json_body(operation, args)?;
    post_with_query(client, ctx, operation, path, query, Some(body)).await
}

async fn post(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    path: &str,
    body: Option<Value>,
) -> Result<Value, AppError> {
    request(
        client,
        ctx,
        operation,
        Method::POST,
        format!("{}{}", apollo_base(ctx.profile()), path),
        body,
    )
    .await
}

async fn patch(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    path: &str,
    body: Option<Value>,
) -> Result<Value, AppError> {
    request(
        client,
        ctx,
        operation,
        Method::PATCH,
        format!("{}{}", apollo_base(ctx.profile()), path),
        body,
    )
    .await
}

async fn put(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    path: &str,
    body: Option<Value>,
) -> Result<Value, AppError> {
    request(
        client,
        ctx,
        operation,
        Method::PUT,
        format!("{}{}", apollo_base(ctx.profile()), path),
        body,
    )
    .await
}

async fn post_with_query(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    path: &str,
    query: Query,
    body: Option<Value>,
) -> Result<Value, AppError> {
    let mut url = format!("{}{}", apollo_base(ctx.profile()), path);
    query.append_to(&mut url);
    request(client, ctx, operation, Method::POST, url, body).await
}

async fn put_with_query(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    path: &str,
    query: Query,
    body: Option<Value>,
) -> Result<Value, AppError> {
    let mut url = format!("{}{}", apollo_base(ctx.profile()), path);
    query.append_to(&mut url);
    request(client, ctx, operation, Method::PUT, url, body).await
}

async fn request(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    method: Method,
    url: String,
    body: Option<Value>,
) -> Result<Value, AppError> {
    client
        .request(SERVICE, operation, ctx.profile(), method, url, body)
        .await
}

async fn paginate_query(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    path: &str,
    method: Method,
    (mut query, limit): (Query, u32),
    max_per_page: u32,
) -> Result<Value, AppError> {
    paginate(
        client,
        ctx,
        operation,
        limit,
        max_per_page,
        |page, per_page| {
            query.set("page", page.to_string());
            query.set("per_page", per_page.to_string());
            let mut url = format!("{}{}", apollo_base(ctx.profile()), path);
            query.append_to(&mut url);
            (method.clone(), url, None)
        },
    )
    .await
    .map(|page| aggregate(page, limit))
}

async fn paginate_body(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    path: &str,
    body: Value,
    limit: u32,
    max_per_page: u32,
) -> Result<Value, AppError> {
    paginate(
        client,
        ctx,
        operation,
        limit,
        max_per_page,
        |page, per_page| {
            let mut body = body.clone();
            set_u64(&mut body, "page", page as u64);
            set_u64(&mut body, "per_page", per_page as u64);
            (
                Method::POST,
                format!("{}{}", apollo_base(ctx.profile()), path),
                Some(body),
            )
        },
    )
    .await
    .map(|page| aggregate(page, limit))
}

async fn paginate_body_page_only(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    path: &str,
    body: Value,
    limit: u32,
    _max_per_page: u32,
) -> Result<Value, AppError> {
    paginate(client, ctx, operation, limit, 100, |page, per_page| {
        let mut body = body.clone();
        set_u64(&mut body, "page", page as u64);
        set_u64(&mut body, "num_fetch_result", per_page as u64);
        (
            Method::POST,
            format!("{}{}", apollo_base(ctx.profile()), path),
            Some(body),
        )
    })
    .await
    .map(|page| aggregate(page, limit))
}

async fn paginate_skip(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    path: &str,
    mut query: Query,
    limit: u32,
    max_per_page: u32,
) -> Result<Value, AppError> {
    if limit == 0 {
        return Ok(json!({ "notes": [] }));
    }
    let mut skip = 0_u32;
    let mut first = None;
    let mut values = Vec::new();
    loop {
        let page_size = (limit - values.len() as u32).clamp(1, max_per_page);
        query.set("skip", skip.to_string());
        query.set("limit", page_size.to_string());
        let mut url = format!("{}{}", apollo_base(ctx.profile()), path);
        query.append_to(&mut url);
        let page = request(client, ctx, operation, Method::GET, url, None).await?;
        if first.is_none() {
            first = Some(page.clone());
        }
        let page_values = collection_values(&page);
        let count = page_values.len();
        values.extend(
            page_values
                .into_iter()
                .take((limit as usize).saturating_sub(values.len())),
        );
        if values.len() >= limit as usize || count < page_size as usize {
            break;
        }
        skip += page_size;
    }
    Ok(first
        .unwrap_or_else(|| json!({ "notes": [] }))
        .with_values(values))
}

async fn paginate<F>(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    limit: u32,
    max_per_page: u32,
    mut build: F,
) -> Result<PageAggregate, AppError>
where
    F: FnMut(u32, u32) -> (Method, String, Option<Value>),
{
    if limit == 0 {
        return Ok(PageAggregate::empty());
    }
    let mut page_no = 1;
    let mut first = None;
    let mut values = Vec::new();
    loop {
        let per_page = (limit - values.len() as u32).clamp(1, max_per_page);
        let (method, url, body) = build(page_no, per_page);
        let page = request(client, ctx, operation, method, url, body).await?;
        if first.is_none() {
            first = Some(page.clone());
        }
        let page_values = collection_values(&page);
        let count = page_values.len();
        values.extend(
            page_values
                .into_iter()
                .take((limit as usize).saturating_sub(values.len())),
        );
        if values.len() >= limit as usize || count < per_page as usize {
            break;
        }
        page_no += 1;
    }
    Ok(PageAggregate {
        first: first.unwrap_or_else(|| json!({ "results": [] })),
        values,
    })
}

#[derive(Debug)]
struct PageAggregate {
    first: Value,
    values: Vec<Value>,
}

impl PageAggregate {
    fn empty() -> Self {
        Self {
            first: json!({ "results": [] }),
            values: Vec::new(),
        }
    }
}

trait WithValues {
    fn with_values(self, values: Vec<Value>) -> Value;
}

impl WithValues for Value {
    fn with_values(mut self, values: Vec<Value>) -> Value {
        set_collection_values(&mut self, values);
        self
    }
}

fn aggregate(page: PageAggregate, _limit: u32) -> Value {
    page.first.with_values(page.values)
}

fn collection_values(value: &Value) -> Vec<Value> {
    PAGE_ARRAY_KEYS
        .iter()
        .find_map(|key| value.get(*key).and_then(Value::as_array))
        .or_else(|| value.pointer("/data/items").and_then(Value::as_array))
        .cloned()
        .unwrap_or_default()
}

fn set_collection_values(value: &mut Value, values: Vec<Value>) {
    let key = PAGE_ARRAY_KEYS
        .iter()
        .find(|key| value.get(**key).and_then(Value::as_array).is_some())
        .copied();
    if let Some(key) = key {
        input::ensure_object(value).insert(key.to_string(), Value::Array(values));
    } else if value
        .pointer("/data/items")
        .and_then(Value::as_array)
        .is_some()
    {
        input::ensure_object(value)
            .entry("data")
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .expect("data was object")
            .insert("items".to_string(), Value::Array(values));
    } else {
        input::ensure_object(value).insert("results".to_string(), Value::Array(values));
    }
}

fn json_body(operation: &'static str, args: ApolloJsonArgs) -> Result<Value, AppError> {
    let body = input::read_json_arg(SERVICE, operation, args.json.as_deref())?;
    if !body.is_object() {
        return Err(AppError::invalid_input(
            SERVICE,
            operation,
            "--json must be a JSON object for Apollo commands",
        ));
    }
    Ok(body)
}

fn set_string(body: &mut Value, key: &str, value: Option<String>) {
    if let Some(value) = value {
        input::ensure_object(body).insert(key.to_string(), Value::String(value));
    }
}

fn set_bool(body: &mut Value, key: &str, value: Option<bool>) {
    if let Some(value) = value {
        input::ensure_object(body).insert(key.to_string(), Value::Bool(value));
    }
}

fn set_f64(body: &mut Value, key: &str, value: Option<f64>) {
    if let Some(value) = value.and_then(Number::from_f64) {
        input::ensure_object(body).insert(key.to_string(), Value::Number(value));
    }
}

fn set_u64(body: &mut Value, key: &str, value: u64) {
    input::ensure_object(body).insert(key.to_string(), json!(value));
}

#[derive(Clone, Debug, Default)]
struct Query(Vec<(String, String)>);

impl Query {
    fn new() -> Self {
        Self::default()
    }

    fn from_pairs(pairs: Vec<String>) -> Result<Self, AppError> {
        let mut query = Self::new();
        for pair in pairs {
            let (key, value) = pair.split_once('=').ok_or_else(|| {
                AppError::invalid_input(
                    SERVICE,
                    "query",
                    format!("query parameter must use key=value: {pair}"),
                )
            })?;
            query.push_value(key, value);
        }
        Ok(query)
    }

    fn push(&mut self, key: &str, value: Option<&str>) {
        if let Some(value) = value.filter(|value| !value.is_empty()) {
            self.push_value(key, value);
        }
    }

    fn push_value(&mut self, key: &str, value: impl ToString) {
        self.0.push((key.to_string(), value.to_string()));
    }

    fn push_bool(&mut self, key: &str, value: bool) {
        if value {
            self.push_value(key, "true");
        }
    }

    fn push_bool_opt(&mut self, key: &str, value: Option<bool>) {
        if let Some(value) = value {
            self.push_value(key, value);
        }
    }

    fn push_u64(&mut self, key: &str, value: Option<u64>) {
        if let Some(value) = value {
            self.push_value(key, value);
        }
    }

    fn push_list(&mut self, key: &str, csv: &str) {
        for value in csv
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            self.push_value(key, value);
        }
    }

    fn set(&mut self, key: &str, value: String) {
        self.0.retain(|(existing, _)| existing != key);
        self.0.push((key.to_string(), value));
    }

    fn append_to(&self, url: &mut String) {
        for (index, (key, value)) in self.0.iter().enumerate() {
            url.push(if index == 0 { '?' } else { '&' });
            url.push_str(&enc(key));
            url.push('=');
            url.push_str(&enc(value));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Profile;

    #[test]
    fn apollo_base_defaults_to_api_v1() {
        assert_eq!(
            apollo_base(&Profile::default()),
            "https://api.apollo.io/api/v1"
        );
    }

    #[test]
    fn typed_flags_override_json_fields() {
        let args = ApolloContactWrite {
            json: ApolloJsonArgs {
                json: Some(r#"{"email":"old@example.com","first_name":"Old"}"#.to_string()),
                query: Vec::new(),
            },
            first_name: Some("Ada".to_string()),
            last_name: None,
            organization_name: None,
            title: None,
            account_id: None,
            email: Some("ada@example.com".to_string()),
            website_url: None,
            contact_stage_id: None,
        };
        let body = contact_body(args).unwrap();
        assert_eq!(body["first_name"], "Ada");
        assert_eq!(body["email"], "ada@example.com");
    }
}
