use actix_web::{web, HttpResponse, HttpRequest, Result};
use crate::models::db::Database;
use crate::models::project::{Project, ProjectConfig};
use crate::models::repository::Repository;
use crate::models::user::User;
use log::{error};
use serde::{Serialize, Deserialize};
use super::user::{self, ApiResponse};

// Структуры запросов
#[derive(Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
}

#[derive(Serialize, Deserialize)]
pub struct CreateRepoInProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateProjectConfigRequest {
    pub config: ProjectConfig,
}

// Структуры ответов
#[derive(Serialize)]
pub struct ProjectWithRepos {
    pub project: Project,
    pub repositories: Vec<Repository>,
    pub config: ProjectConfig,
}

#[derive(Serialize)]
pub struct ProjectDetails {
    pub project: Project,
    pub repositories: Vec<Repository>,
    pub config: ProjectConfig,
    pub owner: User,
}

/// Получение списка проектов пользователя
pub async fn list_projects(
    req: HttpRequest, 
    db: web::Data<Database>
) -> Result<HttpResponse> {
    if let Some(user) = user::check_auth(&req, &db) {
        let conn = db.get_connection();
        match Project::find_by_owner(user.id.unwrap(), conn) {
            Ok(projects) => {
                Ok(HttpResponse::Ok().json(ApiResponse {
                    success: true,
                    message: None,
                    data: Some(projects),
                }))
            },
            Err(e) => {
                error!("Failed to fetch projects: {}", e);
                Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                    success: false,
                    message: Some("Failed to fetch projects".to_string()),
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

/// Получение списка всех публичных проектов
pub async fn list_public_projects(
    _req: HttpRequest, 
    db: web::Data<Database>
) -> Result<HttpResponse> {
    let conn = db.get_connection();
    match Project::find_public(conn) {
        Ok(projects) => {
            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: None,
                data: Some(projects),
            }))
        },
        Err(e) => {
            error!("Failed to fetch public projects: {}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                message: Some("Failed to fetch public projects".to_string()),
                data: None,
            }))
        }
    }
}

/// Создание нового проекта
pub async fn create_project(
    req: HttpRequest,
    project_req: web::Json<CreateProjectRequest>,
    db: web::Data<Database>
) -> Result<HttpResponse> {
    if let Some(user) = user::check_auth(&req, &db) {
        let conn = db.get_connection();
        
        // Проверяем, что проект с таким именем у пользователя не существует
        match Project::find_by_name_and_owner(&project_req.name, user.id.unwrap(), conn.clone()) {
            Ok(Some(_)) => {
                Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
                    success: false,
                    message: Some("Project with this name already exists".to_string()),
                    data: None,
                }))
            },
            Ok(None) => {
                // Создаем новый проект
                let project = Project {
                    id: None,
                    name: project_req.name.clone(),
                    owner_id: user.id.unwrap(),
                    description: project_req.description.clone(),
                    is_public: project_req.is_public,
                    created_at: None,
                };
                
                match project.create(conn) {
                    Ok(_) => {
                        Ok(HttpResponse::Ok().json(ApiResponse {
                            success: true,
                            message: Some("Project created successfully".to_string()),
                            data: Some(project),
                        }))
                    },
                    Err(e) => {
                        error!("Failed to create project: {}", e);
                        Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                            success: false,
                            message: Some("Failed to create project".to_string()),
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
    } else {
        Ok(HttpResponse::Unauthorized().json(ApiResponse::<()> {
            success: false,
            message: Some("Unauthorized".to_string()),
            data: None,
        }))
    }
}

/// Получение информации о проекте
pub async fn get_project(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    let (username, project_name) = path.into_inner();
    let conn = db.get_connection();

    // Находим пользователя по имени
    let owner = match User::find_by_username(&username, conn.clone()) {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
                success: false,
                message: Some("User not found".to_string()),
                data: None,
            }));
        },
        Err(e) => {
            error!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                message: Some("Database error".to_string()),
                data: None,
            }));
        }
    };

    // Находим проект
    match Project::find_by_name_and_owner(&project_name, owner.id.unwrap(), conn.clone()) {
        Ok(Some(project)) => {
            // Проверяем права доступа
            let current_user = user::check_auth(&req, &db);
            let can_access = project.is_public || 
                current_user.as_ref().map(|u| u.id.unwrap()) == Some(project.owner_id);

            if !can_access {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::<()> {
                    success: false,
                    message: Some("Access denied".to_string()),
                    data: None,
                }));
            }

            // Получаем репозитории проекта
            let repositories = Repository::find_by_project(project.id.unwrap(), conn)
                .unwrap_or_else(|_| Vec::new());

            // Загружаем конфигурацию проекта
            let config = project.load_config().unwrap_or_default();

            let project_details = ProjectDetails {
                project,
                repositories,
                config,
                owner,
            };

            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: None,
                data: Some(project_details),
            }))
        },
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
                success: false,
                message: Some("Project not found".to_string()),
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

