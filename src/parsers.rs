pub fn parse_session_index(s: &str) -> Result<u32, String> {
    s.strip_prefix("replay@{")
        .and_then(|rest| rest.strip_suffix('}'))
        .ok_or_else(|| {
            format!(
                "Session name must be of the form replay@{{index}}, got '{}'",
                s
            )
        })?
        .parse::<u32>()
        .map_err(|_| format!("Invalid session index in '{}'", s))
}
