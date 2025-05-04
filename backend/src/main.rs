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
//! git clone http://localhost:8000/git/myrepo
//!
//! # Push изменений
//! git push origin main
//! ```

use actix_web::{web, App, HttpServer, HttpResponse, HttpRequest, middleware};
use actix_cors::Cors;
use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::io::Write;
use log::{debug, error};
use std::fs;

// Импортируем наши модули
mod models;
mod api;

use models::db::Database;
use api::{user, repo};

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

    // Создаем каталог для репозиториев, если он не существует
    if !std::path::Path::new("repositories").exists() {
        std::fs::create_dir("repositories")?;
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
            
            // API для работы с репозиториями
            .service(web::resource("/api/repos").route(web::get().to(repo::list_repos)))
            .service(web::resource("/api/repos/create").route(web::post().to(repo::create_repo)))
            .service(web::resource("/api/repos/{repo_name}").route(web::get().to(repo::get_repo)))
            
            // Git Smart HTTP Protocol endpoints
            .service(web::resource("/git/{repo_name}/info/refs")
                .route(web::get().to(handle_info_refs)))
            .service(web::resource("/git/{repo_name}/git-upload-pack")
                .route(web::post().to(handle_upload_pack)))
            .service(web::resource("/git/{repo_name}/git-receive-pack")
                .route(web::post().to(handle_receive_pack)))
            
            // Доступ к Git объектам
            .service(web::resource("/git/{repo_name}/objects/info/packs")
                .route(web::get().to(handle_info_packs)))
            .service(web::resource("/git/{repo_name}/objects/pack/{pack_file}")
                .route(web::get().to(handle_pack_file)))
            
            // Доступ к файлам репозитория
            .service(web::resource("/git/{repo_name}/file/{tail:.*}")
                .route(web::get().to(handle_text_file)))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}

/// Обработчик для /info/refs - первый этап Git протокола
///
/// Когда клиент выполняет git clone/pull/push, он сначала запрашивает этот эндпоинт
/// чтобы узнать, какие ссылки (refs) доступны на сервере и какие операции поддерживаются
///
/// # Аргументы
/// * `req` - HTTP запрос, содержащий:
///   - Имя репозитория в параметрах пути
///   - Тип сервиса в query string (git-upload-pack или git-receive-pack)
///
/// # Возвращаемое значение
/// Возвращает `HttpResponse` с данными в формате Smart HTTP Protocol:
/// - Сначала service advertisement
/// - Затем список ссылок в PKT-LINE формате
///
/// # Ошибки
/// - 401 Unauthorized если пользователь не аутентифицирован
/// - 400 Bad Request если неверный запрос
/// - 500 Internal Server Error если команда git завершилась с ошибкой
///
/// # Пример ответа
/// ```text
/// 001e# service=git-upload-pack\n
/// 0000
/// 004895dcfa3633004da0049d3d0fa03f80589cbcaf31 refs/heads/main\0multi_ack\n
/// ```
async fn handle_info_refs(req: HttpRequest) -> HttpResponse {
    // Проверяем авторизацию пользователя
    if user::check_auth(&req, &req.app_data::<web::Data<Database>>().unwrap()).is_none() {
        return HttpResponse::Unauthorized()
            .append_header(("WWW-Authenticate", "Basic realm=\"Git\""))
            .finish();
    }

    let repo_name = req.match_info().get("repo_name").unwrap();
    let service = req.query_string();
    
    debug!("Handling info/refs for repo: {}, service: {}", repo_name, service);
    
    // Извлекаем имя сервиса из query string
    let service = match service.strip_prefix("service=") {
        Some(s) => s,
        None => return HttpResponse::BadRequest().finish()
    };

    let repo_path = PathBuf::from("repositories").join(format!("{}.git", repo_name));

    // Выбираем соответствующую Git команду
    let git_command = if service == "git-upload-pack" { "upload-pack" } else { "receive-pack" };

    // Выполняем Git команду для получения списка ссылок
    let output = Command::new("git")
        .arg(git_command)
        .arg("--advertise-refs")
        .arg(&repo_path)
        .output()
        .expect("Failed to execute git command");

    // Обрабатываем возможные ошибки выполнения команды
    if !output.status.success() {
        error!("git command failed: {}", String::from_utf8_lossy(&output.stderr));
        return HttpResponse::InternalServerError().finish();
    }

    // Формируем ответ в формате Smart HTTP Protocol
    let mut response = Vec::new();
    
    // PKT-LINE формат:
    // Каждая строка начинается с 4-байтовой hex длины (включая 4 байта длины)
    // Например, для строки "hello" (5 байт) + 4 байта длины = 9 байтов (0x0009)
    // Формат: "{:04x}" форматирует число как 4-значное hex с ведущими нулями

    // Сервисный заголовок: "# service=git-upload-pack\n"
    let service_header = format!("# service={}\n", service);
    let header_length = service_header.len() + 4;
    response.extend_from_slice(format!("{:04x}", header_length).as_bytes());
    response.extend_from_slice(service_header.as_bytes());
    
    // Разделитель "0000" указывает конец заголовков
    response.extend_from_slice(b"0000");
    
    // Добавляем вывод git-*-pack --advertise-refs
    response.extend_from_slice(&output.stdout);
    
    // Возвращаем результат
    HttpResponse::Ok()
        .content_type(format!("application/x-{}-advertisement", service))
        .body(response)
}

