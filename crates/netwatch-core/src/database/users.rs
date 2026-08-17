use crate::models::User;
use rusqlite::{Connection, Result as SqliteResult, params};

pub struct UserRepository;

impl UserRepository {
    pub fn create(conn: &Connection, name: &str) -> SqliteResult<User> {
        conn.execute("INSERT INTO users (name) VALUES (?1)", params![name])?;
        let id = conn.last_insert_rowid();
        Self::get_by_id(conn, id)
    }

    pub fn get_by_id(conn: &Connection, id: i64) -> SqliteResult<User> {
        conn.query_row(
            "SELECT id, name, created_at FROM users WHERE id = ?1",
            params![id],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                })
            },
        )
    }

    pub fn get_all(conn: &Connection) -> SqliteResult<Vec<User>> {
        let mut stmt = conn.prepare("SELECT id, name, created_at FROM users ORDER BY name")?;
        let iter = stmt.query_map([], |row| {
            Ok(User {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?;

        let mut users = Vec::new();
        for user in iter {
            users.push(user?);
        }
        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::setup_test_db;

    #[test]
    fn creates_and_retrieves_user() {
        let conn = setup_test_db();
        let user = UserRepository::create(&conn, "Mahmoud").unwrap();
        assert_eq!(user.name, "Mahmoud");
        assert!(user.id > 0);

        let fetched = UserRepository::get_by_id(&conn, user.id).unwrap();
        assert_eq!(fetched, user);
    }

    #[test]
    fn retrieves_all_users() {
        let conn = setup_test_db();
        UserRepository::create(&conn, "Ahmed").unwrap();
        UserRepository::create(&conn, "Mahmoud").unwrap();

        let users = UserRepository::get_all(&conn).unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].name, "Ahmed");
        assert_eq!(users[1].name, "Mahmoud");
    }

    #[test]
    fn duplicate_user_name_fails() {
        let conn = setup_test_db();
        UserRepository::create(&conn, "Mahmoud").unwrap();
        let result = UserRepository::create(&conn, "Mahmoud");
        assert!(result.is_err());
    }
}
