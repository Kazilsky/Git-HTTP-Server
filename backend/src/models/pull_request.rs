use rusqlite::{params, Result};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, NaiveDateTime};
use rusqlite::Connection;
use log::{debug, error};
use crate::models::notification::Notification;

/// Статус пул-реквеста
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PullRequestStatus {
    /// Открыт, ожидает рассмотрения
    Open,
    /// Закрыт без принятия изменений
    Closed,
    /// Принят и слит с целевой веткой
    Merged,
}

impl PullRequestStatus {
    /// Преобразует строковое представление статуса в enum
    pub fn from_str(status: &str) -> Self {
        match status.to_lowercase().as_str() {
            "open" => PullRequestStatus::Open,
            "closed" => PullRequestStatus::Closed,
            "merged" => PullRequestStatus::Merged,
            _ => PullRequestStatus::Open, // По умолчанию считаем открытым
        }
    }

    /// Преобразует enum в строковое представление
    pub fn to_str(&self) -> &'static str {
        match self {
            PullRequestStatus::Open => "open",
            PullRequestStatus::Closed => "closed",
            PullRequestStatus::Merged => "merged",
        }
    }
}

/// Модель пул-реквеста
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PullRequest {
    /// Идентификатор пул-реквеста
    pub id: Option<i64>,
    /// Заголовок пул-реквеста
    pub title: String,
    /// Описание пул-реквеста
    pub description: Option<String>,
    /// Идентификатор репозитория
    pub repository_id: i64,
    /// Исходная ветка (откуда берутся изменения)
    pub source_branch: String,
    /// Целевая ветка (куда вносятся изменения)
    pub target_branch: String,
    /// Идентификатор автора пул-реквеста
    pub author_id: i64,
    /// Статус пул-реквеста
    pub status: PullRequestStatus,
    /// Дата создания пул-реквеста
    pub created_at: Option<DateTime<Utc>>,
    /// Дата последнего обновления пул-реквеста
    pub updated_at: Option<DateTime<Utc>>,
}

/// Модель комментария к пул-реквесту
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PullRequestComment {
    /// Идентификатор комментария
    pub id: Option<i64>,
    /// Идентификатор пул-реквеста
    pub pull_request_id: i64,
    /// Идентификатор автора комментария
    pub author_id: i64,
    /// Содержимое комментария
    pub content: String,
    /// Дата создания комментария
    pub created_at: Option<DateTime<Utc>>,
}

impl PullRequest {
    /// Создаёт новый пул-реквест в базе данных
    /// 
    /// # Параметры
    /// 
    /// * `conn` - Соединение с базой данных
    /// 
    /// # Возвращает
    /// 
    /// * `Result<i64>` - ID созданного пул-реквеста
    pub fn create(&self, conn: Arc<Mutex<Connection>>) -> Result<i64> {
        let conn_guard = conn.lock().unwrap();
        
        conn_guard.execute(
            "INSERT INTO pull_requests 
            (title, description, repository_id, source_branch, target_branch, author_id, status) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                self.title,
                self.description,
                self.repository_id,
                self.source_branch,
                self.target_branch,
                self.author_id,
                self.status.to_str()
            ],
        )?;
        
        let pr_id = conn_guard.last_insert_rowid();
        
        // Получаем ID владельца репозитория для отправки уведомления
        let mut stmt = conn_guard.prepare(
            "SELECT owner_id FROM repositories WHERE id = ?1"
        )?;
        
        let owner_id: i64 = stmt.query_row(params![self.repository_id], |row| row.get(0))?;
        
        // Если автор PR не является владельцем репозитория, отправляем уведомление
        if owner_id != self.author_id {
            // Создаем уведомление для владельца репозитория
            let notification = Notification {
                id: None,
                notification_type: "pull_request".to_string(),
                title: format!("New pull request: {}", self.title),
                content: format!("A new pull request has been created in your repository: {}", self.title),
                user_id: owner_id,
                is_read: false,
                created_at: None,
            };
            
            // Сохраняем уведомление в базе данных
            // Create a new connection for the notification
            let new_conn = Arc::clone(&conn);
            match notification.create(new_conn) {
                Ok(_) => debug!("Notification created for pull request"),
                Err(e) => error!("Failed to create notification: {}", e),
            }
        }
        
