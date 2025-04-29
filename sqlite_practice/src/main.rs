use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

// Модель задачи
#[derive(Debug, Serialize, Deserialize)]
struct Task {
    id: Option<i64>,        // Option<i64> - может быть None при создании
    title: String,
    description: String,
    completed: bool,
    created_at: u64,        // Unix timestamp
}

#[derive(Debug, Serialize, Deserialize)]
struct Category {
    id: Option<i64>,
    name: String,
    description: String,
    color: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskCategory {
    id: Option<i64>,
    task_id: i64,
    category_id: i64,
}


impl Task {
    // Конструктор для создания новой задачи
    fn new(title: &str, description: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Task {
            id: None,
            title: title.to_string(),
            description: description.to_string(),
            completed: false,
            created_at: timestamp,
        }
    }
}

impl Category {
    fn new(name: &str, description: &str, color: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Category {
            id: None,
            name: name.to_string(),
            description: description.to_string(),
            color: color.to_string(),
            created_at: timestamp,
        }
    }
}

impl TaskCategory {
    fn new(task_id: i64, category_id: i64) -> Self {
        let timestamp = SystemTime::now()   
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        TaskCategory {
            id: None,
            task_id,
            category_id,
            created_at: timestamp,
        }
    }
}

// Менеджер базы данных
struct TaskDB {
    conn: Connection,
}

impl TaskDB {
    // Создаем новое подключение и таблицу, если она не существует
    fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        
        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                completed BOOLEAN NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL
            )

            CREATE TABLE IF NOT EXISTS categories (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                color TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )

            CREATE TABLE IF NOT EXISTS task_categories (
                id INTEGER PRIMARY KEY,
                task_id INTEGER NOT NULL,
                category_id INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (task_id) REFERENCES tasks (id),
                FOREIGN KEY (category_id) REFERENCES categories (id)
            )
            ",
            [],
        )?;
        
        Ok(TaskDB { conn })
    }
    
    // Добавить новую задачу
    fn add_task(&self, task: &Task) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO tasks (title, description, completed, created_at) 
             VALUES (?1, ?2, ?3, ?4)",
            params![
                task.title,
                task.description,
                task.completed,
                task.created_at,
            ],
        )?;
        
        Ok(self.conn.last_insert_rowid())
    }

    fn add_category(&self, category: &Category) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO categories (name, description, color, created_at) 
             VALUES (?1, ?2, ?3, ?4)",
            params![category.name, category.description, category.color, category.created_at],
        )?;
    }
    
    fn add_task_category(&self, task_category: &TaskCategory) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO task_categories (task_id, category_id, created_at) 
             VALUES (?1, ?2, ?3)",
            params![task_category.task_id, task_category.category_id, task_category.created_at],
        )?;
    }

    // Получить задачу по ID
    fn get_task(&self, id: i64) -> Result<Option<Task>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, completed, created_at 
             FROM tasks 
             WHERE id = ?1"
        )?;
        
        let task_iter = stmt.query_map(params![id], |row| {
            Ok(Task {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                description: row.get(2)?,
                completed: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
        
        // Извлекаем первый (и единственный) результат или None
        for task in task_iter {
            return Ok(Some(task?));
        }
        
        Ok(None)
    }
    
    // Получить все задачи
    fn get_all_tasks(&self) -> Result<Vec<Task>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, completed, created_at 
             FROM tasks 
             ORDER BY created_at DESC"
        )?;
        
        let task_iter = stmt.query_map([], |row| {
            Ok(Task {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                description: row.get(2)?,
                completed: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
        
        let mut tasks = Vec::new();
        for task in task_iter {
            tasks.push(task?);
        }
        
        Ok(tasks)
    }
    
    // Обновить существующую задачу
    fn update_task(&self, task: &Task) -> Result<usize> {
        let id = task.id.ok_or(rusqlite::Error::InvalidParameterName(
            "Task id cannot be None for update".to_string(),
        ))?;
        
        self.conn.execute(
            "UPDATE tasks 
             SET title = ?1, description = ?2, completed = ?3
             WHERE id = ?4",
            params![
                task.title,
                task.description,
                task.completed,
                id,
            ],
        )
    }
    
    // Удалить задачу
    fn delete_task(&self, id: i64) -> Result<usize> {
        self.conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])
    }
    
    // Пометить задачу как выполненную
    fn mark_completed(&self, id: i64, completed: bool) -> Result<usize> {
        self.conn.execute(
            "UPDATE tasks SET completed = ?1 WHERE id = ?2",
            params![completed, id],
        )
    }
}

fn main() -> Result<()> {
    println!("=== SQLite Task Manager ===");
    
    // Создаем подключение к БД
    let db = TaskDB::new("tasks.db")?;
    
    // Примеры CRUD операций:
    
    // Create: Добавляем несколько задач
    let task1 = Task::new(
        "Изучить Rust", 
        "Пройти курс по Rust и понять концепцию владения"
    );
    let task1_id = db.add_task(&task1)?;
    println!("Задача 1 добавлена с ID: {}", task1_id);
    
    let task2 = Task::new(
        "Понять Actix-Web", 
        "Разобраться с маршрутизацией и внедрением зависимостей"
    );
    let task2_id = db.add_task(&task2)?;
    println!("Задача 2 добавлена с ID: {}", task2_id);
    
    // Read: Получаем задачу по ID
    if let Some(task) = db.get_task(task1_id)? {
        println!("\nЗадача по ID {}:", task1_id);
        println!("  Заголовок: {}", task.title);
        println!("  Описание: {}", task.description);
        println!("  Выполнена: {}", task.completed);
    }
    
    // Update: Обновляем задачу
    let mut updated_task = db.get_task(task2_id)?.unwrap();
    updated_task.title = "Понять Actix-Web в деталях".to_string();
    updated_task.description = "Разобраться с маршрутизацией, внедрением зависимостей и middleware".to_string();
    let updated = db.update_task(&updated_task)?;
    println!("\nОбновлено строк: {}", updated);
    
    // Отмечаем задачу как выполненную
    db.mark_completed(task1_id, true)?;
    println!("Задача 1 отмечена как выполненная");
    
    // Read All: Получаем все задачи
    println!("\nСписок всех задач:");
    let all_tasks = db.get_all_tasks()?;
    for (i, task) in all_tasks.iter().enumerate() {
        println!("{}. {} [{}]", 
            i + 1, 
            task.title, 
            if task.completed { "✓" } else { "✗" }
        );
    }
    
    // Delete: Удаляем задачу
    db.delete_task(task2_id)?;
    println!("\nЗадача 2 удалена");
    
    // Проверяем что осталось
    println!("\nОставшиеся задачи:");
    let remaining_tasks = db.get_all_tasks()?;
    for (i, task) in remaining_tasks.iter().enumerate() {
        println!("{}. {} [{}]", 
            i + 1, 
            task.title, 
            if task.completed { "✓" } else { "✗" }
        );
    }
    
    Ok(())
} 