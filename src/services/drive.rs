use std::{ffi::OsStr, path::Path};

use reqwest::Method;
use serde_json::{json, Map, Value};

use crate::{
    cli::*,
    config::Context,
    error::AppError,
    http::{multipart_boundary, ApiClient, BytesRequest},
    services::shared::{enc, google_base, write_download, CtxProfile},
};

const SERVICE: &str = "drive";

const FOLDER_MIME: &str = "application/vnd.google-apps.folder";
const SHORTCUT_MIME: &str = "application/vnd.google-apps.shortcut";
/// Every Google-native type shares this prefix. Native docs hold no bytes of their
/// own, so `alt=media` fails for them and `files.export` is the only way to read one.
const NATIVE_MIME_PREFIX: &str = "application/vnd.google-apps.";

/// Drive documents multipart upload for payloads of 5 MB or less; anything larger must
/// go through a resumable session, so the size decides the protocol, not the caller.
/// The threshold is decimal on purpose (5_000_000, not 5 * 1024 * 1024) so a file near
/// the boundary falls to the resumable path, which has no size limit, rather than to a
/// rejection.
const MULTIPART_UPLOAD_MAX_BYTES: usize = 5_000_000;

/// Drive's own page-size ceilings. Asking for more is a 400.
const FILES_PAGE_SIZE_CAP: u32 = 1000;
const DRIVES_PAGE_SIZE_CAP: u32 = 100;
const PERMISSIONS_PAGE_SIZE_CAP: u32 = 100;

/// Drive's default projection is only `kind,id,name,mimeType,resourceKey`, which omits
/// everything an agent needs to decide what to do with a file. Every request therefore
/// sends an explicit projection.
const FILE_LIST_FIELDS: &str = "id,name,mimeType,size,createdTime,modifiedTime,parents,driveId,trashed,webViewLink,owners(displayName,emailAddress)";
const FILE_GET_FIELDS: &str = "id,name,mimeType,size,md5Checksum,createdTime,modifiedTime,parents,driveId,trashed,shared,description,webViewLink,webContentLink,owners(displayName,emailAddress),lastModifyingUser(displayName,emailAddress),capabilities(canDownload,canEdit,canShare),shortcutDetails(targetId,targetMimeType),exportLinks";
const FILE_UPLOAD_FIELDS: &str =
    "id,name,mimeType,size,parents,driveId,createdTime,modifiedTime,webViewLink,webContentLink";
/// Enough to decide between `alt=media` and `files.export`, and to name the file in
/// the download result.
const FILE_DOWNLOAD_PROBE_FIELDS: &str =
    "id,name,mimeType,size,shortcutDetails(targetId,targetMimeType)";
const DRIVE_LIST_FIELDS: &str = "id,name,createdTime,hidden";
const DRIVE_GET_FIELDS: &str = "id,name,createdTime,hidden,orgUnitId,restrictions,capabilities";
const PERMISSION_FIELDS: &str =
    "id,type,role,emailAddress,domain,displayName,allowFileDiscovery,deleted,pendingOwner";
const ABOUT_FIELDS: &str = "user,storageQuota,exportFormats,maxUploadSize,canCreateDrives";

pub(crate) async fn dispatch(
    client: &ApiClient,
    ctx: &Context,
    command: DriveCommand,
) -> Result<Value, AppError> {
    match command.resource {
        DriveResource::Files(command) => files(client, ctx, command.action).await,
        DriveResource::Folders(command) => folders(client, ctx, command.action).await,
        DriveResource::Drives(command) => drives(client, ctx, command.action).await,
        DriveResource::Permissions(command) => permissions(client, ctx, command.action).await,
        DriveResource::About(command) => about(client, ctx, command.action).await,
    }
}

