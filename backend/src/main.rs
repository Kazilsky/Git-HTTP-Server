use actix_web::{web, App, HttpServer, HttpResponse, HttpRequest, middleware};
use actix_cors::Cors;
use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::io::Write;
use log::{debug, error};
use std::fs;

// Импортируем наши модули
mod models;
mod handlers;

use models::db::Database;
use handlers::api;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    // Создаем каталог для репозиториев, если он не существует
    if !std::path::Path::new("repositories").exists() {
        std::fs::create_dir("repositories")?;
    }
    
    // Инициализируем базу данных
    let db = Database::new().expect("Failed to initialize database");

    HttpServer::new(move || {
        // Настройка CORS для взаимодействия с React
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec!["Authorization", "Content-Type"])
            .supports_credentials()
            .max_age(3600);

        App::new()
            // Добавляем middleware
            .wrap(middleware::Logger::default())
            .wrap(cors)
            // Данные приложения
            .app_data(web::Data::new(db.clone()))
            
            // API для аутентификации и пользователей
            .service(web::resource("/api/auth/login").route(web::post().to(api::login)))
            .service(web::resource("/api/auth/register").route(web::post().to(api::register)))
            .service(web::resource("/api/user/profile").route(web::get().to(api::user_profile)))
            
            // API для репозиториев
            .service(web::resource("/api/repos").route(web::get().to(api::list_repos)))
            .service(web::resource("/api/repos").route(web::post().to(api::create_repo)))
            .service(web::resource("/api/repos/{repo_name}").route(web::get().to(api::get_repo)))
            
            // Smart HTTP Protocol endpoints для Git
            .service(web::resource("/git/{repo_name}/info/refs")
                .route(web::get().to(handle_info_refs)))
            .service(web::resource("/git/{repo_name}/git-upload-pack")
                .route(web::post().to(handle_upload_pack)))
            .service(web::resource("/git/{repo_name}/git-receive-pack")
                .route(web::post().to(handle_receive_pack)))
            // Pack files endpoints
            .service(web::resource("/git/{repo_name}/objects/info/packs")
                .route(web::get().to(handle_info_packs)))
            .service(web::resource("/git/{repo_name}/objects/pack/{pack_file}")
                .route(web::get().to(handle_pack_file)))
            // Text file endpoint
            .service(web::resource("/git/{repo_name}/file/{tail:.*}")
                .route(web::get().to(handle_text_file)))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}

/// Обработчик для /info/refs - первый этап Git протокола
/// Когда клиент выполняет git clone/pull/push, он сначала запрашивает этот эндпоинт
/// чтобы узнать, какие ссылки (refs) доступны на сервере и какие операции поддерживаются
async fn handle_info_refs(req: HttpRequest) -> HttpResponse {
    // Проверяем авторизацию
    if api::check_auth(&req, &req.app_data::<web::Data<Database>>().unwrap()).is_none() {
        return HttpResponse::Unauthorized()
            .append_header(("WWW-Authenticate", "Basic realm=\"Git\""))
            .finish();
    }

    let repo_name = req.match_info().get("repo_name").unwrap();
    let service = req.query_string();
    
    debug!("Handling info/refs for repo: {}, service: {}", repo_name, service);
    
    // Извлекаем имя сервиса (git-upload-pack или git-receive-pack)
    let service = match service.strip_prefix("service=") {
        Some(s) => s,
        None => return HttpResponse::BadRequest().finish()
    };

    let repo_path = PathBuf::from("repositories").join(format!("{}.git", repo_name));

    // Выбираем команду в зависимости от запрошенного сервиса
    let git_command = if service == "git-upload-pack" { "upload-pack" } else { "receive-pack" };

    // Запускаем git команду с флагом --advertise-refs для получения списка ссылок
    let output = Command::new("git")
        .arg(git_command)
        .arg("--advertise-refs")
        .arg(&repo_path)
        .output()
        .expect("Failed to execute git command");

    if !output.status.success() {
        error!("git command failed: {}", String::from_utf8_lossy(&output.stderr));
        return HttpResponse::InternalServerError().finish();
    }

    // Формируем ответ в формате Smart HTTP Protocol
    let mut response = Vec::new();
    
    // PKT-LINE формат:
    // <4-byte length><payload>
    // Где <4-byte length> - это ASCII hex длина пакета (включая 4 байта длины)
    
    // Сервисный заголовок
    let service_header = format!("# service={}\n", service);
    let header_length = service_header.len() + 4; // +4 для самой длины
    response.extend_from_slice(format!("{:04x}", header_length).as_bytes());
    response.extend_from_slice(service_header.as_bytes());
    
    // Разделитель
    response.extend_from_slice(b"0000");
    
    // Добавляем вывод git-*-pack --advertise-refs
    response.extend_from_slice(&output.stdout);
    
    // Возвращаем результат
    HttpResponse::Ok()
        .content_type(format!("application/x-{}-advertisement", service))
        .body(response)
}

