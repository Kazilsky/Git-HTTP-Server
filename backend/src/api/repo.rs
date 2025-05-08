use actix_web::{web, HttpResponse, HttpRequest, Result};
use crate::models::db::Database;
use crate::models::repository::Repository;
use log::{error};
use serde::{Serialize, Deserialize};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::process::Command;

use super::user;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<T>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateRepoRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<T>,
}

/// Проверяет аутентификацию пользователя по HTTP заголовку
pub fn check_auth(req: &HttpRequest, db: &web::Data<Database>) -> Option<User> {
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

    // Проверяем в базе данных
    let conn = db.get_connection();
    match User::authenticate(username, password, conn) {
        Ok(Some(user)) => Some(user),
        _ => None
    }
}

//pub fn check_notification(req: &HttpResponse, db: &web::Data<Database>) -> Option<Notification> {

//}

/// Обработчик для авторизации пользователя
pub async fn login(login_req: web::Json<LoginRequest>, db: web::Data<Database>) -> Result<HttpResponse> {
    let conn = db.get_connection();
    
    match User::authenticate(&login_req.username, &login_req.password, conn) {
        Ok(Some(user)) => {
            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: Some("Login successful".to_string()),
                data: Some(user),
            }))
        },
        _ => {
            Ok(HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                message: Some("Invalid username or password".to_string()),
                data: None,
            }))
        }
    }
}

/// Обработчик для регистрации нового пользователя
pub async fn register(register_req: web::Json<RegisterRequest>, db: web::Data<Database>) -> Result<HttpResponse> {
    let conn = db.get_connection();
    
    // Проверяем, что пользователь с таким именем не существует
    match User::find_by_username(&register_req.username, conn.clone()) {
        Ok(Some(_)) => {
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                message: Some("User with this username already exists".to_string()),
                data: None,
            }))
        },
        Ok(None) => {
            // Создаем нового пользователя
            let user = User {
                id: None,
                username: register_req.username.clone(),
                password: register_req.password.clone(), // В реальном приложении пароль нужно хэшировать!
                email: register_req.email.clone(),
                created_at: None,
            };
            
            match user.create(conn) {
                Ok(_) => {
                    Ok(HttpResponse::Ok().json(ApiResponse {
                        success: true,
                        message: Some("User registered successfully".to_string()),
                        data: Some(user),
                    }))
                },
                Err(e) => {
                    error!("Failed to create user: {}", e);
                    Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                        success: false,
                        message: Some("Failed to create user".to_string()),
                        data: None,
                    }))
                }
            }
        },
        Err(e) => {
            error!("Database error: {}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                message: Some("Database error".to_string()),
                data: None,
            }))
        }
    }
}

/// Получение профиля пользователя
pub async fn user_profile(req: HttpRequest, db: web::Data<Database>) -> Result<HttpResponse> {
    if let Some(user) = check_auth(&req, &db) {
        Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: None,
            data: Some(user),
        }))
    } else {
        Ok(HttpResponse::Unauthorized().json(ApiResponse::<()> {
            success: false,
            message: Some("Unauthorized".to_string()),
            data: None,
        }))
    }
}

/// Получение списка репозиториев
pub async fn list_repos(
    req: HttpRequest, 
    db: web::Data<Database>
) -> Result<HttpResponse> {
    if let Some(user) = user::check_auth(&req, &db) {
        let conn = db.get_connection();
        match Repository::find_by_owner(user.id.unwrap(), conn) {
            Ok(repos) => {
                Ok(HttpResponse::Ok().json(ApiResponse {
                    success: true,
                    message: None,
                    data: Some(repos),
                }))
            },
            Err(e) => {
                error!("Failed to fetch repositories: {}", e);
                Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                    success: false,
                    message: Some("Failed to fetch repositories".to_string()),
                    data: None,
                }))
            }
        }
    } else {
        Ok(HttpResponse::Unauthorized().json(ApiResponse::<()> {
            success: false,
            message: Some("Unauthorized".to_string()),
            data: None,
        }))
    }
}

/// Создание нового репозитория
pub async fn create_repo(
    req: HttpRequest,
    repo_req: web::Json<CreateRepoRequest>,
    db: web::Data<Database>
) -> Result<HttpResponse> {
    if let Some(user) = user::check_auth(&req, &db) {
        let conn = db.get_connection();
        
        // Создаем репозиторий в базе данных
        let repo = Repository {
            id: None,
            name: repo_req.name.clone(),
            description: repo_req.description.clone(),
            owner_id: user.id.unwrap(),
            is_public: repo_req.is_public,
            created_at: None,
        };
        
        match repo.create(conn) {
            Ok(_) => {
                // Инициализируем Git репозиторий
                let repo_path = format!("repositories/{}.git", repo_req.name);
                let init_result = Command::new("git")
                    .args(&["init", "--bare", &repo_path])
                    .output();
                
                match init_result {
                    Ok(output) if output.status.success() => {
                        Ok(HttpResponse::Ok().json(ApiResponse {
                            success: true,
                            message: Some("Repository created successfully".to_string()),
                            data: Some(repo),
                        }))
                    },
                    _ => {
                        error!("Failed to initialize git repository");
                        Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                            success: false,
                            message: Some("Failed to initialize git repository".to_string()),
                            data: None,
                        }))
                    }
                }
            },
            Err(e) => {
                error!("Failed to create repository: {}", e);
                Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                    success: false,
                    message: Some("Failed to create repository".to_string()),
                    data: None,
                }))
            }
        }
    } else {
        Ok(HttpResponse::Unauthorized().json(ApiResponse::<()> {
            success: false,
            message: Some("Unauthorized".to_string()),
            data: None,
        }))
    }
}

/// Получение информации о репозитории
pub async fn get_repo(
    _req: HttpRequest,
    path: web::Path<String>,
    db: web::Data<Database>
) -> Result<HttpResponse> {
    let repo_name = path.into_inner();
    let conn = db.get_connection();
    
    match Repository::find_by_name(&repo_name, conn) {
        Ok(Some(repo)) => {
            // Получаем ветки репозитория
            let repo_path = format!("repositories/{}.git/", repo_name);
            
            let branches_output = Command::new("git")
                .args(&["--git-dir", &repo_path, "branch", "--format=%(refname:short)"])
                .output();

            let branches = match branches_output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    stdout.lines().map(|s| s.to_string()).collect::<Vec<String>>()
                },
                _ => Vec::new(),
            };

            #[derive(Serialize)]
            struct RepoDetails {
                repo: Repository,
                branches: Vec<String>,
            }
            
            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: None,
                data: Some(RepoDetails {
                    repo,
                    branches,
                }),
            }))
        },
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
                success: false,
                message: Some("Repository not found".to_string()),
                data: None,
            }))
        },
        Err(e) => {
            error!("Database error: {}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                message: Some("Database error".to_string()),
                data: None,
            }))
        }
    }
}