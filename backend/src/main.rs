//! Git HTTP сервер с поддержкой Smart HTTP Protocol
//!
//! Этот сервер реализует HTTP интерфейс для работы с Git репозиториями, включая:
//! - Аутентификацию пользователей
//! - Управление репозиториями
//! - Полноценную поддержку Git операций (clone, push, pull) через HTTP
//!
//! # Основные возможности
//! - Регистрация и аутентификация пользователей
//! - Создание и просмотр Git репозиториев
//! - Поддержка Git Smart HTTP Protocol
//! - Доступ к файлам репозиториев через HTTP
//!
//! # Протоколы и endpoints
//! Сервер поддерживает следующие Git HTTP endpoints:
//! - `/info/refs` - получение информации о ссылках
//! - `/git-upload-pack` - для операций fetch/clone
//! - `/git-receive-pack` - для операций push
//! - `/objects/...` - доступ к Git объектам
//!
//! # Безопасность
//! - Все Git операции требуют аутентификации через Basic Auth
//! - CORS настроен только для доверенных origin
//! - Логирование всех операций
//!
//! # Примеры использования
//! ```bash
//! # Клонирование репозитория
//! git clone http://localhost:8000/git/name/repo-name
//!
//! # Push изменений
//! git push origin main
//! ```

use actix_web::{web, App, HttpServer, middleware};
use actix_cors::Cors;
use log::error;

// Импортируем наши модули
mod models;
mod api;

use models::db::Database;
use api::{user, repo, project, git};

struct CorsConfig;

impl CorsConfig {
    /// Создает настроенный CORS middleware
    /// 
    /// # Returns
    /// `actix_cors::Cors` с предустановленными:
    /// - Origin: http://localhost:3000
    /// - Methods: GET, POST, PUT, DELETE
    /// - Headers: Authorization, Content-Type
    /// - Credentials: true
    /// - Max age: 3600
    pub fn configured() -> Cors {
        Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec!["Authorization", "Content-Type"])
            .supports_credentials()
            .max_age(3600)
    }
}

/// Точка входа в приложение - настраивает и запускает HTTP сервер
///
/// # Действия при запуске
/// 1. Инициализирует логгер
/// 2. Создает каталог для репозиториев (если не существует)
/// 3. Инициализирует базу данных
/// 4. Настраивает и запускает HTTP сервер
///
/// # Endpoints
/// Сервер предоставляет следующие группы endpoints:
/// - Аутентификация (/api/auth/...)
/// - Пользователи (/api/user/...)
/// - Репозитории (/api/repos/...)
/// - Git Smart HTTP (/git/...)
///
/// # Ошибки
/// Возвращает `std::io::Error` если не удалось:
/// - Создать каталог репозиториев
/// - Запустить HTTP сервер
///
/// # Пример
/// ```no_run
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     git_http_server::run().await
/// }
/// ```
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Инициализация логгера из переменных окружения
    // По умолчанию уровень логирования - debug
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    // Создаем каталоги для репозиториев и проектов, если они не существуют
    if !std::path::Path::new("repositories").exists() {
        std::fs::create_dir("repositories")?;
    }
    if !std::path::Path::new("projects").exists() {
        std::fs::create_dir("projects")?;
    }
    
    // Инициализация базы данных
    // В случае ошибки программа завершится с сообщением
    let db = Database::new().expect("Failed to initialize database");

    // Настройка и запуск HTTP сервера
    HttpServer::new(move || {
        App::new()
            // Middleware для логирования запросов
            .wrap(middleware::Logger::default())
            .wrap(CorsConfig::configured())
            
            // Общие данные приложения
            .app_data(web::Data::new(db.clone()))
            
            // API для аутентификации
            .service(web::resource("/api/auth/login").route(web::post().to(user::login)))
            .service(web::resource("/api/auth/register").route(web::post().to(user::register)))
            
            // API для работы с пользователями
            .service(web::resource("/api/user/profile").route(web::get().to(user::user_profile)))
            
            // API для работы с проектами
            .service(web::resource("/api/projects").route(web::get().to(project::list_projects)))
            .service(web::resource("/api/projects/public").route(web::get().to(project::list_public_projects)))
            .service(web::resource("/api/projects/create").route(web::post().to(project::create_project)))
            .service(web::resource("/api/projects/{user_name}/{project_name}").route(web::get().to(project::get_project)))
            .service(web::resource("/api/projects/{user_name}/{project_name}/config").route(web::get().to(project::get_project_config)))
            .service(web::resource("/api/projects/{user_name}/{project_name}/config").route(web::put().to(project::update_project_config)))
            .service(web::resource("/api/projects/{user_name}/{project_name}/repos/create").route(web::post().to(project::create_repo_in_project)))
            
            // API для работы с репозиториями (устаревшие endpoints)
            .service(web::resource("/api/repos").route(web::get().to(repo::list_repos)))
            .service(web::resource("/api/repos/create").route(web::post().to(repo::create_repo)))
            .service(web::resource("/api/repos/{user_name}/{repo_name}").route(web::get().to(repo::get_repo)))
            
            // Git Smart HTTP Protocol endpoints
            .service(web::resource("/git/{user_name}/{repo_name}/info/refs")
                .route(web::get().to(git::handle_info_refs)))
            .service(web::resource("/git/{user_name}/{repo_name}/git-upload-pack")
                .route(web::post().to(git::handle_upload_pack)))
            .service(web::resource("/git/{user_name}/{repo_name}/git-receive-pack")
                .route(web::post().to(git::handle_receive_pack)))
            
            // Доступ к Git объектам
            .service(web::resource("/git/{user_name}/{repo_name}/objects/info/packs")
                .route(web::get().to(git::handle_info_packs)))
            .service(web::resource("/git/{user_name}/{repo_name}/objects/pack/{pack_file}")
                .route(web::get().to(git::handle_pack_file)))
            
            // Доступ к файлам репозитория
            .service(web::resource("/git/{user_name}/{repo_name}/file/{tail:.*}")
                .route(web::get().to(git::handle_text_file)))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
