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

// Структура для автоматического парсинга query-параметров
#[derive(Debug, Deserialize)]
pub struct RepoQuery {
    branch: Option<String>,  // Опциональный параметр
}

#[derive(Debug, Serialize)]
struct GitFile {
    name: String,
    #[serde(rename = "type")]
    type_: String,  // "blob" (файл), "tree" (директория)
    last_branch: String,// кеш вид ветки
    size: Option<u64>,  // Размер файла (если есть)
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

// pub async fn get_files_in_repo(
//     _req: HttpRequest,
//     path: web::Path<String>,
//     db: web::Data<Database>
// ) -> Result<HttpResponse> {
//     let repo_name = path.into_inner();
//     let conn = db.get_connection();

//     match Repository::match Repository::find_by_name(&repo_name, conn) {
//         Ok(Some(repo)) => {
//             let repo_path = format!()
//         }
//     }
// }

/// Получение информации о репозитории
pub async fn get_repo(
    _req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<RepoQuery>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    let repo_name = path.into_inner();
    let branch = query.branch.as_deref().unwrap_or("main");
    let conn = db.get_connection();

    match Repository::find_by_name(&repo_name, conn) {
        Ok(Some(repo)) => {
            // Получаем ветки репозитория
            let repo_path = format!("repositories/{}.git/", repo_name);
            
            let branches_output = Command::new("git")
                .args(&["--git-dir", &repo_path, "branch", "--format=%(refname:short)"])
                .output();

            let files_output = Command::new("git")
                .args(&["--git-dir", &repo_path, "ls-tree", "-l", branch])
                .output();

            let branches = match branches_output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    stdout.lines().map(|s| s.to_string()).collect::<Vec<String>>()
                },
                _ => Vec::new(),
            };

            let files = match files_output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.lines()
                    .filter_map(|line| {
                        let mut parts = line.split_whitespace();
                        let _mode = parts.next()?;
                        let type_ = parts.next()?.to_string();
                        let branch = parts.next()?.to_string();
                        let size = parts.next()?.parse().ok();
                        let name = parts.collect::<Vec<&str>>().join(" ");
                    
                        Some(GitFile {
                            name,
                            type_: type_.clone(),
                            last_branch: branch,
                            size: if type_ == "blob" { size } else { None },
                        })
                    })
                    .collect()
            },
            _ => Vec::new(),
        };
            #[derive(Serialize)]
            struct RepoDetails {
                repo: Repository,
                branches: Vec<String>,
                files: Vec<GitFile>,
            }
            
            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: None,
                data: Some(RepoDetails {
                    repo,
                    branches,
                    files,
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
