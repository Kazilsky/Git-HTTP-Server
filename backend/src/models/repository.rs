use rusqlite::{params, Result};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, NaiveDateTime};
use rusqlite::Connection;
use std::process::Command;
use std::path::Path;
use log::{debug, error};

/// Модель репозитория Git
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Repository {
    /// Идентификатор репозитория
    pub id: Option<i64>,
    /// Название репозитория
    pub name: String,
    /// Идентификатор владельца репозитория
    pub owner_id: i64,
    /// Описание репозитория
    pub description: Option<String>,
    /// Флаг публичности репозитория
    pub is_public: bool,
    /// Дата создания репозитория
    pub created_at: Option<DateTime<Utc>>,
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

impl Repository {
    /// Создаёт новый репозиторий в базе данных и на диске
    /// 
    /// # Параметры
    /// 
    /// * `conn` - Соединение с базой данных
    /// 
    /// # Возвращает
    /// 
    /// * `Result<i64>` - ID созданного репозитория
    pub fn create(&self, conn: Arc<Mutex<Connection>>) -> Result<i64> {
        let conn_guard = conn.lock().unwrap();
        
        // Добавляем репозиторий в базу данных
        conn_guard.execute(
            "INSERT INTO repositories (name, owner_id, description, is_public) VALUES (?1, ?2, ?3, ?4)",
            params![self.name, self.owner_id, self.description, self.is_public],
        )?;
        
        let repo_id = conn_guard.last_insert_rowid();
        drop(conn_guard); // Освобождаем блокировку

        // Создаём репозиторий на диске
        let repo_path = format!("repositories/{}.git", self.name);
        let path = Path::new(&repo_path);
        
        if !path.exists() {
            // Создаём каталог для репозитория
            if let Err(e) = std::fs::create_dir_all(path) {
                error!("Не удалось создать каталог для репозитория: {}", e);
                return Err(rusqlite::Error::ExecuteReturnedResults);
            }
            
            // Инициализируем bare репозиторий Git
            let output = Command::new("git")
                .arg("init")
                .arg("--bare")
                .arg(path)
                .output();
                
            match output {
                Ok(output) if output.status.success() => {
                    debug!("Репозиторий успешно инициализирован: {}", self.name);
                }
                Ok(output) => {
                    error!("Ошибка при инициализации репозитория: {}", String::from_utf8_lossy(&output.stderr));
                    return Err(rusqlite::Error::ExecuteReturnedResults);
                }
                Err(e) => {
                    error!("Не удалось выполнить команду git init: {}", e);
                    return Err(rusqlite::Error::ExecuteReturnedResults);
                }
            }
        }
        
        Ok(repo_id)
    }

    /// Получает список репозиториев пользователя
    /// 
    /// # Параметры
    /// 
    /// * `owner_id` - ID пользователя
    /// * `conn` - Соединение с базой данных
    /// 
    /// # Возвращает
    /// 
    /// * `Result<Vec<Repository>>` - Список репозиториев
    pub fn find_by_owner(owner_id: i64, conn: Arc<Mutex<Connection>>) -> Result<Vec<Repository>> {
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, name, owner_id, description, is_public, created_at FROM repositories WHERE owner_id = ?1"
        )?;
        
        let repos = stmt.query_map(params![owner_id], |row| {
            let created_at: String = row.get(5)?;
            
            Ok(Repository {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                owner_id: row.get(2)?,
                description: row.get(3)?,
                is_public: row.get(4)?,
                
                created_at: parse_datetime(&created_at),
            })
        })?;
        
        let mut result = Vec::new();
        for repo in repos {
            result.push(repo?);
        }
        
        Ok(result)
    }

    /// Находит репозиторий по имени
    /// 
    /// # Параметры
    /// 
    /// * `name` - Имя репозитория
    /// * `conn` - Соединение с базой данных
    /// 
    /// # Возвращает
    /// 
    /// * `Result<Option<Repository>>` - Найденный репозиторий или None
    pub fn find_by_name(name: &str, conn: Arc<Mutex<Connection>>) -> Result<Option<Repository>> {
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, name, owner_id, description, is_public, created_at FROM repositories WHERE name = ?1"
        )?;
        
        let mut rows = stmt.query(params![name])?;
        
        if let Some(row) = rows.next()? {
            let created_at: String = row.get(5)?;
            
            Ok(Some(Repository {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                owner_id: row.get(2)?,
                description: row.get(3)?,
                is_public: row.get(4)?,
                created_at: parse_datetime(&created_at),
            }))
        } else {
            Ok(None)
        }
    }
}