        Ok(pr_id)
    }

    /// Получает список пул-реквестов для репозитория
    /// 
    /// # Параметры
    /// 
    /// * `repository_id` - ID репозитория
    /// * `conn` - Соединение с базой данных
    /// 
    /// # Возвращает
    /// 
    /// * `Result<Vec<PullRequest>>` - Список пул-реквестов
    pub fn find_by_repository(repository_id: i64, conn: Arc<Mutex<Connection>>) -> Result<Vec<PullRequest>> {
        let conn_guard = conn.lock().unwrap();
        
        let mut stmt = conn_guard.prepare(
            "SELECT id, title, description, repository_id, source_branch, target_branch, 
                    author_id, status, created_at, updated_at 
             FROM pull_requests 
             WHERE repository_id = ?1 
             ORDER BY created_at DESC"
        )?;
        
        let pull_requests = stmt.query_map(params![repository_id], |row| {
            let created_at_str: String = row.get(8)?;
            let updated_at_str: String = row.get(9)?;
            let status_str: String = row.get(7)?;
            
            Ok(PullRequest {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                description: row.get(2)?,
                repository_id: row.get(3)?,
                source_branch: row.get(4)?,
                target_branch: row.get(5)?,
                author_id: row.get(6)?,
                status: PullRequestStatus::from_str(&status_str),
                created_at: parse_datetime(&created_at_str),
                updated_at: parse_datetime(&updated_at_str),
            })
        })?;
        
        let mut result = Vec::new();
        for pr in pull_requests {
            result.push(pr?);
        }
        
        Ok(result)
    }

    /// Получает пул-реквест по ID
    /// 
    /// # Параметры
    /// 
    /// * `id` - ID пул-реквеста
    /// * `conn` - Соединение с базой данных
    /// 
    /// # Возвращает
    /// 
    /// * `Result<Option<PullRequest>>` - Найденный пул-реквест или None
    pub fn find_by_id(id: i64, conn: Arc<Mutex<Connection>>) -> Result<Option<PullRequest>> {
        let conn_guard = conn.lock().unwrap();
        
        let mut stmt = conn_guard.prepare(
            "SELECT id, title, description, repository_id, source_branch, target_branch, 
                    author_id, status, created_at, updated_at 
             FROM pull_requests 
             WHERE id = ?1"
        )?;
        
        let mut rows = stmt.query(params![id])?;
        
        if let Some(row) = rows.next()? {
            let created_at_str: String = row.get(8)?;
            let updated_at_str: String = row.get(9)?;
            let status_str: String = row.get(7)?;
            
            Ok(Some(PullRequest {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                description: row.get(2)?,
                repository_id: row.get(3)?,
                source_branch: row.get(4)?,
                target_branch: row.get(5)?,
                author_id: row.get(6)?,
                status: PullRequestStatus::from_str(&status_str),
                created_at: parse_datetime(&created_at_str),
                updated_at: parse_datetime(&updated_at_str),
            }))
        } else {
            Ok(None)
        }
    }

    /// Обновляет статус пул-реквеста
    /// 
    /// # Параметры
    /// 
    /// * `id` - ID пул-реквеста
    /// * `status` - Новый статус
    /// * `conn` - Соединение с базой данных
    /// 
    /// # Возвращает
    /// 
    /// * `Result<()>` - Результат операции
    pub fn update_status(id: i64, status: PullRequestStatus, conn: Arc<Mutex<Connection>>) -> Result<()> {
        let conn_guard = conn.lock().unwrap();
        
        conn_guard.execute(
            "UPDATE pull_requests SET status = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![status.to_str(), id],
        )?;
        
        Ok(())
    }

    /// Сливает пул-реквест (выполняет git merge)
    /// 
    /// # Параметры
    /// 
    /// * `id` - ID пул-реквеста
    /// * `conn` - Соединение с базой данных
    /// 
    /// # Возвращает
    /// 
    /// * `Result<()>` - Результат операции
    pub fn merge(id: i64, conn: Arc<Mutex<Connection>>) -> Result<()> {
        // Получаем информацию о пул-реквесте
        let pr = match Self::find_by_id(id, conn.clone())? {
            Some(pr) => pr,
            None => return Err(rusqlite::Error::QueryReturnedNoRows),
        };
        
        // Получаем имя репозитория
        // Get repository name
        let repo_name = {
            let conn_guard = conn.lock().unwrap();
            let mut stmt = conn_guard.prepare(
                "SELECT name FROM repositories WHERE id = ?1"
            )?;
            
            stmt.query_row(params![pr.repository_id], |row| row.get::<_, String>(0))?
        };
        
        // Путь к репозиторию
        let repo_path = format!("repositories/{}.git", repo_name);
        
        // Выполняем слияние веток с помощью git
        // Это упрощенная реализация, в реальном проекте нужно больше проверок и обработки ошибок
        use std::process::Command;
        
        // Клонируем репозиторий во временную директорию
        let temp_dir = format!("temp_merge_{}", id);
        let clone_status = Command::new("git")
            .args(&["clone", &repo_path, &temp_dir])
            .status();
        
        if let Err(e) = clone_status {
            error!("Failed to clone repository: {}", e);
            return Err(rusqlite::Error::ExecuteReturnedResults);
        }
        
        // Переключаемся на целевую ветку
        let checkout_status = Command::new("git")
            .args(&["-C", &temp_dir, "checkout", &pr.target_branch])
            .status();
        
        if let Err(e) = checkout_status {
            error!("Failed to checkout target branch: {}", e);
            // Удаляем временную директорию
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Err(rusqlite::Error::ExecuteReturnedResults);
        }
        
        // Выполняем слияние
        let merge_status = Command::new("git")
            .args(&["-C", &temp_dir, "merge", &pr.source_branch])
            .status();
        
        if let Err(e) = merge_status {
            error!("Failed to merge branches: {}", e);
            // Удаляем временную директорию
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Err(rusqlite::Error::ExecuteReturnedResults);
        }
        
        // Отправляем изменения обратно в репозиторий
        let push_status = Command::new("git")
            .args(&["-C", &temp_dir, "push", "origin", &pr.target_branch])
            .status();
        
        if let Err(e) = push_status {
            error!("Failed to push changes: {}", e);
            // Удаляем временную директорию
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Err(rusqlite::Error::ExecuteReturnedResults);
        }
        
        // Удаляем временную директорию
        if let Err(e) = std::fs::remove_dir_all(&temp_dir) {
            error!("Failed to remove temporary directory: {}", e);
        }
        
        // Обновляем статус пул-реквеста
        let conn_clone = Arc::clone(&conn);
        Self::update_status(id, PullRequestStatus::Merged, conn_clone)?;
        
        Ok(())
    }
}

