#[derive(Debug)]
pub struct Unsubscribe {
    pub packet_id: u16,
    pub filters: Vec<String>,
    pub properties: Option<UnsubscribeProperties>,
}

#[derive(Debug)]
pub struct UnsubscribeProperties {
    pub user_properties: Vec<(String, String)>,
}
