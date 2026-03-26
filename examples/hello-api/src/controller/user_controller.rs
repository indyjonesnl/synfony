use synfony::axum::extract::Path;
use synfony::axum::http::StatusCode;
use synfony::axum::response::IntoResponse;
use synfony::axum::routing::get;
use synfony::axum::Router;
use synfony::di::Inject;
use synfony::security::CurrentUser;
use synfony::{ApiError, AppState};
use synfony_event::EventDispatcher;
use synfony_messenger::MessageBus;
use synfony_orm::Repository;
use synfony_serializer::GroupedJson;
use synfony_validation::JsonBody;

use crate::dto::{CreateUserDto, UserDto};
use crate::event::{UserCreatedEvent, UserDeletedEvent};
use crate::message::{NotifyAdminsOfNewUser, SendWelcomeEmail};
use crate::repository::UserRepository;

/// User API controller.
///
/// Demonstrates all Synfony features:
/// - `Inject<T>` for DI (repository, event dispatcher, message bus)
/// - `JsonBody<T>` for auto-validating request payloads
/// - `GroupedJson` for serialization groups
/// - `CurrentUser` for authenticated user context
/// - `EventDispatcher` for dispatching domain events
/// - `MessageBus` for async background jobs
pub struct UserController;

impl UserController {
    pub fn routes() -> Router<AppState> {
        Router::new()
            .route("/api/users", get(Self::list).post(Self::create))
            .route("/api/users/{id}", get(Self::show).delete(Self::remove))
    }

    /// GET /api/users — List all users (serialization group: "list")
    async fn list(
        repo: Inject<UserRepository>,
    ) -> Result<GroupedJson, ApiError> {
        let users = repo.find_all().await.map_err(|e| ApiError::internal(e.to_string()))?;
        let dtos: Vec<UserDto> = users.iter().map(UserDto::from_model).collect();
        let values: Vec<_> = dtos
            .iter()
            .map(|u| u.serialize_group("list").unwrap())
            .collect();
        Ok(GroupedJson::array(values))
    }

    /// GET /api/users/:id — Show a single user
    async fn show(
        Path(id): Path<i32>,
        user: CurrentUser,
        repo: Inject<UserRepository>,
    ) -> Result<GroupedJson, ApiError> {
        let model = repo
            .find_by_id(id)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?
            .ok_or_else(|| ApiError::not_found(format!("User with id {id} not found")))?;

        let dto = UserDto::from_model(&model);
        let group = if user.has_role("ROLE_ADMIN") { "admin" } else { "detail" };
        Ok(GroupedJson::value(dto.serialize_group(group).unwrap()))
    }

    /// POST /api/users — Create a new user
    ///
    /// After creation:
    /// 1. Dispatches `UserCreatedEvent` (sync event listeners fire immediately)
    /// 2. Dispatches `SendWelcomeEmail` message (async, via tokio::spawn)
    /// 3. Dispatches `NotifyAdminsOfNewUser` message (async)
    async fn create(
        repo: Inject<UserRepository>,
        dispatcher: Inject<EventDispatcher>,
        bus: Inject<MessageBus>,
        payload: JsonBody<CreateUserDto>,
    ) -> Result<impl IntoResponse, ApiError> {
        let dto = payload.into_inner();

        if repo.find_by_email(&dto.email).await.map_err(|e| ApiError::internal(e.to_string()))?.is_some() {
            return Err(ApiError::conflict(format!("Email {} is already taken", dto.email)));
        }

        let user = repo
            .create(&dto)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?;

        // Dispatch domain event (sync — listeners fire before response)
        dispatcher
            .dispatch(UserCreatedEvent {
                user_id: user.id,
                email: user.email.clone(),
            })
            .await;

        // Dispatch async messages (fire-and-forget background tasks)
        bus.dispatch_async(SendWelcomeEmail {
            user_id: user.id,
            email: user.email.clone(),
        });
        bus.dispatch_async(NotifyAdminsOfNewUser {
            user_id: user.id,
            user_name: user.name.clone(),
        });

        let response = UserDto::from_model(&user);
        Ok((StatusCode::CREATED, GroupedJson::value(response.serialize_group("detail").unwrap())))
    }

    /// DELETE /api/users/:id — Delete a user
    async fn remove(
        Path(id): Path<i32>,
        repo: Inject<UserRepository>,
        dispatcher: Inject<EventDispatcher>,
    ) -> Result<StatusCode, ApiError> {
        let deleted = repo
            .delete_by_id(id)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?;

        if deleted {
            dispatcher.dispatch(UserDeletedEvent { user_id: id }).await;
            Ok(StatusCode::NO_CONTENT)
        } else {
            Err(ApiError::not_found(format!("User with id {id} not found")))
        }
    }
}
