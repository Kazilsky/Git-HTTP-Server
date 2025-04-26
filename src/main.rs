use actix_web::{web, App, HttpServer, HttpResponse, HttpRequest, Result};
use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::io::Write;
use std::collections::HashMap;
use log::{debug, error};
use std::fs;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use lazy_static::lazy_static;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    HttpServer::new(|| {
        App::new()
            // Smart HTTP Protocol endpoints
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
            // Text file endpoint - используем path param для оставшейся части пути
            .service(web::resource("/git/{repo_name}/file/{tail:.*}")
                .route(web::get().to(handle_text_file)))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}

// Простая "база данных" пользователей
lazy_static! {
    static ref USERS: HashMap<String, String> = {
        let mut m = HashMap::new();
        m.insert("Kazilsky".to_string(), "password123".to_string());
        m
    };
}

// Проверка авторизации
fn check_auth(req: &HttpRequest) -> Option<String> {
    // Получаем заголовок Authorization
    let auth_header = req.headers().get("Authorization")?;
    let auth_str = auth_header.to_str().ok()?;
    
    // Проверяем, что это Basic Auth
    if !auth_str.starts_with("Basic ") {
        return None;
    }

    // Декодируем Base64
    let credentials = BASE64.decode(auth_str.trim_start_matches("Basic "))
        .ok()?;
    let credentials_str = String::from_utf8(credentials).ok()?;
    
    // Разделяем на username:password
    let mut parts = credentials_str.splitn(2, ':');
    let username = parts.next()?;
    let password = parts.next()?;

    // Проверяем пароль
    if let Some(stored_password) = USERS.get(username) {
        if password == stored_password {
            return Some(username.to_string());
        }
    }
    log!("{}", credentials_str)
    None
}

/// Обработчик для /info/refs - первый этап Git протокола
/// Когда клиент выполняет git clone/pull/push, он сначала запрашивает этот эндпоинт
/// чтобы узнать, какие ссылки (refs) доступны на сервере и какие операции поддерживаются
async fn handle_info_refs(req: HttpRequest) -> HttpResponse {
    // Проверяем авторизацию
    if check_auth(&req).is_none() {
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
    // - первые 4 символа - длина строки в hex (включая сами 4 символа)
    // - затем содержимое строки
    let header = format!("# service={}\n", service);
    let header_len = format!("{:04x}", header.len() + 4);
    response.write_all(header_len.as_bytes()).unwrap();
    response.write_all(header.as_bytes()).unwrap();
    
    // Flush-pkt (0000) - разделитель в протоколе
    response.write_all(b"0000").unwrap();
    
    // Добавляем список ссылок от git команды
    response.write_all(&output.stdout).unwrap();

    HttpResponse::Ok()
        .content_type(format!("application/x-{}-advertisement", service))
        .body(response)
}

/// Обработчик для git-upload-pack - используется при git clone/fetch
/// Клиент запрашивает определенные объекты, сервер их упаковывает и отправляет
async fn handle_upload_pack(req: HttpRequest, body: web::Bytes) -> HttpResponse {
    // Проверяем авторизацию
    if check_auth(&req).is_none() {
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
    let username = match check_auth(&req) {
        Some(username) => username,
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
    
    debug!("Handling text file request for repo: {}, path: {}", repo_name, path);

    let repo_path = PathBuf::from("repositories")
        .join(format!("{}.git", repo_name));

    // Используем git show для получения содержимого файла
    let output = Command::new("git")
        .arg("--git-dir")
        .arg(&repo_path)
        .arg("show")
        .arg(format!("HEAD:{}", path))
        .output()
        .expect("Failed to execute git show");

    if output.status.success() {
        HttpResponse::Ok()
            .content_type("text/plain")
            .body(output.stdout)
    } else {
        debug!("File not found or error: {}", String::from_utf8_lossy(&output.stderr));
        HttpResponse::NotFound().finish()
    }
}