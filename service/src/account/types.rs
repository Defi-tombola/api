use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct CreateAccount {
    pub address: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct UpdateAccount {
    pub avatar: Option<String>,
    pub name: Option<String>,
    pub twitter: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl Default for UpdateAccount {
    fn default() -> Self {
        UpdateAccount {
            avatar: None,
            name: None,
            twitter: None,
            updated_at: Utc::now(),
        }
    }
}

pub struct LinkTwitter {
    pub access_token: String,
}
