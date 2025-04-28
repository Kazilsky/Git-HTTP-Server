use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

/// База данных для хранения информации о пользователях и репозиториях
#[derive(Clone)]
pub struct Database {
    /// Соединение с базой данных SQLite
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Создаёт новый экземпляр базы данных
    /// 
    /// # Возвращает
    /// 
    /// * `Result<Database>` - Результат создания базы данных
    pub fn new() -> Result<Self> {
        let conn = Connection::open("gitea.db")?;
        
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