async fn files(
    client: &ApiClient,
    ctx: &Context,
    action: DriveFilesAction,
) -> Result<Value, AppError> {
    match action {
        DriveFilesAction::List(args) => {
            let query = build_file_query(&FileQuery {
                parent: args.parent.as_deref(),
                name_contains: args.name_contains.as_deref(),
                mime_type: args.mime_type.as_deref(),
                raw: args.q.as_deref(),
                include_trashed: args.include_trashed,
            });
            let request = DriveList {
                operation: "files.list",
                collection: "files",
                url: files_list_url(
                    ctx,
                    query.as_deref(),
                    args.order_by.as_deref(),
                    args.fields.as_deref().unwrap_or(FILE_LIST_FIELDS),
                    args.drive_id.as_deref(),
                ),
                page_size_cap: FILES_PAGE_SIZE_CAP,
                limit: args.limit,
                page_token: args.page_token,
            };
            paginate(client, ctx, request).await
        }
        DriveFilesAction::Get(args) => {
            get_file(
                client,
                ctx,
                "files.get",
                &args.file_id,
                args.fields.as_deref().unwrap_or(FILE_GET_FIELDS),
            )
            .await
        }
        DriveFilesAction::Download(args) => download(client, ctx, args).await,
        DriveFilesAction::Upload(args) => upload(client, ctx, args).await,
    }
}

async fn folders(
    client: &ApiClient,
    ctx: &Context,
    action: DriveFoldersAction,
) -> Result<Value, AppError> {
    match action {
        // A folder is not its own Drive resource — it is a file with the folder MIME
        // type. This command is that filter, so agents do not have to know the type
        // string to browse a tree.
        DriveFoldersAction::List(args) => {
            let query = build_file_query(&FileQuery {
                parent: args.parent.as_deref(),
                name_contains: args.name_contains.as_deref(),
                mime_type: Some(FOLDER_MIME),
                raw: None,
                include_trashed: false,
            });
            let request = DriveList {
                operation: "folders.list",
                collection: "files",
                url: files_list_url(
                    ctx,
                    query.as_deref(),
                    Some("name"),
                    FILE_LIST_FIELDS,
                    args.drive_id.as_deref(),
                ),
                page_size_cap: FILES_PAGE_SIZE_CAP,
                limit: args.limit,
                page_token: args.page_token,
            };
            paginate(client, ctx, request).await
        }
        DriveFoldersAction::Get(args) => {
            get_file(
                client,
                ctx,
                "folders.get",
                &args.folder_id,
                args.fields.as_deref().unwrap_or(FILE_GET_FIELDS),
            )
            .await
        }
    }
}

async fn drives(
    client: &ApiClient,
    ctx: &Context,
    action: DriveDrivesAction,
) -> Result<Value, AppError> {
    match action {
        DriveDrivesAction::List(args) => {
            let request = DriveList {
                operation: "drives.list",
                collection: "drives",
                url: format!(
                    "{}/drive/v3/drives?fields={}",
                    google_base(ctx.profile()),
                    enc(&format!("nextPageToken,drives({DRIVE_LIST_FIELDS})"))
                ),
                page_size_cap: DRIVES_PAGE_SIZE_CAP,
                limit: args.limit,
                page_token: args.page_token,
            };
            paginate(client, ctx, request).await
        }
        DriveDrivesAction::Get(args) => {
            let url = format!(
                "{}/drive/v3/drives/{}?fields={}",
                google_base(ctx.profile()),
                enc(&args.drive_id),
                enc(DRIVE_GET_FIELDS)
            );
            client
                .request(SERVICE, "drives.get", ctx.profile(), Method::GET, url, None)
                .await
        }
    }
}

