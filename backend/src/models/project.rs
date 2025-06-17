use rusqlite::{params, Result};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use rusqlite::Connection;
use std::fs;
use std::path::Path;
use log::{debug, error};

/// Модель проекта
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    /// Идентификатор проекта
    pub id: Option<i64>,
    /// Название проекта
    pub name: String,
    /// Идентификатор владельца проекта
    pub owner_id: i64,
    /// Описание проекта
    pub description: Option<String>,
    /// Флаг публичности проекта
    pub is_public: bool,
    /// Дата создания проекта
    pub created_at: Option<String>,
}

/// Конфигурация проекта (хранится в .project_config.json)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectConfig {
    /// Настройки коллаборации
    pub collaboration: CollaborationSettings,
    /// Роли участников
    pub roles: Vec<ProjectRole>,
    /// Настройки видимости
    pub visibility: VisibilitySettings,
    /// Настройки уведомлений
    pub notifications: NotificationSettings,
    /// Дополнительные метаданные
    pub metadata: ProjectMetadata,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CollaborationSettings {
    /// Разрешить форки
    pub allow_forks: bool,
    /// Разрешить issues
    pub allow_issues: bool,
    /// Разрешить pull requests
    pub allow_pull_requests: bool,
    /// Автоматическое слияние
    pub auto_merge: bool,
    /// Требовать ревью
    pub require_review: bool,
    /// Минимальное количество ревьюеров
    pub min_reviewers: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectRole {
    /// ID пользователя
    pub user_id: i64,
    /// Роль пользователя
    pub role: UserRole,
    /// Дата добавления
    pub added_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UserRole {
    Owner,
    Admin,
    Developer,
    Viewer,
    Guest,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VisibilitySettings {
    /// Публичный проект
    pub is_public: bool,
    /// Разрешить анонимное чтение
    pub allow_anonymous_read: bool,
    /// Скрыть от поиска
    pub hide_from_search: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationSettings {
    /// Уведомления о коммитах
    pub notify_on_commits: bool,
    /// Уведомления о issues
    pub notify_on_issues: bool,
    /// Уведомления о pull requests
    pub notify_on_pull_requests: bool,
    /// Email уведомления
    pub email_notifications: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectMetadata {
    /// Теги проекта
    pub tags: Vec<String>,
    /// Язык программирования
    pub primary_language: Option<String>,
    /// Лицензия
    pub license: Option<String>,
    /// Домашняя страница
    pub homepage: Option<String>,
    /// Размер проекта в байтах
    pub size: Option<u64>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            collaboration: CollaborationSettings {
                allow_forks: true,
                allow_issues: true,
                allow_pull_requests: true,
                auto_merge: false,
                require_review: false,
                min_reviewers: 1,
            },
            roles: Vec::new(),
            visibility: VisibilitySettings {
                is_public: true,
                allow_anonymous_read: true,
                hide_from_search: false,
            },
            notifications: NotificationSettings {
                notify_on_commits: true,
                notify_on_issues: true,
                notify_on_pull_requests: true,
                email_notifications: false,
            },
            metadata: ProjectMetadata {
                tags: Vec::new(),
                primary_language: None,
                license: None,
                homepage: None,
                size: None,
            },
        }
    }
}

impl Project {
    /// Создаёт новый проект в базе данных и на диске
    pub fn create(&self, conn: Arc<Mutex<Connection>>) -> Result<i64> {
        let conn_guard = conn.lock().unwrap();
        
        // Добавляем проект в базу данных
        conn_guard.execute(
            "INSERT INTO projects (name, owner_id, description, is_public) VALUES (?1, ?2, ?3, ?4)",
            params![self.name, self.owner_id, self.description, self.is_public],
        )?;
        
        let project_id = conn_guard.last_insert_rowid();
        drop(conn_guard);

        // Создаём каталог для проекта
        let project_path = format!("projects/{}", self.name);
        let path = Path::new(&project_path);
        
        if !path.exists() {
            if let Err(e) = fs::create_dir_all(path) {
                error!("Не удалось создать каталог для проекта: {}", e);
                return Err(rusqlite::Error::ExecuteReturnedResults);
            }
            
            // Создаём конфигурационный файл проекта
            let mut config = ProjectConfig::default();
            config.roles.push(ProjectRole {
                user_id: self.owner_id,
                role: UserRole::Owner,
                added_at: chrono::Utc::now().to_rfc3339(),
            });
            
            self.save_config(&config)?;
            debug!("Проект успешно создан: {}", self.name);
        }
        
        Ok(project_id)
    }

    /// Сохраняет конфигурацию проекта в файл
    pub fn save_config(&self, config: &ProjectConfig) -> Result<()> {
        let config_path = format!("projects/{}/.project_config.json", self.name);
        let config_json = serde_json::to_string_pretty(config)
            .map_err(|_| rusqlite::Error::ExecuteReturnedResults)?;
        
        fs::write(config_path, config_json)
            .map_err(|_| rusqlite::Error::ExecuteReturnedResults)?;
        
        Ok(())
    }

    /// Загружает конфигурацию проекта из файла
    pub fn load_config(&self) -> Result<ProjectConfig> {
        let config_path = format!("projects/{}/.project_config.json", self.name);
        
        if !Path::new(&config_path).exists() {
            return Ok(ProjectConfig::default());
        }
        
        let config_content = fs::read_to_string(config_path)
            .map_err(|_| rusqlite::Error::ExecuteReturnedResults)?;
        
        let config: ProjectConfig = serde_json::from_str(&config_content)
            .map_err(|_| rusqlite::Error::ExecuteReturnedResults)?;
        
        Ok(config)
    }

    /// Получает список проектов пользователя
    pub fn find_by_owner(owner_id: i64, conn: Arc<Mutex<Connection>>) -> Result<Vec<Project>> {
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, name, owner_id, description, is_public, created_at FROM projects WHERE owner_id = ?1"
        )?;
        
        let projects = stmt.query_map(params![owner_id], |row| {
            Ok(Project {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                owner_id: row.get(2)?,
                description: row.get(3)?,
                is_public: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        
        let mut result = Vec::new();
        for project in projects {
            result.push(project?);
        }
        
        Ok(result)
    }

    /// Находит проект по имени
    pub fn find_by_name(name: &str, conn: Arc<Mutex<Connection>>) -> Result<Option<Project>> {
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, name, owner_id, description, is_public, created_at FROM projects WHERE name = ?1"
        )?;
        
        let mut rows = stmt.query(params![name])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Project {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                owner_id: row.get(2)?,
                description: row.get(3)?,
                is_public: row.get(4)?,
                created_at: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Находит проект по имени и владельцу
    pub fn find_by_name_and_owner(name: &str, owner_id: i64, conn: Arc<Mutex<Connection>>) -> Result<Option<Project>> {
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, name, owner_id, description, is_public, created_at FROM projects WHERE name = ?1 AND owner_id = ?2"
        )?;
        
        let mut rows = stmt.query(params![name, owner_id])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Project {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                owner_id: row.get(2)?,
                description: row.get(3)?,
                is_public: row.get(4)?,
                created_at: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Получает все публичные проекты
    pub fn find_public(conn: Arc<Mutex<Connection>>) -> Result<Vec<Project>> {
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, name, owner_id, description, is_public, created_at FROM projects WHERE is_public = 1"
        )?;
        
        let projects = stmt.query_map([], |row| {
            Ok(Project {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                owner_id: row.get(2)?,
                description: row.get(3)?,
                is_public: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        
        let mut result = Vec::new();
        for project in projects {
            result.push(project?);
        }
        
        Ok(result)
    }
}
