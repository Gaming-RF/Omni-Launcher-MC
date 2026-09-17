use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    pub uuid: String,
    pub username: String,
    pub access_token: String,
    pub refresh_token: String,
    pub skin_url: Option<String>,
}

/// Get all accounts.
pub fn get_all_accounts(db: &rusqlite::Connection) -> Result<Vec<Account>> {
    let mut stmt = db.prepare(
        "SELECT uuid, username, access_token, refresh_token, skin_url 
         FROM accounts ORDER BY last_used DESC NULLS LAST, username",
    )?;

    let accounts = stmt
        .query_map([], |row| {
            Ok(Account {
                uuid: row.get(0)?,
                username: row.get(1)?,
                access_token: row.get(2)?,
                refresh_token: row.get(3)?,
                skin_url: row.get(4)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(accounts)
}

/// Get the active (first/only) account.
pub fn get_active_account(db: &rusqlite::Connection) -> Result<Option<Account>> {
    let mut stmt = db.prepare(
        "SELECT uuid, username, access_token, refresh_token, skin_url 
         FROM accounts LIMIT 1",
    )?;

    let mut rows = stmt.query_map([], |row| {
        Ok(Account {
            uuid: row.get(0)?,
            username: row.get(1)?,
            access_token: row.get(2)?,
            refresh_token: row.get(3)?,
            skin_url: row.get(4)?,
        })
    })?;

    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

/// Insert or replace an account.
pub fn upsert_account(db: &rusqlite::Connection, account: &Account) -> Result<()> {
    db.execute(
        "INSERT INTO accounts (uuid, username, access_token, refresh_token, skin_url, last_used)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(uuid) DO UPDATE SET
           username = excluded.username,
           access_token = excluded.access_token,
           refresh_token = excluded.refresh_token,
           skin_url = excluded.skin_url,
           last_used = excluded.last_used",
        rusqlite::params![
            account.uuid,
            account.username,
            account.access_token,
            account.refresh_token,
            account.skin_url,
            chrono::Utc::now().to_rfc3339(),
        ],
    )?;
    Ok(())
}

/// Delete an account.
pub fn delete_account(db: &rusqlite::Connection, uuid: &str) -> Result<()> {
    db.execute("DELETE FROM accounts WHERE uuid = ?1", [uuid])?;
    Ok(())
}

/// Update tokens for an existing account.
pub fn update_tokens(
    db: &rusqlite::Connection,
    uuid: &str,
    access_token: &str,
    refresh_token: &str,
) -> Result<()> {
    db.execute(
        "UPDATE accounts SET access_token = ?2, refresh_token = ?3 WHERE uuid = ?1",
        rusqlite::params![uuid, access_token, refresh_token],
    )?;
    Ok(())
}