async fn permissions(
    client: &ApiClient,
    ctx: &Context,
    action: DrivePermissionsAction,
) -> Result<Value, AppError> {
    match action {
        DrivePermissionsAction::List(args) => {
            let request = DriveList {
                operation: "permissions.list",
                collection: "permissions",
                url: format!(
                    "{}/drive/v3/files/{}/permissions?supportsAllDrives=true&fields={}",
                    google_base(ctx.profile()),
                    enc(&args.file_id),
                    enc(&format!("nextPageToken,permissions({PERMISSION_FIELDS})"))
                ),
                page_size_cap: PERMISSIONS_PAGE_SIZE_CAP,
                limit: args.limit,
                page_token: args.page_token,
            };
            paginate(client, ctx, request).await
        }
        DrivePermissionsAction::Get(args) => {
            let url = format!(
                "{}/drive/v3/files/{}/permissions/{}?supportsAllDrives=true&fields={}",
                google_base(ctx.profile()),
                enc(&args.file_id),
                enc(&args.permission_id),
                enc(PERMISSION_FIELDS)
            );
            client
                .request(
                    SERVICE,
                    "permissions.get",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
        }
    }
}

async fn about(
    client: &ApiClient,
    ctx: &Context,
    action: DriveAboutAction,
) -> Result<Value, AppError> {
    let DriveAboutAction::Get = action;
    // about.get has no default projection at all: without `fields` it is a 400.
    let url = format!(
        "{}/drive/v3/about?fields={}",
        google_base(ctx.profile()),
        enc(ABOUT_FIELDS)
    );
    client
        .request(SERVICE, "about.get", ctx.profile(), Method::GET, url, None)
        .await
}

async fn get_file(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    file_id: &str,
    fields: &str,
) -> Result<Value, AppError> {
    let url = format!(
        "{}/drive/v3/files/{}?supportsAllDrives=true&fields={}",
        google_base(ctx.profile()),
        enc(file_id),
        enc(fields)
    );
    client
        .request(SERVICE, operation, ctx.profile(), Method::GET, url, None)
        .await
}

/// Read file content and write it to `--output`.
///
/// The MIME type decides the mechanism, so it is fetched first: Google-native docs
/// must go through `files.export`, everything else through `alt=media`. Content never
/// reaches stdout — the command returns metadata about what was written.
async fn download(
    client: &ApiClient,
    ctx: &Context,
    args: DriveFilesDownloadArgs,
) -> Result<Value, AppError> {
    let operation = "files.download";
    let metadata = get_file(
        client,
        ctx,
        operation,
        &args.file_id,
        FILE_DOWNLOAD_PROBE_FIELDS,
    )
    .await?;
    let mime_type = metadata
        .get("mimeType")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let name = metadata
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    if mime_type == FOLDER_MIME {
        return Err(AppError::invalid_input(
            SERVICE,
            operation,
            format!("{name:?} is a folder and has no content; list its children with `drive files list --parent {}`", args.file_id),
        ));
    }
    if mime_type == SHORTCUT_MIME {
        let target = metadata
            .pointer("/shortcutDetails/targetId")
            .and_then(Value::as_str)
            .unwrap_or("<unknown>");
        return Err(AppError::invalid_input(
            SERVICE,
            operation,
            format!("{name:?} is a shortcut and has no content of its own; download its target {target} instead"),
        ));
    }

    let export_mime = if mime_type.starts_with(NATIVE_MIME_PREFIX) {
        Some(
            args.mime_type
                .unwrap_or_else(|| default_export_mime(&mime_type).to_string()),
        )
    } else {
        None
    };

    let url = match &export_mime {
        // files.export documents only fileId and mimeType, and caps the converted
        // result at 10 MB, so it is sent nothing else.
        Some(target) => format!(
            "{}/drive/v3/files/{}/export?mimeType={}",
            google_base(ctx.profile()),
            enc(&args.file_id),
            enc(target)
        ),
        None => format!(
            "{}/drive/v3/files/{}?alt=media&supportsAllDrives=true",
            google_base(ctx.profile()),
            enc(&args.file_id)
        ),
    };

    let bytes = client
        .download(SERVICE, operation, ctx.profile(), url)
        .await?;
    let mut result = write_download(SERVICE, operation, &args.output, &bytes)?;
    result["file_id"] = json!(args.file_id);
    result["name"] = json!(name);
    result["mime_type"] = json!(mime_type);
    result["exported_mime_type"] = json!(export_mime);
    Ok(result)
}

/// Create a new Drive file from local bytes.
///
/// The upload protocol is picked from the payload size rather than exposed as a flag:
/// multipart up to Drive's 5 MB limit, a resumable session above it.
async fn upload(
    client: &ApiClient,
    ctx: &Context,
    args: DriveFilesUploadArgs,
) -> Result<Value, AppError> {
    let operation = "files.upload";
    let bytes = std::fs::read(&args.file).map_err(|err| {
        AppError::invalid_input(
            SERVICE,
            operation,
            format!("failed to read {}: {err}", args.file),
        )
    })?;
    let default_name = local_file_name(&args.file);
    let name = args.name.unwrap_or(default_name);
    let media_type = args
        .mime_type
        .unwrap_or_else(|| guess_upload_mime(&name).to_string());

    let mut metadata = Map::new();
    metadata.insert("name".to_string(), json!(name));
    metadata.insert("mimeType".to_string(), json!(media_type));
    if let Some(parent) = &args.parent {
        metadata.insert("parents".to_string(), json!([parent]));
    }
    if let Some(description) = &args.description {
        metadata.insert("description".to_string(), json!(description));
    }
    let metadata = Value::Object(metadata);

    match upload_strategy(bytes.len()) {
        UploadStrategy::Multipart => {
            let boundary = multipart_boundary();
            let request = BytesRequest {
                method: Method::POST,
                url: upload_url(ctx, "multipart"),
                content_type: format!("multipart/related; boundary={boundary}"),
                headers: Vec::new(),
                body: multipart_related_body(&boundary, &metadata, &media_type, &bytes),
            };
            let response = client
                .request_bytes(SERVICE, operation, ctx.profile(), request)
                .await?;
            Ok(response.body)
        }
        UploadStrategy::Resumable => {
            let total = bytes.len();
            let session = client
                .request_bytes(
                    SERVICE,
                    operation,
                    ctx.profile(),
                    BytesRequest {
                        method: Method::POST,
                        url: upload_url(ctx, "resumable"),
                        content_type: "application/json; charset=UTF-8".to_string(),
                        headers: vec![
                            ("X-Upload-Content-Type".to_string(), media_type.clone()),
                            ("X-Upload-Content-Length".to_string(), total.to_string()),
                        ],
                        body: metadata.to_string().into_bytes(),
                    },
                )
                .await?;
            let session_url = session.location.ok_or_else(|| {
                AppError::internal(
                    SERVICE,
                    operation,
                    "resumable upload session response had no Location header",
                )
            })?;
            let response = client
                .request_bytes(
                    SERVICE,
                    operation,
                    ctx.profile(),
                    BytesRequest {
                        method: Method::PUT,
                        url: session_url,
                        content_type: media_type,
                        headers: vec![(
                            "Content-Range".to_string(),
                            format!("bytes 0-{}/{total}", total - 1),
                        )],
                        body: bytes,
                    },
                )
                .await?;
            Ok(response.body)
        }
    }
}

fn upload_url(ctx: &Context, upload_type: &str) -> String {
    format!(
        "{}/upload/drive/v3/files?uploadType={upload_type}&supportsAllDrives=true&fields={}",
        google_base(ctx.profile()),
        enc(FILE_UPLOAD_FIELDS)
    )
}

/// One list request, minus the paging parameters `paginate` owns.
struct DriveList {
    operation: &'static str,
    collection: &'static str,
    url: String,
    page_size_cap: u32,
    limit: u32,
    page_token: Option<String>,
}

/// Follow `nextPageToken` until `--limit` items are collected or Drive runs out.
///
/// The last page's envelope is kept and only its collection array is replaced, so the
/// provider response shape survives aggregation. A surviving `nextPageToken` means
/// there is more beyond `--limit`, which is what `_aai.pagination` reports.
async fn paginate(
    client: &ApiClient,
    ctx: &Context,
    request: DriveList,
) -> Result<Value, AppError> {
    let limit = request.limit.max(1) as usize;
    let mut accumulated: Vec<Value> = Vec::new();
    let mut page_token = request.page_token;

    // The loop yields the final page's envelope, which is what the aggregated
    // collection is spliced back into.
    let mut envelope = loop {
        let page_size = (limit - accumulated.len()).min(request.page_size_cap as usize);
        let mut url = format!("{}&pageSize={page_size}", request.url);
        if let Some(token) = &page_token {
            url.push_str(&format!("&pageToken={}", enc(token)));
        }
        let response = client
            .request(
                SERVICE,
                request.operation,
                ctx.profile(),
                Method::GET,
                url,
                None,
            )
            .await?;
        let Some(object) = response.as_object().cloned() else {
            return Err(AppError::internal(
                SERVICE,
                request.operation,
                "expected a JSON object from Drive",
            ));
        };
        let page = object
            .get(request.collection)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let page_len = page.len();
        accumulated.extend(page);
        page_token = object
            .get("nextPageToken")
            .and_then(Value::as_str)
            .map(str::to_string);

        if page_len == 0 || page_token.is_none() || accumulated.len() >= limit {
            break object;
        }
    };

    envelope.insert(request.collection.to_string(), Value::Array(accumulated));
    // Keeping the token only when pages remain is what turns `_aai.pagination` into
    // `more_available` versus a listing that is genuinely finished.
    if let Some(token) = page_token {
        envelope.insert("nextPageToken".to_string(), Value::String(token));
    } else {
        envelope.remove("nextPageToken");
    }
    Ok(Value::Object(envelope))
}

fn files_list_url(
    ctx: &Context,
    query: Option<&str>,
    order_by: Option<&str>,
    file_fields: &str,
    drive_id: Option<&str>,
) -> String {
    let mut url = format!(
        "{}/drive/v3/files?fields={}",
        google_base(ctx.profile()),
        enc(&format!(
            "nextPageToken,incompleteSearch,files({file_fields})"
        ))
    );
    // Without both of these, content on shared drives is silently missing from the
    // response rather than reported as inaccessible.
    url.push_str("&supportsAllDrives=true&includeItemsFromAllDrives=true");
    match drive_id {
        Some(drive_id) => url.push_str(&format!("&corpora=drive&driveId={}", enc(drive_id))),
        None => url.push_str("&corpora=allDrives"),
    }
    if let Some(query) = query {
        url.push_str(&format!("&q={}", enc(query)));
    }
    if let Some(order_by) = order_by {
        url.push_str(&format!("&orderBy={}", enc(order_by)));
    }
    url
}

struct FileQuery<'a> {
    parent: Option<&'a str>,
    name_contains: Option<&'a str>,
    mime_type: Option<&'a str>,
    raw: Option<&'a str>,
    include_trashed: bool,
}