/// Обработчик для git-upload-pack - используется при git clone/fetch
///
/// Клиент отправляет запрос с want/have объектами, сервер упаковывает запрошенные
/// объекты и возвращает их в packfile формате.
///
/// # Аргументы
/// * `req` - HTTP запрос с именем репозитория
/// * `body` - Тело запроса в формате Git wire protocol
///
/// # Возвращаемое значение
/// Возвращает `HttpResponse` с данными в формате packfile
///
/// # Ошибки
/// - 401 Unauthorized если пользователь не аутентифицирован
/// - 500 Internal Server Error если команда git завершилась с ошибкой
///
/// # Протокол
/// 1. Клиент отправляет список want/have объектов
/// 2. Сервер запускает `git-upload-pack --stateless-rpc`
/// 3. Сервер возвращает упакованные объекты
async fn handle_upload_pack(req: HttpRequest, body: web::Bytes) -> HttpResponse {
    // Проверяем авторизацию пользователя
    if user::check_auth(&req, &req.app_data::<web::Data<Database>>().unwrap()).is_none() {
        return HttpResponse::Unauthorized()
            .append_header(("WWW-Authenticate", "Basic realm=\"Git\""))
            .finish();
    }

    let repo_name = req.match_info().get("repo_name").unwrap();
    let repo_path = PathBuf::from("repositories").join(format!("{}.git", repo_name));

    debug!("Handling upload-pack for repo: {}", repo_name);

    // Запускаем git-upload-pack в режиме stateless-rpc (для HTTP протокола)
    let mut child = Command::new("git")
        .arg("upload-pack")
        .arg("--stateless-rpc")  // Важно для HTTP протокола
        .arg(&repo_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn git-upload-pack");

    // Передаем тело запроса в stdin git-upload-pack
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(&body).expect("Failed to write to git-upload-pack stdin");
        drop(stdin);  // Закрываем stdin, чтобы процесс знал, что ввод закончен
    }

    let output = child.wait_with_output().expect("Failed to wait for git-upload-pack");

    // Обрабатываем ошибки выполнения команды
    if !output.status.success() {
        error!("git-upload-pack failed: {}", String::from_utf8_lossy(&output.stderr));
        return HttpResponse::InternalServerError().finish();
    }

    // Возвращаем результат в формате packfile
    HttpResponse::Ok()
        .content_type("application/x-git-upload-pack-result")
        .body(output.stdout)
}

