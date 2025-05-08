use actix_web::{web, HttpResponse, HttpRequest, Result};
use crate::models::db::Database;
use crate::models::user::User;
use crate::models::repository::Repository;
use crate::models::notification::Notification;
use crate::models::pull_request::{PullRequest, PullRequestComment, PullRequestStatus};
use log::{debug, error};
use serde::{Serialize, Deserialize};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::process::Command;

// Структуры запросов и ответов
#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
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
pub async fn list_repos(req: HttpRequest, db: web::Data<Database>) -> Result<HttpResponse> {
    if let Some(user) = check_auth(&req, &db) {
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
    if let Some(user) = check_auth(&req, &db) {
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
    req: HttpRequest,
    path: web::Path<String>,
    db: web::Data<Database>
) -> Result<HttpResponse> {
    let repo_name = path.into_inner();
    let conn = db.get_connection();
    
    match Repository::find_by_name(&repo_name, conn.clone()) {
        Ok(Some(repo)) => {
            // Получаем ветки репозитория
            let repo_path = format!("repositories/{}.git", repo_name);
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
            
            // Получаем пул-реквесты для репозитория
            let pull_requests = match PullRequest::find_by_repository(repo.id.unwrap(), conn) {
                Ok(prs) => prs,
                Err(e) => {
                    error!("Failed to fetch pull requests: {}", e);
                    Vec::new()
                }
            };
            
            #[derive(Serialize)]
            struct RepoDetails {
                repo: Repository,
                branches: Vec<String>,
                pull_requests: Vec<PullRequest>,
            }
            
            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: None,
                data: Some(RepoDetails {
                    repo,
                    branches,
                    pull_requests,
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

// Структуры запросов для пул-реквестов
#[derive(Serialize, Deserialize)]
pub struct CreatePullRequestRequest {
    pub title: String,
    pub description: Option<String>,
    pub source_branch: String,
    pub target_branch: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateCommentRequest {
    pub content: String,
}

#[derive(Serialize, Deserialize)]
pub struct UpdatePullRequestStatusRequest {
    pub status: String,
}

/// Создание нового пул-реквеста
pub async fn create_pull_request(
    req: HttpRequest,
    path: web::Path<String>,
    pr_req: web::Json<CreatePullRequestRequest>,
    db: web::Data<Database>
) -> Result<HttpResponse> {
    if let Some(user) = check_auth(&req, &db) {
        let repo_name = path.into_inner();
        let conn = db.get_connection();
        
        // Находим репозиторий по имени
        match Repository::find_by_name(&repo_name, conn.clone()) {
            Ok(Some(repo)) => {
                // Создаем пул-реквест
                let pull_request = PullRequest {
                    id: None,
                    title: pr_req.title.clone(),
                    description: pr_req.description.clone(),
                    repository_id: repo.id.unwrap(),
                    source_branch: pr_req.source_branch.clone(),
                    target_branch: pr_req.target_branch.clone(),
                    author_id: user.id.unwrap(),
                    status: PullRequestStatus::Open,
                    created_at: None,
                    updated_at: None,
                };
                
                match pull_request.create(conn) {
                    Ok(_) => {
                        Ok(HttpResponse::Ok().json(ApiResponse {
                            success: true,
                            message: Some("Pull request created successfully".to_string()),
                            data: Some(pull_request),
                        }))
                    },
                    Err(e) => {
                        error!("Failed to create pull request: {}", e);
                        Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                            success: false,
                            message: Some("Failed to create pull request".to_string()),
                            data: None,
                        }))
                    }
                }
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
    } else {
        Ok(HttpResponse::Unauthorized().json(ApiResponse::<()> {
            success: false,
            message: Some("Unauthorized".to_string()),
            data: None,
        }))
    }
}

/// Получение информации о пул-реквесте
pub async fn get_pull_request(
    req: HttpRequest,
    path: web::Path<(String, i64)>,
    db: web::Data<Database>
) -> Result<HttpResponse> {
    if let Some(_) = check_auth(&req, &db) {
        let (repo_name, pr_id) = path.into_inner();
        let conn = db.get_connection();
        
        // Находим репозиторий по имени
        match Repository::find_by_name(&repo_name, conn.clone()) {
            Ok(Some(_)) => {
                // Находим пул-реквест по ID
                match PullRequest::find_by_id(pr_id, conn.clone()) {
                    Ok(Some(pr)) => {
                        // Получаем комментарии к пул-реквесту
                        let comments = match PullRequestComment::find_by_pull_request(pr_id, conn) {
                            Ok(comments) => comments,
                            Err(e) => {
                                error!("Failed to fetch comments: {}", e);
                                Vec::new()
                            }
                        };
                        
                        #[derive(Serialize)]
                        struct PullRequestDetails {
                            pull_request: PullRequest,
                            comments: Vec<PullRequestComment>,
                        }
                        
                        Ok(HttpResponse::Ok().json(ApiResponse {
                            success: true,
                            message: None,
                            data: Some(PullRequestDetails {
                                pull_request: pr,
                                comments,
                            }),
                        }))
                    },
                    Ok(None) => {
                        Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
                            success: false,
                            message: Some("Pull request not found".to_string()),
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
    } else {
        Ok(HttpResponse::Unauthorized().json(ApiResponse::<()> {
            success: false,
            message: Some("Unauthorized".to_string()),
            data: None,
        }))
    }
}

/// Добавление комментария к пул-реквесту
pub async fn add_comment_to_pull_request(
    req: HttpRequest,
    path: web::Path<(String, i64)>,
    comment_req: web::Json<CreateCommentRequest>,
    db: web::Data<Database>
) -> Result<HttpResponse> {
    if let Some(user) = check_auth(&req, &db) {
        let (repo_name, pr_id) = path.into_inner();
        let conn = db.get_connection();
        
        // Находим репозиторий по имени
        match Repository::find_by_name(&repo_name, conn.clone()) {
            Ok(Some(_)) => {
                // Находим пул-реквест по ID
                match PullRequest::find_by_id(pr_id, conn.clone()) {
                    Ok(Some(_)) => {
                        // Создаем комментарий
                        let comment = PullRequestComment {
                            id: None,
                            pull_request_id: pr_id,
                            author_id: user.id.unwrap(),
                            content: comment_req.content.clone(),
                            created_at: None,
                        };
                        
                        match comment.create(conn) {
                            Ok(_) => {
                                Ok(HttpResponse::Ok().json(ApiResponse {
                                    success: true,
                                    message: Some("Comment added successfully".to_string()),
                                    data: Some(comment),
                                }))
                            },
                            Err(e) => {
                                error!("Failed to create comment: {}", e);
                                Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                                    success: false,
                                    message: Some("Failed to create comment".to_string()),
                                    data: None,
                                }))
                            }
                        }
                    },
                    Ok(None) => {
                        Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
                            success: false,
                            message: Some("Pull request not found".to_string()),
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
    } else {
        Ok(HttpResponse::Unauthorized().json(ApiResponse::<()> {
            success: false,
            message: Some("Unauthorized".to_string()),
            data: None,
        }))
    }
}

/// Обновление статуса пул-реквеста
pub async fn update_pull_request_status(
    req: HttpRequest,
    path: web::Path<(String, i64)>,
    status_req: web::Json<UpdatePullRequestStatusRequest>,
    db: web::Data<Database>
) -> Result<HttpResponse> {
    if let Some(user) = check_auth(&req, &db) {
        let (repo_name, pr_id) = path.into_inner();
        let conn = db.get_connection();
        
        // Находим репозиторий по имени
        match Repository::find_by_name(&repo_name, conn.clone()) {
            Ok(Some(repo)) => {
                // Проверяем, что пользователь является владельцем репозитория
                if repo.owner_id != user.id.unwrap() {
                    return Ok(HttpResponse::Forbidden().json(ApiResponse::<()> {
                        success: false,
                        message: Some("Only repository owner can update pull request status".to_string()),
                        data: None,
                    }));
                }
                
                // Находим пул-реквест по ID
                match PullRequest::find_by_id(pr_id, conn.clone()) {
                    Ok(Some(_)) => {
                        let status = PullRequestStatus::from_str(&status_req.status);
                        
                        // Если статус "merged", выполняем слияние веток
                        if status == PullRequestStatus::Merged {
                            match PullRequest::merge(pr_id, conn.clone()) {
                                Ok(_) => {
                                    Ok(HttpResponse::Ok().json(ApiResponse::<()> {
                                        success: true,
                                        message: Some("Pull request merged successfully".to_string()),
                                        data: None,
                                    }))
                                },
                                Err(e) => {
                                    error!("Failed to merge pull request: {}", e);
                                    Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                                        success: false,
                                        message: Some("Failed to merge pull request".to_string()),
                                        data: None,
                                    }))
                                }
                            }
                        } else {
                            // Просто обновляем статус
                            match PullRequest::update_status(pr_id, status, conn) {
                                Ok(_) => {
                                    Ok(HttpResponse::Ok().json(ApiResponse::<()> {
                                        success: true,
                                        message: Some("Pull request status updated successfully".to_string()),
                                        data: None,
                                    }))
                                },
                                Err(e) => {
                                    error!("Failed to update pull request status: {}", e);
                                    Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                                        success: false,
                                        message: Some("Failed to update pull request status".to_string()),
                                        data: None,
                                    }))
                                }
                            }
                        }
                    },
                    Ok(None) => {
                        Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
                            success: false,
                            message: Some("Pull request not found".to_string()),
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
    } else {
        Ok(HttpResponse::Unauthorized().json(ApiResponse::<()> {
            success: false,
            message: Some("Unauthorized".to_string()),
            data: None,
        }))
    }
}

/// Получение уведомлений пользователя
pub async fn get_notifications(
    req: HttpRequest,
    db: web::Data<Database>
) -> Result<HttpResponse> {
    if let Some(user) = check_auth(&req, &db) {
        let conn = db.get_connection();
        
        match Notification::find_by_user_id(user.id.unwrap(), conn) {
            Ok(notifications) => {
                Ok(HttpResponse::Ok().json(ApiResponse {
                    success: true,
                    message: None,
                    data: Some(notifications),
                }))
            },
            Err(e) => {
                error!("Failed to fetch notifications: {}", e);
                Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                    success: false,
                    message: Some("Failed to fetch notifications".to_string()),
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

/// Отметка уведомления как прочитанного
pub async fn mark_notification_as_read(
    req: HttpRequest,
    path: web::Path<i64>,
    db: web::Data<Database>
) -> Result<HttpResponse> {
    if let Some(_) = check_auth(&req, &db) {
        let notification_id = path.into_inner();
        let conn = db.get_connection();
        
        match Notification::mark_as_read(notification_id, conn) {
            Ok(_) => {
                Ok(HttpResponse::Ok().json(ApiResponse::<()> {
                    success: true,
                    message: Some("Notification marked as read".to_string()),
                    data: None,
                }))
            },
            Err(e) => {
                error!("Failed to mark notification as read: {}", e);
                Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                    success: false,
                    message: Some("Failed to mark notification as read".to_string()),
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