/// Создание репозитория в проекте
pub async fn create_repo_in_project(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    repo_req: web::Json<CreateRepoInProjectRequest>,
    db: web::Data<Database>
) -> Result<HttpResponse> {
    let (username, project_name) = path.into_inner();
    
    if let Some(user) = user::check_auth(&req, &db) {
        let conn = db.get_connection();

        // Находим владельца проекта
        let owner = match User::find_by_username(&username, conn.clone()) {
            Ok(Some(user)) => user,
            Ok(None) => {
                return Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
                    success: false,
                    message: Some("User not found".to_string()),
                    data: None,
                }));
            },
            Err(e) => {
                error!("Database error: {}", e);
                return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                    success: false,
                    message: Some("Database error".to_string()),
                    data: None,
                }));
            }
        };

        // Находим проект
        let project = match Project::find_by_name_and_owner(&project_name, owner.id.unwrap(), conn.clone()) {
            Ok(Some(project)) => project,
            Ok(None) => {
                return Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
                    success: false,
                    message: Some("Project not found".to_string()),
                    data: None,
                }));
            },
            Err(e) => {
                error!("Database error: {}", e);
                return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                    success: false,
                    message: Some("Database error".to_string()),
                    data: None,
                }));
            }
        };

        // Проверяем права доступа (только владелец может создавать репозитории)
        if user.id.unwrap() != project.owner_id {
            return Ok(HttpResponse::Forbidden().json(ApiResponse::<()> {
                success: false,
                message: Some("Only project owner can create repositories".to_string()),
                data: None,
            }));
        }

        // Проверяем, что репозиторий с таким именем в проекте не существует
        match Repository::find_by_name_and_project(&repo_req.name, project.id.unwrap(), conn.clone()) {
            Ok(Some(_)) => {
                Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
                    success: false,
                    message: Some("Repository with this name already exists in project".to_string()),
                    data: None,
                }))
            },
            Ok(None) => {
                // Создаем новый репозиторий
                let repo = Repository {
                    id: None,
                    name: repo_req.name.clone(),
                    project_id: project.id.unwrap(),
                    owner_id: user.id.unwrap(),
                    description: repo_req.description.clone(),
                    is_public: repo_req.is_public,
                    created_at: None,
                };
                
                match repo.create(conn) {
                    Ok(_) => {
                        Ok(HttpResponse::Ok().json(ApiResponse {
                            success: true,
                            message: Some("Repository created successfully".to_string()),
                            data: Some(repo),
                        }))
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

/// Обновление конфигурации проекта
pub async fn update_project_config(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    config_req: web::Json<UpdateProjectConfigRequest>,
    db: web::Data<Database>
) -> Result<HttpResponse> {
    let (username, project_name) = path.into_inner();
    
    if let Some(user) = user::check_auth(&req, &db) {
        let conn = db.get_connection();

        // Находим владельца проекта
        let owner = match User::find_by_username(&username, conn.clone()) {
            Ok(Some(user)) => user,
            Ok(None) => {
                return Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
                    success: false,
                    message: Some("User not found".to_string()),
                    data: None,
                }));
            },
            Err(e) => {
                error!("Database error: {}", e);
                return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                    success: false,
                    message: Some("Database error".to_string()),
                    data: None,
                }));
            }
        };

        // Находим проект
        let project = match Project::find_by_name_and_owner(&project_name, owner.id.unwrap(), conn) {
            Ok(Some(project)) => project,
            Ok(None) => {
                return Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
                    success: false,
                    message: Some("Project not found".to_string()),
                    data: None,
                }));
            },
            Err(e) => {
                error!("Database error: {}", e);
                return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                    success: false,
                    message: Some("Database error".to_string()),
                    data: None,
                }));
            }
        };

        // Проверяем права доступа (только владелец может изменять конфигурацию)
        if user.id.unwrap() != project.owner_id {
            return Ok(HttpResponse::Forbidden().json(ApiResponse::<()> {
                success: false,
                message: Some("Only project owner can update configuration".to_string()),
                data: None,
            }));
        }

        // Сохраняем конфигурацию
        match project.save_config(&config_req.config) {
            Ok(_) => {
                Ok(HttpResponse::Ok().json(ApiResponse {
                    success: true,
                    message: Some("Project configuration updated successfully".to_string()),
                    data: Some(&config_req.config),
                }))
            },
            Err(e) => {
                error!("Failed to save project configuration: {}", e);
                Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                    success: false,
                    message: Some("Failed to save project configuration".to_string()),
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

/// Получение конфигурации проекта
pub async fn get_project_config(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    let (username, project_name) = path.into_inner();
    let conn = db.get_connection();

    // Находим пользователя по имени
    let owner = match User::find_by_username(&username, conn.clone()) {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
                success: false,
                message: Some("User not found".to_string()),
                data: None,
            }));
        },
        Err(e) => {
            error!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                message: Some("Database error".to_string()),
                data: None,
            }));
        }
    };

    // Находим проект
    match Project::find_by_name_and_owner(&project_name, owner.id.unwrap(), conn) {
        Ok(Some(project)) => {
            // Проверяем права доступа
            let current_user = user::check_auth(&req, &db);
            let can_access = project.is_public || 
                current_user.as_ref().map(|u| u.id.unwrap()) == Some(project.owner_id);

            if !can_access {
                return Ok(HttpResponse::Forbidden().json(ApiResponse::<()> {
                    success: false,
                    message: Some("Access denied".to_string()),
                    data: None,
                }));
            }

            // Загружаем конфигурацию проекта
            let config = project.load_config().unwrap_or_default();

            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: None,
                data: Some(config),
            }))
        },
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
                success: false,
                message: Some("Project not found".to_string()),
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
