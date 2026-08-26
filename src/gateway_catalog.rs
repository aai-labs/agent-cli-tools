use reqwest::Method;

/// Conservative method checks for the stable typed-operation namespace.
///
/// The gateway still validates the destination host against the stored profile;
/// this catalog check prevents a caller from relabeling an obvious write as a
/// read (or using the generic request escape hatch).
pub fn allows(service: &str, operation: &str, method: &Method) -> bool {
    if service.trim().is_empty()
        || operation.trim().is_empty()
        || operation == "request"
        || operation.contains('/')
        || operation.contains(' ')
    {
        return false;
    }
    let suffix = operation.rsplit('.').next().unwrap_or_default();
    let read = matches!(
        suffix,
        "about"
            | "check"
            | "diff"
            | "download"
            | "drives"
            | "export"
            | "files"
            | "get"
            | "health"
            | "info"
            | "list"
            | "logs"
            | "me"
            | "metrics"
            | "permissions"
            | "query"
            | "search"
            | "source"
    );
    let write = matches!(
        suffix,
        "add"
            | "clear"
            | "comment"
            | "copy"
            | "create"
            | "delete"
            | "merge"
            | "move"
            | "rename"
            | "remove"
            | "send"
            | "update"
            | "upload"
    );
    if read && !matches!(method, &Method::GET | &Method::HEAD) {
        return false;
    }
    if write && matches!(method, &Method::GET | &Method::HEAD) {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_and_gateway_method_catalog_agree() {
        assert!(allows("github", "repos.list", &Method::GET));
        assert!(!allows("github", "repos.list", &Method::POST));
        assert!(allows("github", "issues.create", &Method::POST));
        assert!(!allows("github", "issues.create", &Method::GET));
    }
}
