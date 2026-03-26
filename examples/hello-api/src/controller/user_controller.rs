use synfony::axum::extract::Path;
use synfony::axum::http::StatusCode;
use synfony::axum::response::IntoResponse;
use synfony::di::Inject;
use synfony::prelude::*;
use synfony::security::CurrentUser;
use synfony_event::EventDispatcher;
use synfony_messenger::MessageBus;
use synfony_orm::Repository;
use synfony_serializer::GroupedJson;
use synfony_validation::JsonBody;

use crate::dto::{CreateUserDto, UserDto};
use crate::event::{UserCreatedEvent, UserDeletedEvent};
use crate::message::{NotifyAdminsOfNewUser, SendWelcomeEmail};
use crate::repository::UserRepository;

pub struct UserController;

#[controller("/api/users")]
impl UserController {
    #[route(GET, "/", name = "user_list")]
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

    #[route(GET, "/{id}", name = "user_show")]
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

    #[route(POST, "/", name = "user_create")]
    async fn create(
        repo: Inject<UserRepository>,
        url_gen: Inject<UrlGenerator>,
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

        let location = url_gen
            .url("user_show", &[("id", &user.id.to_string())])
            .map_err(|e| ApiError::internal(e.to_string()))?;

        dispatcher
            .dispatch(UserCreatedEvent {
                user_id: user.id,
                email: user.email.clone(),
            })
            .await;

        bus.dispatch_async(SendWelcomeEmail {
            user_id: user.id,
            email: user.email.clone(),
        });
        bus.dispatch_async(NotifyAdminsOfNewUser {
            user_id: user.id,
            user_name: user.name.clone(),
        });

        let response = UserDto::from_model(&user);
        Ok((
            StatusCode::CREATED,
            [("Location", location)],
            GroupedJson::value(response.serialize_group("detail").unwrap()),
        ))
    }

    #[route(DELETE, "/{id}", name = "user_delete")]
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
