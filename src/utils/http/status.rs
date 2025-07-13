/// Create a map of HTTP status codes with human-readable descriptions
pub fn get_status_description(status_code: u16) -> String {
    match status_code {
        // 1xx Informational
        100 => "100 Continue".to_string(),
        101 => "101 Switching Protocols".to_string(),
        102 => "102 Processing".to_string(),
        103 => "103 Early Hints".to_string(),

        // 2xx Success
        200 => "200 OK".to_string(),
        201 => "201 Created".to_string(),
        202 => "202 Accepted".to_string(),
        203 => "203 Non-Authoritative Information".to_string(),
        204 => "204 No Content".to_string(),
        205 => "205 Reset Content".to_string(),
        206 => "206 Partial Content".to_string(),
        207 => "207 Multi-Status".to_string(),
        208 => "208 Already Reported".to_string(),
        226 => "226 IM Used".to_string(),

        // 3xx Redirection
        300 => "300 Multiple Choices".to_string(),
        301 => "301 Moved Permanently".to_string(),
        302 => "302 Found".to_string(),
        303 => "303 See Other".to_string(),
        304 => "304 Not Modified".to_string(),
        305 => "305 Use Proxy".to_string(),
        307 => "307 Temporary Redirect".to_string(),
        308 => "308 Permanent Redirect".to_string(),

        // 4xx Client Error
        400 => "400 Bad Request".to_string(),
        401 => "401 Unauthorized".to_string(),
        402 => "402 Payment Required".to_string(),
        403 => "403 Forbidden".to_string(),
        404 => "404 Not Found".to_string(),
        405 => "405 Method Not Allowed".to_string(),
        406 => "406 Not Acceptable".to_string(),
        407 => "407 Proxy Authentication Required".to_string(),
        408 => "408 Request Timeout".to_string(),
        409 => "409 Conflict".to_string(),
        410 => "410 Gone".to_string(),
        411 => "411 Length Required".to_string(),
        412 => "412 Precondition Failed".to_string(),
        413 => "413 Payload Too Large".to_string(),
        414 => "414 URI Too Long".to_string(),
        415 => "415 Unsupported Media Type".to_string(),
        416 => "416 Range Not Satisfiable".to_string(),
        417 => "417 Expectation Failed".to_string(),
        418 => "418 I'm a teapot".to_string(),
        421 => "421 Misdirected Request".to_string(),
        422 => "422 Unprocessable Entity".to_string(),
        423 => "423 Locked".to_string(),
        424 => "424 Failed Dependency".to_string(),
        425 => "425 Too Early".to_string(),
        426 => "426 Upgrade Required".to_string(),
        428 => "428 Precondition Required".to_string(),
        429 => "429 Too Many Requests".to_string(),
        431 => "431 Request Header Fields Too Large".to_string(),
        451 => "451 Unavailable For Legal Reasons".to_string(),

        // 5xx Server Error
        500 => "500 Internal Server Error".to_string(),
        501 => "501 Not Implemented".to_string(),
        502 => "502 Bad Gateway".to_string(),
        503 => "503 Service Unavailable".to_string(),
        504 => "504 Gateway Timeout".to_string(),
        505 => "505 HTTP Version Not Supported".to_string(),
        506 => "506 Variant Also Negotiates".to_string(),
        507 => "507 Insufficient Storage".to_string(),
        508 => "508 Loop Detected".to_string(),
        510 => "510 Not Extended".to_string(),
        511 => "511 Network Authentication Required".to_string(),

        // Default for unknown codes
        _ => format!("{status_code} Unknown"),
    }
}

/// Get just the status text for HTTP status code (without the code number)
pub fn get_status_text(status_code: u16) -> String {
    match status_code {
        // 1xx Informational
        100 => "Continue",
        101 => "Switching Protocols",
        102 => "Processing",
        103 => "Early Hints",

        // 2xx Success
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        203 => "Non-Authoritative Information",
        204 => "No Content",
        205 => "Reset Content",
        206 => "Partial Content",
        207 => "Multi-Status",
        208 => "Already Reported",
        226 => "IM Used",

        // 3xx Redirection
        300 => "Multiple Choices",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        305 => "Use Proxy",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",

        // 4xx Client Error
        400 => "Bad Request",
        401 => "Unauthorized",
        402 => "Payment Required",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        407 => "Proxy Authentication Required",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        411 => "Length Required",
        412 => "Precondition Failed",
        413 => "Payload Too Large",
        414 => "URI Too Long",
        415 => "Unsupported Media Type",
        416 => "Range Not Satisfiable",
        417 => "Expectation Failed",
        418 => "I'm a teapot",
        421 => "Misdirected Request",
        422 => "Unprocessable Entity",
        423 => "Locked",
        424 => "Failed Dependency",
        425 => "Too Early",
        426 => "Upgrade Required",
        428 => "Precondition Required",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        451 => "Unavailable For Legal Reasons",

        // 5xx Server Error
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        505 => "HTTP Version Not Supported",
        506 => "Variant Also Negotiates",
        507 => "Insufficient Storage",
        508 => "Loop Detected",
        510 => "Not Extended",
        511 => "Network Authentication Required",

        // Default for unknown codes
        _ => "Unknown",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1xx_status_codes() {
        assert_eq!(get_status_description(100), "100 Continue");
        assert_eq!(get_status_description(101), "101 Switching Protocols");
        assert_eq!(get_status_description(102), "102 Processing");
        assert_eq!(get_status_description(103), "103 Early Hints");
    }

    #[test]
    fn test_2xx_status_codes() {
        assert_eq!(get_status_description(200), "200 OK");
        assert_eq!(get_status_description(201), "201 Created");
        assert_eq!(get_status_description(204), "204 No Content");
    }

    #[test]
    fn test_3xx_status_codes() {
        assert_eq!(get_status_description(301), "301 Moved Permanently");
        assert_eq!(get_status_description(302), "302 Found");
        assert_eq!(get_status_description(304), "304 Not Modified");
    }

    #[test]
    fn test_4xx_status_codes() {
        assert_eq!(get_status_description(400), "400 Bad Request");
        assert_eq!(get_status_description(401), "401 Unauthorized");
        assert_eq!(get_status_description(404), "404 Not Found");
        assert_eq!(get_status_description(418), "418 I'm a teapot");
    }

    #[test]
    fn test_5xx_status_codes() {
        assert_eq!(get_status_description(500), "500 Internal Server Error");
        assert_eq!(get_status_description(502), "502 Bad Gateway");
        assert_eq!(get_status_description(503), "503 Service Unavailable");
        assert_eq!(get_status_description(504), "504 Gateway Timeout");
    }

    #[test]
    fn test_unknown_status_code() {
        assert_eq!(get_status_description(999), "999 Unknown");
        assert_eq!(get_status_description(123), "123 Unknown");
    }

    #[test]
    fn test_status_text_only() {
        assert_eq!(get_status_text(200), "OK");
        assert_eq!(get_status_text(404), "Not Found");
        assert_eq!(get_status_text(500), "Internal Server Error");
        assert_eq!(get_status_text(999), "Unknown");
    }
}