/// Assemble the filters into one Drive query string.
///
/// Trashed files are excluded unless asked for: a listing that quietly includes
/// deleted files is worse than one that misses them.
fn build_file_query(query: &FileQuery) -> Option<String> {
    let mut clauses: Vec<String> = Vec::new();
    if !query.include_trashed {
        clauses.push("trashed = false".to_string());
    }
    if let Some(parent) = query.parent {
        clauses.push(format!("'{}' in parents", escape_query_literal(parent)));
    }
    if let Some(mime_type) = query.mime_type {
        clauses.push(format!("mimeType = '{}'", escape_query_literal(mime_type)));
    }
    if let Some(name) = query.name_contains {
        clauses.push(format!("name contains '{}'", escape_query_literal(name)));
    }
    if let Some(raw) = query.raw.map(str::trim).filter(|raw| !raw.is_empty()) {
        clauses.push(format!("({raw})"));
    }
    (!clauses.is_empty()).then(|| clauses.join(" and "))
}

/// Drive query literals are single-quoted, so a quote or backslash in a folder ID or
/// file name would otherwise terminate the clause early.
fn escape_query_literal(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

fn default_export_mime(native_mime: &str) -> &'static str {
    match native_mime {
        "application/vnd.google-apps.document" => {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        }
        "application/vnd.google-apps.spreadsheet" => {
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        }
        "application/vnd.google-apps.presentation" => {
            "application/vnd.openxmlformats-officedocument.presentationml.presentation"
        }
        "application/vnd.google-apps.drawing" => "image/png",
        "application/vnd.google-apps.script" => "application/vnd.google-apps.script+json",
        // Every other native type (forms, sites, jam boards) converts to PDF or to
        // nothing; PDF is the one an agent can still read.
        _ => "application/pdf",
    }
}

