use actix_web::{web, HttpResponse, Result, HttpRequest};
use crate::models::db::Database;
use crate::handlers::auth::check_auth;
use crate::models::repository::Repository;
use tera::Context;
use log::{debug, error};

/// Обработчик главной страницы
/// 
/// # Параметры
/// 
/// * `req` - HTTP запрос
/// * `tmpl` - Шаблонизатор
/// * `db` - Подключение к базе данных
/// 
/// # Возвращает
/// 
/// * `Result<HttpResponse>` - HTTP ответ
pub async fn index(req: HttpRequest, tmpl: web::Data<tera::Tera>, db: web::Data<Database>) -> Result<HttpResponse> {
    // Создаем контекст для шаблона
    let mut ctx = Context::new();
    
    // Если пользователь авторизован, добавляем его в контекст
    if let Some(user) = check_auth(&req, &db) {
        ctx.insert("user", &user);
        
        // Получаем репозитории пользователя
        let conn = db.get_connection();
        match Repository::find_by_owner(user.id.unwrap(), conn) {
            Ok(repos) => {
                ctx.insert("repos", &repos);
            },
            Err(e) => {
                error!("Failed to fetch repositories: {}", e);
            }
        }
    }
    
    // Рендерим шаблон
    let rendered = tmpl.render("index.html", &ctx)
        .map_err(|e| {
            error!("Template error: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(rendered))
}

/// Обработчик страницы документации 
/// 
/// # Параметры
/// 
/// * `req` - HTTP запрос
/// * `tmpl` - Шаблонизатор
/// * `db` - Подключение к базе данных
/// 
/// # Возвращает
/// 
/// * `Result<HttpResponse>` - HTTP ответ
pub async fn docs(req: HttpRequest, tmpl: web::Data<tera::Tera>, db: web::Data<Database>) -> Result<HttpResponse> {
    let mut ctx = Context::new();
    
    // Если пользователь авторизован, добавляем его в контекст
    if let Some(user) = check_auth(&req, &db) {
        ctx.insert("user", &user);
    }
    
    let rendered = tmpl.render("docs.html", &ctx)
        .map_err(|e| {
            error!("Template error: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(rendered))
}

/// Обработчик для страницы профиля пользователя
/// 
/// # Параметры
/// 
/// * `req` - HTTP запрос
/// * `username` - Имя пользователя
/// * `tmpl` - Шаблонизатор
/// * `db` - Подключение к базе данных
/// 
/// # Возвращает
/// 
/// * `Result<HttpResponse>` - HTTP ответ
pub async fn profile(
    req: HttpRequest, 
    username: web::Path<String>,
    tmpl: web::Data<tera::Tera>, 
    db: web::Data<Database>
) -> Result<HttpResponse> {
    let username = username.into_inner();
    let conn = db.get_connection();
    
    // Получаем информацию о пользователе
    let target_user = match crate::models::user::User::find_by_username(&username, conn.clone()) {
        Ok(Some(user)) => user,
        _ => {
            return Ok(HttpResponse::NotFound().body("User not found"));
        }
    };
    
    // Получаем репозитории пользователя
    let repos = match Repository::find_by_owner(target_user.id.unwrap(), conn) {
        Ok(repos) => repos,
        Err(e) => {
            error!("Failed to fetch repositories: {}", e);
            vec![]
        }
    };
    
    let mut ctx = Context::new();
    ctx.insert("profile_user", &target_user);
    ctx.insert("repos", &repos);
    
    // Проверяем, авторизован ли текущий пользователь
    if let Some(current_user) = check_auth(&req, &db) {
        ctx.insert("user", &current_user);
        ctx.insert("is_own_profile", &(current_user.username == target_user.username));
    } else {
        ctx.insert("is_own_profile", &false);
    }
    
    let rendered = tmpl.render("profile.html", &ctx)
        .map_err(|e| {
            error!("Template error: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(rendered))
} 