use rusqlite::{params, Result};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, NaiveDateTime};
use rusqlite::Connection;

/// Модель уведомления в системе
/// 
/// Используется для отправки уведомлений пользователям о различных событиях,
/// таких как создание пул-реквестов, комментарии, изменения в репозитории и т.д.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Notification {
    /// Идентификатор уведомления
    pub id: Option<i64>,
    /// Тип уведомления (например, "pull_request", "comment", "mention")
    pub notification_type: String,
    /// Заголовок уведомления
    pub title: String,
    /// Содержимое уведомления
    pub content: String,
    /// ID пользователя, которому адресовано уведомление
    pub user_id: i64,
    /// Флаг прочтения уведомления
    pub is_read: bool,
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
    pub fn create(&self, conn: Arc<Mutex<Connection>>) -> Result<i64> {
        let conn_guard = conn.lock().unwrap();

        conn_guard.execute(
            "INSERT INTO notifications 
            (notification_type, title, content, user_id, is_read) 
            VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                self.notification_type,
                self.title,
                self.content,
                self.user_id,
                self.is_read
            ]
        )?;

        Ok(conn_guard.last_insert_rowid())
    }

    /// Находит уведомления по ID пользователя
    /// 
    /// # Параметры
    /// 
    /// * `user_id` - ID пользователя
    /// * `conn` - Соединение с базой данных
    /// 
    /// # Возвращает
    /// 
    /// * `Result<Vec<Notification>>` - Список уведомлений пользователя
    pub fn find_by_user_id(user_id: i64, conn: Arc<Mutex<Connection>>) -> Result<Vec<Notification>> {
        let conn_guard = conn.lock().unwrap();
        
        let mut stmt = conn_guard.prepare(
            "SELECT id, notification_type, title, content, user_id, is_read, created_at 
             FROM notifications 
             WHERE user_id = ?1 
             ORDER BY created_at DESC"
        )?;
        
        let notifications = stmt.query_map(params![user_id], |row| {
            let created_at_str: String = row.get(6)?;
            
            Ok(Notification {
                id: Some(row.get(0)?),
                notification_type: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                user_id: row.get(4)?,
                is_read: row.get(5)?,
                created_at: match DateTime::parse_from_rfc3339(&created_at_str) {
                    Ok(dt) => Some(dt.with_timezone(&Utc)),
                    Err(_) => {
                        // Пробуем формат SQLite
                        let naive = NaiveDateTime::parse_from_str(&created_at_str, "%Y-%m-%d %H:%M:%S")
                            .or_else(|_| NaiveDateTime::parse_from_str(&created_at_str, "%Y-%m-%dT%H:%M:%S"));
                        
                        match naive {
                            Ok(ndt) => Some(DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc)),
                            Err(_) => None
                        }
                    }
                },
            })
        })?;
        
        let mut result = Vec::new();
        for notification in notifications {
            result.push(notification?);
        }
        
        Ok(result)
    }

    /// Отмечает уведомление как прочитанное
    /// 
    /// # Параметры
    /// 
    /// * `id` - ID уведомления
    /// * `conn` - Соединение с базой данных
    /// 
    /// # Возвращает
    /// 
    /// * `Result<()>` - Результат операции
    pub fn mark_as_read(id: i64, conn: Arc<Mutex<Connection>>) -> Result<()> {
        let conn_guard = conn.lock().unwrap();
        
        conn_guard.execute(
            "UPDATE notifications SET is_read = 1 WHERE id = ?1",
            params![id]
        )?;
        
        Ok(())
    }
}
