/// taopic 是否含有通配符
pub fn topic_has_wildcards(s: &str) -> bool {
    s.contains('+') || s.contains('#')
}