fn guess_upload_mime(name: &str) -> &'static str {
    let extension = Path::new(name)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "txt" | "log" => "text/plain",
        "md" => "text/markdown",
        "csv" => "text/csv",
        "tsv" => "text/tab-separated-values",
        "json" => "application/json",
        "xml" => "application/xml",
        "html" | "htm" => "text/html",
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "zip" => "application/zip",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        _ => "application/octet-stream",
    }
}

fn local_file_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

#[derive(Debug, PartialEq, Eq)]
enum UploadStrategy {
    Multipart,
    Resumable,
}

fn upload_strategy(size: usize) -> UploadStrategy {
    if size <= MULTIPART_UPLOAD_MAX_BYTES {
        UploadStrategy::Multipart
    } else {
        UploadStrategy::Resumable
    }
}

fn multipart_related_body(
    boundary: &str,
    metadata: &Value,
    media_type: &str,
    media: &[u8],
) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(
        format!("--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n").as_bytes(),
    );
    body.extend_from_slice(metadata.to_string().as_bytes());
    body.extend_from_slice(
        format!("\r\n--{boundary}\r\nContent-Type: {media_type}\r\n\r\n").as_bytes(),
    );
    body.extend_from_slice(media);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query(parent: Option<&str>, include_trashed: bool) -> Option<String> {
        build_file_query(&FileQuery {
            parent,
            name_contains: None,
            mime_type: None,
            raw: None,
            include_trashed,
        })
    }

    #[test]
    fn listings_exclude_trashed_files_unless_asked() {
        assert_eq!(query(None, false).as_deref(), Some("trashed = false"));
        assert_eq!(query(None, true), None);
    }

    #[test]
    fn a_parent_scopes_the_listing_to_that_folder() {
        assert_eq!(
            query(Some("1AbC"), false).as_deref(),
            Some("trashed = false and '1AbC' in parents")
        );
    }

    #[test]
    fn filters_and_a_raw_query_combine_with_and() {
        let combined = build_file_query(&FileQuery {
            parent: Some("1AbC"),
            name_contains: Some("report"),
            mime_type: Some(FOLDER_MIME),
            raw: Some("modifiedTime > '2026-01-01T00:00:00'"),
            include_trashed: false,
        });
        assert_eq!(
            combined.as_deref(),
            Some(
                "trashed = false and '1AbC' in parents \
                 and mimeType = 'application/vnd.google-apps.folder' \
                 and name contains 'report' \
                 and (modifiedTime > '2026-01-01T00:00:00')"
            )
        );
    }

    #[test]
    fn query_literals_cannot_break_out_of_their_quotes() {
        let escaped = build_file_query(&FileQuery {
            parent: None,
            name_contains: Some("o'brien\\"),
            mime_type: None,
            raw: None,
            include_trashed: true,
        });
        assert_eq!(escaped.as_deref(), Some(r"name contains 'o\'brien\\'"));
    }

    #[test]
    fn native_docs_export_to_openable_office_formats() {
        assert_eq!(
            default_export_mime("application/vnd.google-apps.document"),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        );
        assert_eq!(
            default_export_mime("application/vnd.google-apps.spreadsheet"),
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        );
        assert_eq!(
            default_export_mime("application/vnd.google-apps.presentation"),
            "application/vnd.openxmlformats-officedocument.presentationml.presentation"
        );
        assert_eq!(
            default_export_mime("application/vnd.google-apps.form"),
            "application/pdf"
        );
    }

    #[test]
    fn upload_protocol_follows_the_payload_size() {
        assert_eq!(upload_strategy(0), UploadStrategy::Multipart);
        assert_eq!(
            upload_strategy(MULTIPART_UPLOAD_MAX_BYTES),
            UploadStrategy::Multipart
        );
        assert_eq!(
            upload_strategy(MULTIPART_UPLOAD_MAX_BYTES + 1),
            UploadStrategy::Resumable
        );
    }

    #[test]
    fn upload_mime_is_guessed_from_the_extension() {
        assert_eq!(guess_upload_mime("notes.md"), "text/markdown");
        assert_eq!(guess_upload_mime("Q3 Report.PDF"), "application/pdf");
        assert_eq!(guess_upload_mime("archive"), "application/octet-stream");
    }

    #[test]
    fn multipart_body_carries_metadata_then_media() {
        let body = multipart_related_body(
            "BOUNDARY",
            &json!({"name": "notes.md"}),
            "text/markdown",
            b"hello",
        );
        let body = String::from_utf8(body).expect("ascii body");
        assert_eq!(
            body,
            "--BOUNDARY\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n\
             {\"name\":\"notes.md\"}\r\n\
             --BOUNDARY\r\nContent-Type: text/markdown\r\n\r\n\
             hello\r\n--BOUNDARY--\r\n"
        );
    }
}
