/// taopic 是否含有通配符
pub fn topic_has_wildcards(s: &str) -> bool {
    s.contains('+') || s.contains('#')
}

/// 匹配发布消息使用的 topic 和 订阅的 filter
pub fn matches(topic: &str, filter: &str) -> bool {
    // 以 $ 开头的topic不可以由用户publish
    if !topic.is_empty() && topic.starts_with('$') {
        return false
    }
    let mut topics = topic.split('/');
    let filters = filter.split('/');

    for f in filters {
        // # 字符匹配所有子级
        if f == "#" {
            return true;
        }

        let top = topics.next();
        match top {
            // + 字符直接匹配这一层
            Some(_) if f == "+" => continue,
            // 没有通配符，必须完全匹配
            Some(t) if f != t => return false,
            Some(_) => continue,
            // topic 层级不够了
            None => return false,
        }
    }

    // filter 层级不够了
    topics.next().is_none()
}