impl PullRequestComment {
    /// Создаёт новый комментарий к пул-реквесту
    /// 
    /// # Параметры
    /// 
    /// * `conn` - Соединение с базой данных
    /// 
    /// # Возвращает
    /// 
    /// * `Result<i64>` - ID созданного комментария
    pub fn create(&self, conn: Arc<Mutex<Connection>>) -> Result<i64> {
        let conn_guard = conn.lock().unwrap();
        
        conn_guard.execute(
            "INSERT INTO pull_request_comments 
            (pull_request_id, author_id, content) 
            VALUES (?1, ?2, ?3)",
            params![self.pull_request_id, self.author_id, self.content],
        )?;
        
        let comment_id = conn_guard.last_insert_rowid();
        
        // Получаем информацию о пул-реквесте для отправки уведомления
        let mut stmt = conn_guard.prepare(
            "SELECT author_id FROM pull_requests WHERE id = ?1"
        )?;
        
        let pr_author_id: i64 = stmt.query_row(params![self.pull_request_id], |row| row.get(0))?;
        
        // Если автор комментария не является автором PR, отправляем уведомление
        if pr_author_id != self.author_id {
            // Создаем уведомление для автора PR
            let notification = Notification {
                id: None,
                notification_type: "comment".to_string(),
                title: "New comment on your pull request".to_string(),
                content: format!("Someone commented on your pull request: {}", self.content),
                user_id: pr_author_id,
                is_read: false,
                created_at: None,
            };
            
            // Сохраняем уведомление в базе данных
            let new_conn = Arc::clone(&conn);
            match notification.create(new_conn) {
                Ok(_) => debug!("Notification created for comment"),
                Err(e) => error!("Failed to create notification: {}", e),
            }
        }
        
        Ok(comment_id)
    }

    /// Получает комментарии для пул-реквеста
    /// 
    /// # Параметры
    /// 
    /// * `pull_request_id` - ID пул-реквеста
    /// * `conn` - Соединение с базой данных
    /// 
    /// # Возвращает
    /// 
    /// * `Result<Vec<PullRequestComment>>` - Список комментариев
    pub fn find_by_pull_request(pull_request_id: i64, conn: Arc<Mutex<Connection>>) -> Result<Vec<PullRequestComment>> {
        let conn_guard = conn.lock().unwrap();
        
        let mut stmt = conn_guard.prepare(
            "SELECT id, pull_request_id, author_id, content, created_at 
             FROM pull_request_comments 
             WHERE pull_request_id = ?1 
             ORDER BY created_at ASC"
        )?;
        
        let comments = stmt.query_map(params![pull_request_id], |row| {
            let created_at_str: String = row.get(4)?;
            
            Ok(PullRequestComment {
                id: Some(row.get(0)?),
                pull_request_id: row.get(1)?,
                author_id: row.get(2)?,
                content: row.get(3)?,
                created_at: parse_datetime(&created_at_str),
            })
        })?;
        
        let mut result = Vec::new();
        for comment in comments {
            result.push(comment?);
        }
        
        Ok(result)
    }
}

/// Вспомогательная функция для парсинга даты/времени из строки
fn parse_datetime(datetime_str: &str) -> Option<DateTime<Utc>> {
    // Пробуем разные форматы даты
    if let Ok(dt) = DateTime::parse_from_rfc3339(datetime_str) {
        return Some(dt.with_timezone(&Utc));
    }
    
    // Если формат не RFC3339, возможно это формат SQLite (YYYY-MM-DD HH:MM:SS)
    let naive = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S"));
    
    if let Ok(ndt) = naive {
        return Some(DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc));
    }
    
    None
}