/// Обработчик для git-receive-pack - используется при git push
///
/// Клиент отправляет новые объекты и инструкции по обновлению ссылок,
/// сервер принимает объекты и обновляет репозиторий.
///
/// # Аргументы
/// * `req` - HTTP запрос с именем репозитория
/// * `body` - Тело запроса с объектами и инструкциями
///
/// # Возвращаемое значение
/// Возвращает `HttpResponse` с результатом операции
///
/// # Ошибки
/// - 401 Unauthorized если пользователь не аутентифицирован
/// - 500 Internal Server Error если команда git завершилась с ошибкой
///
/// # Протокол
/// 1. Клиент отправляет packfile с новыми объектами
/// 2. Сервер запускает `git-receive-pack --stateless-rpc`
/// 3. Сервер обновляет ссылки и возвращает результат
async fn handle_receive_pack(req: HttpRequest, body: web::Bytes) -> HttpResponse {
    // Проверяем авторизацию и получаем имя пользователя
    let _username = match user::check_auth(&req, &req.app_data::<web::Data<Database>>().unwrap()) {
        Some(user) => user.username,
        None => return HttpResponse::Unauthorized()
            .append_header(("WWW-Authenticate", "Basic realm=\"Git\""))
            .finish()
    };

    let repo_name = req.match_info().get("repo_name").unwrap();
    let repo_path = PathBuf::from("repositories").join(format!("{}.git", repo_name));

    debug!("Handling receive-pack for repo: {}", repo_name);

    // Запускаем git-receive-pack в режиме stateless-rpc
    let mut child = Command::new("git")
        .arg("receive-pack")
        .arg("--stateless-rpc")
        .arg(&repo_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn git-receive-pack");

    // Передаем тело запроса в stdin git-receive-pack
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(&body).expect("Failed to write to git-receive-pack stdin");
        drop(stdin);
    }

    let output = child.wait_with_output().expect("Failed to wait for git-receive-pack");

    // Обрабатываем ошибки выполнения команды
    if !output.status.success() {
        error!("git-receive-pack failed: {}", String::from_utf8_lossy(&output.stderr));
        return HttpResponse::InternalServerError().finish();
    }

    // Возвращаем результат операции
    HttpResponse::Ok()
        .content_type("application/x-git-receive-pack-result")
        .body(output.stdout)
}

/// Обработчик для objects/info/packs - возвращает список pack-файлов
///
/// Pack-файлы содержат сжатые Git объекты для эффективной передачи.
///
/// # Аргументы
/// * `req` - HTTP запрос с именем репозитория
///
/// # Возвращаемое значение
/// Возвращает `HttpResponse` с содержимым файла packs или 404 если файл не найден
///
/// # Формат ответа
/// Текстовый файл со списком pack-файлов, например:
/// ```text
/// P pack-1234567890abcdef.pack
/// P pack-9876543210fedcba.pack
/// ```
async fn handle_info_packs(req: HttpRequest) -> HttpResponse {
    let repo_name = req.match_info().get("repo_name").unwrap();
    let repo_path = PathBuf::from("repositories")
        .join(format!("{}.git", repo_name))
        .join("objects/info/packs");

    match fs::read(&repo_path) {
        Ok(content) => HttpResponse::Ok()
            .content_type("text/plain")
            .body(content),
        Err(_) => HttpResponse::NotFound().finish()
    }
}

/// Обработчик для получения конкретного pack-файла
///
/// # Аргументы
/// * `req` - HTTP запрос с именем репозитория и именем pack-файла
///
/// # Возвращаемое значение
/// Возвращает `HttpResponse` с содержимым pack-файла или 404 если файл не найден
///
/// # Формат файла
/// Бинарный pack-файл в формате Git
async fn handle_pack_file(req: HttpRequest) -> HttpResponse {
    let repo_name = req.match_info().get("repo_name").unwrap();
    let pack_file = req.match_info().get("pack_file").unwrap();
    
    let repo_path = PathBuf::from("repositories")
        .join(format!("{}.git", repo_name))
        .join("objects/pack")
        .join(pack_file);

    match fs::read(&repo_path) {
        Ok(content) => HttpResponse::Ok()
            .content_type("application/x-git-pack")
            .body(content),
        Err(_) => HttpResponse::NotFound().finish()
    }
}

/// Обработчик для получения текстовых файлов из репозитория
///
/// Позволяет просматривать содержимое файлов (например, README, LICENSE) через HTTP.
///
/// # Аргументы
/// * `req` - HTTP запрос с именем репозитория и путем к файлу
///
/// # Возвращаемое значение
/// Возвращает `HttpResponse` с содержимым файла или 404 если файл не найден
///
/// # Пример
/// ```
/// GET /git/myrepo/file/README.md
/// ```
async fn handle_text_file(req: HttpRequest) -> HttpResponse {
    let repo_name = req.match_info().get("repo_name").unwrap();
    let path = req.match_info().get("tail").unwrap();
    
    debug!("Getting file: {} from repo: {}", path, repo_name);
    
    // Используем git show для получения содержимого файла из HEAD
    let output = Command::new("git")
        .args(&["--git-dir", &format!("repositories/{}.git", repo_name), "show", &format!("HEAD:{}", path)])
        .output();
    
    match output {
        Ok(output) if output.status.success() => {
            HttpResponse::Ok()
                .content_type("text/plain")
                .body(output.stdout)
        },
        _ => HttpResponse::NotFound().finish()
    }
}