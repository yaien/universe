use chrono::{DateTime, Utc};

pub struct Organization {
    pub id: i64,
    pub hostname: String,
    pub url: String,
    pub title: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

pub struct Sitemap {
    pub id: i64,
    pub organization_id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Page {
    pub id: i64,
    pub sitemap_id: i64,
    pub layout_id: i64,
    pub path: String,
    pub html: String,
    pub css: String,
    pub js: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Layout {
    pub id: i64,
    pub sitemap_id: i64,
    pub name: String,
    pub html: String,
    pub css: String,
    pub js: String,
}

pub struct Email {
    pub id: i64,
    pub sitemap_id: i64,
    pub name: String,
    pub subject: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
