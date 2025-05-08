use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

/// База данных для хранения информации о пользователях, репозиториях и других данных
#[derive(Clone)]
pub struct Database {
    /// Соединение с базой данных SQLite
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Создаёт новый экземпляр базы данных и инициализирует необходимые таблицы
    /// 
    /// # Возвращает
    /// 
    /// * `Result<Database>` - Результат создания базы данных
    pub fn new() -> Result<Self> {
        let conn = Connection::open(
            "gitea.db"
        )?;
        
        // Создаём таблицы, если они ещё не существуют
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                password TEXT NOT NULL,
                email TEXT UNIQUE,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS repositories (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                owner_id INTEGER NOT NULL,
                description TEXT,
                is_public BOOLEAN NOT NULL DEFAULT 1,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (owner_id) REFERENCES users (id),
                UNIQUE (name, owner_id)
            )",
            [],
        )?;
        
        // Создаем таблицу для уведомлений
        conn.execute(
            "CREATE TABLE IF NOT EXISTS notifications (
                id INTEGER PRIMARY KEY,
                notification_type TEXT NOT NULL,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                user_id INTEGER NOT NULL,
                is_read BOOLEAN NOT NULL DEFAULT 0,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (user_id) REFERENCES users (id)
            )",
            [],
        )?;
        
        // Создаем таблицу для пул-реквестов
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pull_requests (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                repository_id INTEGER NOT NULL,
                source_branch TEXT NOT NULL,
                target_branch TEXT NOT NULL,
                author_id INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'open',
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (repository_id) REFERENCES repositories (id),
                FOREIGN KEY (author_id) REFERENCES users (id)
            )",
            [],
        )?;
        
        // Создаем таблицу для комментариев к пул-реквестам
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pull_request_comments (
                id INTEGER PRIMARY KEY,
                pull_request_id INTEGER NOT NULL,
                author_id INTEGER NOT NULL,
                content TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (pull_request_id) REFERENCES pull_requests (id),
                FOREIGN KEY (author_id) REFERENCES users (id)
            )",
            [],
        )?;

        // Добавим тестового пользователя, если он ещё не существует
        conn.execute(
            "INSERT OR IGNORE INTO users (username, password, email) VALUES ('Kazilsky', 'password123', 'test@example.com')",
            [],
        )?;

        Ok(Database {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Получает соединение с базой данных
    /// 
    /// # Возвращает
    /// 
    /// * `Arc<Mutex<Connection>>` - Соединение с базой данных
    pub fn get_connection(&self) -> Arc<Mutex<Connection>> {
        self.conn.clone()
    }
}