/// Обработчик для git-upload-pack - используется при git clone/fetch
/// Клиент запрашивает определенные объекты, сервер их упаковывает и отправляет
async fn handle_upload_pack(req: HttpRequest, body: web::Bytes) -> HttpResponse {
    // Проверяем авторизацию
    if api::check_auth(&req, &req.app_data::<web::Data<Database>>().unwrap()).is_none() {
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

    // Передаем запрос клиента в git-upload-pack
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(&body).expect("Failed to write to git-upload-pack stdin");
        drop(stdin);  // Важно закрыть stdin, чтобы процесс знал, что ввод закончен
    }

    let output = child.wait_with_output().expect("Failed to wait for git-upload-pack");

    if !output.status.success() {
        error!("git-upload-pack failed: {}", String::from_utf8_lossy(&output.stderr));
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok()
        .content_type("application/x-git-upload-pack-result")
        .body(output.stdout)
}

/// Обработчик для git-receive-pack - используется при git push
/// Клиент отправляет новые объекты, сервер их принимает и обновляет ссылки
async fn handle_receive_pack(req: HttpRequest, body: web::Bytes) -> HttpResponse {
    // Проверяем авторизацию
    let _username = match api::check_auth(&req, &req.app_data::<web::Data<Database>>().unwrap()) {
        Some(user) => user.username,
        None => return HttpResponse::Unauthorized()
            .append_header(("WWW-Authenticate", "Basic realm=\"Git\""))
            .finish()
    };

    let repo_name = req.match_info().get("repo_name").unwrap();
    let repo_path = PathBuf::from("repositories").join(format!("{}.git", repo_name));

    debug!("Handling receive-pack for repo: {}", repo_name);

    let mut child = Command::new("git")
        .arg("receive-pack")
        .arg("--stateless-rpc")
        .arg(&repo_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn git-receive-pack");

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(&body).expect("Failed to write to git-receive-pack stdin");
        drop(stdin);
    }

    let output = child.wait_with_output().expect("Failed to wait for git-receive-pack");

    if !output.status.success() {
        error!("git-receive-pack failed: {}", String::from_utf8_lossy(&output.stderr));
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok()
        .content_type("application/x-git-receive-pack-result")
        .body(output.stdout)
}

/// Обработчик для objects/info/packs - возвращает список доступных pack-файлов
/// Pack-файлы содержат сжатые Git объекты для эффективной передачи
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
/// Используется, например, для просмотра README, LICENSE и других файлов
async fn handle_text_file(req: HttpRequest) -> HttpResponse {
    let repo_name = req.match_info().get("repo_name").unwrap();
    let path = req.match_info().get("tail").unwrap();
    
    debug!("Getting file: {} from repo: {}", path, repo_name);
    
    // Используем git show для получения содержимого файла
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
