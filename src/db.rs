use crate::prelude::*;

use std::io::{Read as _, Write as _};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

#[derive(
    serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq,
)]
pub struct Entry {
    pub id: String,
    pub org_id: Option<String>,
    pub folder: Option<String>,
    pub name: String,
    pub data: EntryData,
    pub notes: Option<String>,
    pub history: Vec<HistoryEntry>,
}

#[derive(
    serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq,
)]
pub enum EntryData {
    Login {
        username: Option<String>,
        password: Option<String>,
    },
    Card {
        cardholder_name: Option<String>,
        number: Option<String>,
        brand: Option<String>,
        exp_month: Option<String>,
        exp_year: Option<String>,
        code: Option<String>,
    },
    Identity {
        title: Option<String>,
        first_name: Option<String>,
        middle_name: Option<String>,
        last_name: Option<String>,
        address1: Option<String>,
        address2: Option<String>,
        address3: Option<String>,
        city: Option<String>,
        state: Option<String>,
        postal_code: Option<String>,
        country: Option<String>,
        phone: Option<String>,
        email: Option<String>,
        ssn: Option<String>,
        license_number: Option<String>,
        passport_number: Option<String>,
        username: Option<String>,
    },
    SecureNote,
}

#[derive(
    serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq,
)]
pub struct HistoryEntry {
    pub last_used_date: String,
    pub password: String,
}

#[derive(serde::Serialize, serde::Deserialize, Default, Debug)]
pub struct Db {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,

    pub iterations: Option<u32>,
    pub protected_key: Option<String>,
    pub protected_private_key: Option<String>,
    pub protected_org_keys: std::collections::HashMap<String, String>,

    pub entries: Vec<Entry>,
}

impl Db {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(server: &str, email: &str) -> Result<Self> {
        let mut fh = std::fs::File::open(crate::dirs::db_file(server, email))
            .context(crate::error::LoadDb)?;
        let mut json = String::new();
        fh.read_to_string(&mut json).context(crate::error::LoadDb)?;
        let slf: Self =
            serde_json::from_str(&json).context(crate::error::LoadDbJson)?;
        Ok(slf)
    }

    pub async fn load_async(server: &str, email: &str) -> Result<Self> {
        let mut fh =
            tokio::fs::File::open(crate::dirs::db_file(server, email))
                .await
                .context(crate::error::LoadDbAsync)?;
        let mut json = String::new();
        fh.read_to_string(&mut json)
            .await
            .context(crate::error::LoadDbAsync)?;
        let slf: Self =
            serde_json::from_str(&json).context(crate::error::LoadDbJson)?;
        Ok(slf)
    }

    // XXX need to make this atomic
    pub fn save(&self, server: &str, email: &str) -> Result<()> {
        let filename = crate::dirs::db_file(server, email);
        // unwrap is safe here because Self::filename is explicitly
        // constructed as a filename in a directory
        std::fs::create_dir_all(filename.parent().unwrap())
            .context(crate::error::SaveDb)?;
        let mut fh =
            std::fs::File::create(filename).context(crate::error::SaveDb)?;
        fh.write_all(
            serde_json::to_string(self)
                .context(crate::error::SaveDbJson)?
                .as_bytes(),
        )
        .context(crate::error::SaveDb)?;
        Ok(())
    }

    // XXX need to make this atomic
    pub async fn save_async(&self, server: &str, email: &str) -> Result<()> {
        let filename = crate::dirs::db_file(server, email);
        // unwrap is safe here because Self::filename is explicitly
        // constructed as a filename in a directory
        tokio::fs::create_dir_all(filename.parent().unwrap())
            .await
            .context(crate::error::SaveDbAsync)?;
        let mut fh = tokio::fs::File::create(filename)
            .await
            .context(crate::error::SaveDbAsync)?;
        fh.write_all(
            serde_json::to_string(self)
                .context(crate::error::SaveDbJson)?
                .as_bytes(),
        )
        .await
        .context(crate::error::SaveDbAsync)?;
        Ok(())
    }

    pub fn remove(server: &str, email: &str) -> Result<()> {
        let filename = crate::dirs::db_file(server, email);
        let res = std::fs::remove_file(filename);
        if let Err(e) = &res {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Ok(());
            }
        }
        res.context(crate::error::RemoveDb)?;
        Ok(())
    }

    pub fn needs_login(&self) -> bool {
        self.access_token.is_none()
            || self.refresh_token.is_none()
            || self.iterations.is_none()
            || self.protected_key.is_none()
    }
}
