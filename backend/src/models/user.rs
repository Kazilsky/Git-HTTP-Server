use rusqlite::{params, Result};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, NaiveDateTime};
use rusqlite::Connection;

/// Модель пользователя системы
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    /// Идентификатор пользователя
    pub id: Option<i64>,
    /// Имя пользователя для входа
    pub username: String,
    /// Пароль пользователя (в хэшированном виде)
    #[serde(skip_serializing)]
    pub password: String,
    /// Электронная почта пользователя
    pub email: Option<String>,
    /// Дата создания пользователя
    pub created_at: Option<DateTime<Utc>>,
}

impl User {
    /// Создаёт нового пользователя в базе данных
    /// 
    /// # Параметры
    /// 
    /// * `conn` - Соединение с базой данных
    /// 
    /// # Возвращает
    /// 
    /// * `Result<i64>` - ID созданного пользователя
    pub fn create(&self, conn: Arc<Mutex<Connection>>) -> Result<i64> {
        let conn = conn.lock().unwrap();
        
        conn.execute(
            "INSERT INTO users (username, password, email) VALUES (?1, ?2, ?3)",
            params![self.username, self.password, self.email],
        )?;
        
        Ok(conn.last_insert_rowid())
    }

    /// Находит пользователя по имени пользователя
    /// 
    /// # Параметры
    /// 
    /// * `username` - Имя пользователя для поиска
    /// * `conn` - Соединение с базой данных
    /// 
    /// # Возвращает
    /// 
    /// * `Result<Option<User>>` - Найденный пользователь или None
    pub fn find_by_username(username: &str, conn: Arc<Mutex<Connection>>) -> Result<Option<User>> {
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare("SELECT id, username, password, email, created_at FROM users WHERE username = ?1")?;
        let mut rows = stmt.query(params![username])?;
        
        if let Some(row) = rows.next()? {
            // Безопасное получение даты создания (с обработкой возможных ошибок формата)
            let created_at_str: Option<String> = row.get(4).ok();
            let created_at = if let Some(datetime_str) = created_at_str {
                // Пробуем разные форматы даты
                if let Ok(dt) = DateTime::parse_from_rfc3339(&datetime_str) {
                    Some(dt.with_timezone(&Utc))
                } else {
                    // Если формат не RFC3339, возможно это формат SQLite (YYYY-MM-DD HH:MM:SS)
                    let naive = NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M:%S")
                        .or_else(|_| NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%dT%H:%M:%S"));
                    
                    if let Ok(ndt) = naive {
                        Some(DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
                    } else {
                        // Если не можем разобрать дату, вернем None
                        None
                    }
                }
            } else {
                None
            };
            
            Ok(Some(User {
                id: Some(row.get(0)?),
                username: row.get(1)?,
                password: row.get(2)?,
                email: row.get(3)?,
                created_at,
            }))
        } else {
            Ok(None)
        }
    }

    /// Проверяет учетные данные пользователя
    /// 
    /// # Параметры
    /// 
    /// * `username` - Имя пользователя
    /// * `password` - Пароль пользователя
    /// * `conn` - Соединение с базой данных
    /// 
    /// # Возвращает
    /// 
    /// * `Result<Option<User>>` - Пользователь, если учетные данные верны
    pub fn authenticate(username: &str, password: &str, conn: Arc<Mutex<Connection>>) -> Result<Option<User>> {
        let user = Self::find_by_username(username, conn)?;
        
        if let Some(user) = user {
            if user.password == password {
                return Ok(Some(user));
            }
        }
        
        Ok(None)
    }
} 