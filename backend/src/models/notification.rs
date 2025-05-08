use rusqlite::{params, Result};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, NaiveDateTime};
use rusqlite::Connection;

pub struct Notification {
    /// Уведомление
    ///
    /// Идентификатор
    pub id: Option<i64>,
    /// Наименование 
    pub name: String,
    /// Содержимое
    pub content: String,
    /// Дата создания уведомления
    pub created_at: Option<DateTime<Utc>>,
}

impl Notification {
    /// Создаёт новое уведомление в базе данных
    /// 
    /// # Параметры
    /// 
    /// * `conn` - Соединение с базой данных
    /// 
    /// # Возвращает
    /// 
    /// * `Result<i64>` - ID созданного уведомления
    pub fn create (&self, conn: Arc<Mutex<Connection>>) -> Result<i64> {
        let conn = conn.lock();

        conn.execute("
            INSERT INTO notification 
            (name, content) 
            VALUES (?1, ?2)
        ", params![self.name, self.content])?;

        Ok(conn.last_insert_rowid())
    }

    pub fn find_notification_by_name (&self, name: &str) -> Result<Option<Notification>> {
        let conn = conn.lock().map_err(|_| rusqlite::Error::InvalidQuery)?;

        let mut stmt = conn.prepare("SELECT id, name, content, created_at FROM notification WHERE name = ?1");

        let row = stmt.query(params![name]);
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
            
            Ok(Some(Notification {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                content: row.get(2)?,
                created_at,
            }))
        } else {
            Ok(None)
        }
    }
}
