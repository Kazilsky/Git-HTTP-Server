use rusqlite::{params, Result};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
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
    /// Идентификатор проекта
    pub project_id: i64,
    /// Идентификатор владельца репозитория
    pub owner_id: i64,
    /// Описание репозитория
    pub description: Option<String>,
    /// Флаг публичности репозитория
    pub is_public: bool,
    /// Дата создания репозитория
    pub created_at: Option<String>,
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
            "INSERT INTO repositories (name, project_id, owner_id, description, is_public) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![self.name, self.project_id, self.owner_id, self.description, self.is_public],
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
            "SELECT id, name, project_id, owner_id, description, is_public, created_at FROM repositories WHERE owner_id = ?1"
        )?;
        
        let repos = stmt.query_map(params![owner_id], |row| {
            Ok(Repository {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                project_id: row.get(2)?,
                owner_id: row.get(3)?,
                description: row.get(4)?,
                is_public: row.get(5)?,
                created_at: row.get(6)?,
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
            "SELECT id, name, project_id, owner_id, description, is_public, created_at FROM repositories WHERE name = ?1"
        )?;
        
        let mut rows = stmt.query(params![name])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Repository {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                project_id: row.get(2)?,
                owner_id: row.get(3)?,
                description: row.get(4)?,
                is_public: row.get(5)?,
                created_at: row.get(6)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Получает список репозиториев проекта
    /// 
    /// # Параметры
    /// 
    /// * `project_id` - ID проекта
    /// * `conn` - Соединение с базой данных
    /// 
    /// # Возвращает
    /// 
    /// * `Result<Vec<Repository>>` - Список репозиториев
    pub fn find_by_project(project_id: i64, conn: Arc<Mutex<Connection>>) -> Result<Vec<Repository>> {
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, name, project_id, owner_id, description, is_public, created_at FROM repositories WHERE project_id = ?1"
        )?;
        
        let repos = stmt.query_map(params![project_id], |row| {
            Ok(Repository {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                project_id: row.get(2)?,
                owner_id: row.get(3)?,
                description: row.get(4)?,
                is_public: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;
        
        let mut result = Vec::new();
        for repo in repos {
            result.push(repo?);
        }
        
        Ok(result)
    }

    /// Находит репозиторий по имени и проекту
    /// 
    /// # Параметры
    /// 
    /// * `name` - Имя репозитория
    /// * `project_id` - ID проекта
    /// * `conn` - Соединение с базой данных
    /// 
    /// # Возвращает
    /// 
    /// * `Result<Option<Repository>>` - Найденный репозиторий или None
    pub fn find_by_name_and_project(name: &str, project_id: i64, conn: Arc<Mutex<Connection>>) -> Result<Option<Repository>> {
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, name, project_id, owner_id, description, is_public, created_at FROM repositories WHERE name = ?1 AND project_id = ?2"
        )?;
        
        let mut rows = stmt.query(params![name, project_id])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Repository {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                project_id: row.get(2)?,
                owner_id: row.get(3)?,
                description: row.get(4)?,
                is_public: row.get(5)?,
                created_at: row.get(6)?,
            }))
        } else {
            Ok(None)
        }
    }
}
