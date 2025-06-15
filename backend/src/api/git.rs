use actix_web::{web, HttpResponse, HttpRequest, Result};
use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::io::Write;
use log::{debug, error};
use std::fs;
use crate::models::db::Database;
use super::user;

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
pub async fn handle_info_refs(req: HttpRequest) -> HttpResponse {
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
pub async fn handle_upload_pack(req: HttpRequest, body: web::Bytes) -> HttpResponse {
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
pub async fn handle_receive_pack(req: HttpRequest, body: web::Bytes) -> HttpResponse {
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
pub async fn handle_info_packs(req: HttpRequest) -> HttpResponse {
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
pub async fn handle_pack_file(req: HttpRequest) -> HttpResponse {
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
pub async fn handle_text_file(req: HttpRequest) -> HttpResponse {
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
