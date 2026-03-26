# Synfony

**A Symfony-inspired web framework for Rust.**

Synfony brings the familiar patterns and conventions of PHP's [Symfony](https://symfony.com) framework to the Rust ecosystem. If you've built applications with Symfony and want Rust's performance without learning an entirely new way of thinking about web applications, Synfony is for you.

```rust
use synfony::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Application::new()?;
    app.register_routes(UserController::routes());
    app.run().await
}

struct UserController;

impl UserController {
    fn routes() -> Router<AppState> {
        Router::new()
            .route("/api/users", get(Self::list))
            .route("/api/users/{id}", get(Self::show))
    }

    async fn list(repo: Inject<UserRepository>) -> Result<Json<Vec<User>>, ApiError> {
        let users = repo.find_all().await.map_err(|e| ApiError::internal(e.to_string()))?;
        Ok(Json(users))
    }

    async fn show(Path(id): Path<i32>, repo: Inject<UserRepository>) -> Result<Json<User>, ApiError> {
        repo.find_by_id(id)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?
            .map(Json)
            .ok_or_else(|| ApiError::not_found(format!("User {id} not found")))
    }
}
```

## Why Synfony?

| What you know from Symfony | What Synfony gives you |
|---|---|
| `#[Route('/api/users')]` | Controller routing with `Router::new().route(...)` |
| Autowired constructor injection | `Inject<T>` extractor resolves services from the DI container |
| `security.yaml` firewalls | `FirewallLayer` with pattern matching and authenticators |
| `#[IsGranted('ROLE_ADMIN')]` | `CurrentUser` extractor + voter system |
| `#[MapRequestPayload]` | `JsonBody<T>` auto-deserializes and validates |
| `#[Groups(['list', 'detail'])]` | `#[groups("list", "detail")]` with `SerializeGroups` derive |
| Doctrine EntityRepository | `Repository<E>` trait backed by SeaORM |
| EventDispatcher | `EventDispatcher` with priority-ordered async listeners |
| Messenger component | `MessageBus` with sync and async (tokio::spawn) dispatch |
| `bin/console` | `Application::run()` detects CLI commands vs HTTP serving |
| `.env` cascading | `.env` → `.env.local` → `.env.{APP_ENV}` → `.env.{APP_ENV}.local` |

**What you gain:** 5-50x performance over PHP, compile-time guarantees (DI graph, types, SQL), true async concurrency, memory safety, single-binary deployment.

**What you trade:** Slower feedback loop (compile vs refresh), smaller ecosystem (no EasyAdmin, API Platform equivalents yet), steeper learning curve for Rust newcomers.

---

## Table of Contents

- [Getting Started](#getting-started)
- [Architecture Overview](#architecture-overview)
- [Core Components](#core-components)
  - [Application & Kernel](#application--kernel)
  - [Dependency Injection](#dependency-injection)
  - [Configuration](#configuration)
  - [Controllers & Routing](#controllers--routing)
  - [Error Handling](#error-handling)
  - [Security](#security)
  - [ORM & Repositories](#orm--repositories)
  - [Validation](#validation)
  - [Serialization Groups](#serialization-groups)
  - [Event Dispatcher](#event-dispatcher)
  - [Message Bus](#message-bus)
  - [Console](#console)
- [Complete Example](#complete-example)
- [Design Choices](#design-choices)
- [Known Technical Debt](#known-technical-debt)
- [Roadmap](#roadmap)
- [Project Structure](#project-structure)

---

## Getting Started

### Prerequisites

- Rust 1.80+ (edition 2024)
- Cargo

### Add Synfony to your project

```toml
[dependencies]
synfony = { git = "https://github.com/synfony-framework/synfony" }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

### Minimal application

```rust
use synfony::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Application::new()?;

    app.register_routes(
        Router::new().route("/api/health", axum::routing::get(health))
    );

    app.run().await
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}
```

```sh
# Start the server (default: http://0.0.0.0:8000)
cargo run

# Start on a specific port
cargo run -- serve --port 3000

# CLI commands
cargo run -- debug:router
cargo run -- debug:container
```

### Project structure

Synfony follows Symfony's conventions:

```
my-app/
  Cargo.toml
  .env                          # Default env vars (committed)
  .env.local                    # Local overrides (gitignored)
  config/
    app.yaml                    # Application configuration
    security.yaml               # Firewall & access control (future)
  src/
    main.rs                     # Entry point
    controller/                 # HTTP controllers
    entity/                     # SeaORM entities (like src/Entity/)
    repository/                 # Data access (like src/Repository/)
    service/                    # Business logic
    dto/                        # Request/response DTOs
    event/                      # Domain events
    message/                    # Async messages & handlers
    security/                   # Voters, authenticators
  migrations/                   # Database migrations
  tests/
```

---

## Architecture Overview

Synfony is a **layer on top of [Axum](https://github.com/tokio-rs/axum) + [Tower](https://github.com/tower-rs/tower)**, not a from-scratch web server. It provides Symfony-like conventions and developer ergonomics while leveraging Rust's battle-tested async ecosystem.

```
┌─────────────────────────────────────────────┐
│                  Your App                    │
│  Controllers · Services · Entities · Events  │
├─────────────────────────────────────────────┤
│                  Synfony                     │
│  DI · Security · ORM · Validation · Events   │
├─────────────────────────────────────────────┤
│              Axum + Tower                    │
│  Router · Extractors · Middleware · Layers    │
├─────────────────────────────────────────────┤
│           Hyper + Tokio                      │
│  HTTP/1.1 · HTTP/2 · Async I/O               │
└─────────────────────────────────────────────┘
```

### Crate structure

The framework is split into 13 focused crates, like Symfony's component architecture:

| Crate | Symfony Equivalent | Purpose |
|---|---|---|
| `synfony` | `symfony/framework-bundle` | Main crate — re-exports everything |
| `synfony-core` | HttpKernel | Application lifecycle, kernel, error handling |
| `synfony-macros` | PHP Attributes | `#[controller]`, `#[route]`, `#[service]` proc macros |
| `synfony-di` | DependencyInjection | `Container` + `Inject<T>` extractor |
| `synfony-config` | Config | YAML + `.env` with environment cascading |
| `synfony-console` | Console | `ConsoleIO` (tables, progress bars, prompts) |
| `synfony-security` | Security | JWT, firewalls, voters, `CurrentUser` |
| `synfony-orm` | Doctrine ORM | `Repository<E>`, `EntityManager` (SeaORM) |
| `synfony-validation` | Validator | `JsonBody<T>`, `QueryParams<T>` (auto-validate) |
| `synfony-serializer` | Serializer | `#[groups]` derive + `GroupedJson` response |
| `synfony-event` | EventDispatcher | Typed async event dispatch with priorities |
| `synfony-messenger` | Messenger | Sync + async message bus |

---

## Core Components

### Application & Kernel

The `Application` is the entry point — equivalent to Symfony's Kernel combined with `bin/console`.

```rust
let mut app = Application::new()?;

// Register services in the DI container
app.register_service(Arc::new(my_service));

// Register controller routes
app.register_routes(MyController::routes());

// Run: serves HTTP or executes a CLI command (auto-detected from args)
app.run().await?;
```

The `Kernel` builds the Axum router with middleware:

```rust
let kernel = Kernel::new(config, container)
    .register_routes(UserController::routes())
    .register_routes(AdminController::routes())
    .with_default_middleware()
    .build(); // → axum::Router ready to serve
```

Built-in CLI commands:
- `serve --port 8000` — Start the HTTP server
- `debug:router` — List all registered routes
- `debug:container` — List all registered services

---

### Dependency Injection

`Inject<T>` resolves services from the container — like Symfony's autowired constructor parameters.

```rust
// Register a service
app.register_service(Arc::new(UserRepository::new(db)));

// Use it in any handler via Inject<T>
async fn list_users(repo: Inject<UserRepository>) -> Json<Vec<User>> {
    Json(repo.find_all().await.unwrap())
}

// Multiple injections work naturally
async fn create_user(
    repo: Inject<UserRepository>,
    dispatcher: Inject<EventDispatcher>,
    bus: Inject<MessageBus>,
    payload: JsonBody<CreateUserDto>,
) -> Result<impl IntoResponse, ApiError> {
    let user = repo.create(&payload).await?;
    dispatcher.dispatch(UserCreatedEvent { user_id: user.id }).await;
    bus.dispatch_async(SendWelcomeEmail { email: user.email.clone() });
    Ok((StatusCode::CREATED, Json(user)))
}
```

The container stores services as `Arc<T>` — all services are shared singletons by default.

**Symfony comparison:**

```php
// Symfony: constructor injection via autowiring
class UserController extends AbstractController {
    public function __construct(
        private UserRepository $repo,      // ← autowired
        private EventDispatcherInterface $dispatcher,
    ) {}
}

// Synfony: parameter injection via Inject<T>
async fn create(
    repo: Inject<UserRepository>,          // ← resolved from container
    dispatcher: Inject<EventDispatcher>,
) -> impl IntoResponse { ... }
```

---

### Configuration

Configuration mirrors Symfony's system exactly.

**`.env` cascading** (same order as Symfony):

```
.env                    # Defaults, committed to VCS
.env.local              # Local overrides, gitignored
.env.dev                # Dev-specific defaults
.env.dev.local          # Local dev overrides
```

**YAML config** (`config/app.yaml`):

```yaml
app:
  name: "My API"
  secret: "${APP_SECRET}"           # env var interpolation
  debug: "${APP_DEBUG:false}"       # default values

database:
  url: "${DATABASE_URL}"
  pool_size: "${DB_POOL_SIZE:5}"

jwt:
  secret: "${JWT_SECRET}"
  ttl: 3600
```

**Type-safe access:**

```rust
#[derive(Deserialize)]
struct JwtConfig {
    secret: String,
    ttl: u64,
}

let config = SynfonyConfig::load(".")?;
let jwt: JwtConfig = config.section("jwt")?;
let app_env = config.app_env();       // "dev", "prod", etc.
let debug = config.is_debug();        // true in dev
```

---

### Controllers & Routing

Controllers are plain structs with an `impl` block that returns an Axum `Router`:

```rust
pub struct UserController;

impl UserController {
    pub fn routes() -> Router<AppState> {
        Router::new()
            .route("/api/users", get(Self::list).post(Self::create))
            .route("/api/users/{id}", get(Self::show).delete(Self::remove))
    }

    async fn list(repo: Inject<UserRepository>) -> Json<Vec<UserDto>> { ... }
    async fn show(Path(id): Path<i32>, repo: Inject<UserRepo>) -> Result<Json<UserDto>, ApiError> { ... }
    async fn create(repo: Inject<UserRepo>, payload: JsonBody<CreateDto>) -> impl IntoResponse { ... }
    async fn remove(Path(id): Path<i32>, repo: Inject<UserRepo>) -> Result<StatusCode, ApiError> { ... }
}

// Register in main.rs
app.register_routes(UserController::routes());
```

**Symfony comparison:**

```php
// Symfony                                    // Synfony
#[Route('/api/users')]                        // .route("/api/users", get(...))
class UserController {
    #[Route('/', methods: ['GET'])]           // get(Self::list)
    public function list(): JsonResponse      // async fn list() -> Json<Vec<User>>

    #[Route('/{id}', methods: ['GET'])]       // .route("/api/users/{id}", get(...))
    public function show(int $id): Response   // async fn show(Path(id): Path<i32>) -> ...
}
```

---

### Error Handling

`ApiError` provides structured JSON error responses matching RFC 7807:

```rust
async fn show(Path(id): Path<i32>, repo: Inject<UserRepo>) -> Result<Json<User>, ApiError> {
    repo.find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("User {id} not found")))
}
```

Response:
```json
{
    "type": "https://httpstatuses.com/404",
    "title": "Not Found",
    "status": 404,
    "detail": "User 999 not found"
}
```

Available error constructors:

| Method | HTTP Status |
|---|---|
| `ApiError::bad_request(msg)` | 400 |
| `ApiError::unauthorized(msg)` | 401 |
| `ApiError::forbidden(msg)` | 403 |
| `ApiError::not_found(msg)` | 404 |
| `ApiError::conflict(msg)` | 409 |
| `ApiError::unprocessable(msg, errors)` | 422 |
| `ApiError::internal(msg)` | 500 |

---

### Security

The security component mirrors Symfony's architecture: authenticators, firewalls, access control, and voters.

#### JWT Authentication

```rust
use synfony_security::jwt::{JwtAuthenticator, JwtConfig, JwtManager};

// Configure JWT
let config = JwtConfig::new("your-secret-key").with_ttl(3600);
let authenticator = Arc::new(JwtAuthenticator::new(config.clone()));
let manager = Arc::new(JwtManager::new(config));

// Generate tokens (in your login endpoint)
let token = SecurityToken::new("user-123", "alice@example.com")
    .with_role("ROLE_USER")
    .with_role("ROLE_ADMIN");
let jwt_string = manager.generate(&token)?;

// Register the manager for injection into controllers
app.register_service(manager);
```

#### Firewalls & Access Control

```rust
use synfony_security::firewall::*;

let security_config = SecurityConfig {
    firewalls: HashMap::from([
        ("public".into(), FirewallConfig {
            pattern: "/api/health".into(),
            authenticator: None,
            anonymous: true,
        }),
        ("api".into(), FirewallConfig {
            pattern: "/api/*".into(),
            authenticator: Some("jwt".into()),
            anonymous: false,
        }),
    ]),
    access_control: vec![
        AccessControlEntry { path: "/api/admin/*".into(), roles: vec!["ROLE_ADMIN".into()] },
        AccessControlEntry { path: "/api/*".into(), roles: vec!["ROLE_USER".into()] },
    ],
};

let firewall = FirewallLayer::from_config(security_config, authenticators);

// Apply to routes as Tower middleware
app.register_routes(UserController::routes().layer(firewall.clone()));
app.register_routes(AdminController::routes().layer(firewall));
```

#### CurrentUser Extractor

```rust
// Like Symfony's $this->getUser()
async fn me(user: CurrentUser) -> Json<MeResponse> {
    Json(MeResponse {
        id: user.user_id().to_string(),
        email: user.user_identifier().to_string(),
        roles: user.roles().iter().cloned().collect(),
    })
}

// Optional — returns None for anonymous users
async fn greeting(user: OptionalUser) -> Json<String> {
    match user.0 {
        Some(u) => Json(format!("Hello, {}!", u.user_identifier())),
        None => Json("Hello, guest!".into()),
    }
}
```

#### Voters

```rust
use synfony::prelude::*;
use std::any::Any;

struct PostVoter;

impl Voter for PostVoter {
    fn supports(&self, attribute: &str, subject: &dyn Any) -> bool {
        ["EDIT", "DELETE"].contains(&attribute) && subject.is::<Post>()
    }

    fn vote(&self, token: &SecurityToken, attribute: &str, subject: &dyn Any) -> Vote {
        let post = subject.downcast_ref::<Post>().unwrap();
        match attribute {
            "EDIT" if token.user_id() == &post.author_id.to_string() => Vote::Granted,
            "DELETE" if token.has_role("ROLE_ADMIN") => Vote::Granted,
            _ => Vote::Denied,
        }
    }
}
```

---

### ORM & Repositories

Synfony wraps [SeaORM](https://www.sea-ql.org/SeaORM/) with a Doctrine-inspired repository pattern.

#### Entities

```rust
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    #[sea_orm(unique)]
    pub email: String,
    pub role: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

#### Repository Pattern

```rust
use synfony_orm::{Repository, OrmError};

pub struct UserRepository {
    db: Arc<DatabaseConnection>,
}

// Get built-in methods for free: find_all, find_by_id, find_or_fail, delete_by_id
#[async_trait]
impl Repository<user::Entity> for UserRepository {
    fn connection(&self) -> &DatabaseConnection { &self.db }
}

// Add custom query methods
impl UserRepository {
    pub async fn find_by_email(&self, email: &str) -> Result<Option<user::Model>, OrmError> {
        user::Entity::find()
            .filter(user::Column::Email.eq(email))
            .one(&*self.db)
            .await
            .map_err(OrmError::Database)
    }
}
```

#### EntityManager

```rust
use synfony_orm::{EntityManager, Set};

let em = EntityManager::new(db);

// Create (persist + flush in Doctrine terms)
let user = em.persist(user::ActiveModel {
    name: Set("Alice".into()),
    email: Set("alice@example.com".into()),
    ..Default::default()
}).await?;

// Update
let mut active: user::ActiveModel = user.into();
active.name = Set("Alice Updated".into());
let updated = em.update(active).await?;

// Delete
em.remove::<user::ActiveModel, _>(updated).await?;
```

---

### Validation

`JsonBody<T>` auto-deserializes and validates — like Symfony's `#[MapRequestPayload]`.

```rust
use synfony_validation::JsonBody;
use validator::Validate;

#[derive(Deserialize, Validate)]
struct CreateUserDto {
    #[validate(length(min = 2, max = 100, message = "Name must be 2-100 characters"))]
    name: String,

    #[validate(email(message = "Invalid email address"))]
    email: String,
}

async fn create(payload: JsonBody<CreateUserDto>) -> impl IntoResponse {
    // If we reach here, payload is guaranteed valid
    let dto = payload.into_inner();
    // ...
}
```

Invalid input returns 422 with field-level errors:

```json
{
    "type": "https://httpstatuses.com/422",
    "title": "Validation Failed",
    "status": 422,
    "detail": "The submitted data is invalid.",
    "errors": {
        "email": [{ "code": "email", "message": "Invalid email address" }],
        "name": [{ "code": "length", "message": "Name must be 2-100 characters" }]
    }
}
```

`QueryParams<T>` does the same for query string parameters:

```rust
#[derive(Deserialize, Validate)]
struct SearchParams {
    #[validate(length(min = 1))]
    q: String,
    #[validate(range(min = 1, max = 100))]
    limit: Option<u32>,
}

async fn search(params: QueryParams<SearchParams>) -> Json<Vec<Result>> {
    let search = params.into_inner();
    // ...
}
```

---

### Serialization Groups

Control which fields are included in responses based on context — like Symfony's `#[Groups]`.

```rust
use synfony_serializer::SerializeGroups;

#[derive(Serialize, SerializeGroups)]
struct UserDto {
    #[groups("list", "detail", "admin")]
    id: i32,

    #[groups("list", "detail", "admin")]
    name: String,

    #[groups("detail", "admin")]
    email: String,

    #[groups("admin")]
    role: String,
}
```

Use different groups per endpoint:

```rust
use synfony_serializer::GroupedJson;

// GET /api/users → returns { id, name } only
async fn list(repo: Inject<UserRepo>) -> GroupedJson {
    let users = repo.find_all().await.unwrap();
    let values: Vec<_> = users.iter()
        .map(|u| UserDto::from(u).serialize_group("list").unwrap())
        .collect();
    GroupedJson::array(values)
}

// GET /api/users/:id → returns { id, name, email } for users, + { role } for admins
async fn show(id: Path<i32>, user: CurrentUser, repo: Inject<UserRepo>) -> GroupedJson {
    let dto = UserDto::from(repo.find_or_fail(*id).await.unwrap());
    let group = if user.has_role("ROLE_ADMIN") { "admin" } else { "detail" };
    GroupedJson::value(dto.serialize_group(group).unwrap())
}
```

---

### Event Dispatcher

Typed async event dispatch with priority ordering — like Symfony's EventDispatcher.

```rust
use synfony_event::EventDispatcher;

// Events are plain structs (must implement Clone)
#[derive(Clone)]
struct UserCreatedEvent {
    user_id: i32,
    email: String,
}

// Register listeners with priorities (higher = fires first)
let dispatcher = EventDispatcher::new();

dispatcher.listen::<UserCreatedEvent>(20, |event: UserCreatedEvent| async move {
    tracing::info!("Audit: user {} created", event.user_id);
});

dispatcher.listen::<UserCreatedEvent>(10, |event: UserCreatedEvent| async move {
    tracing::info!("Analytics: track signup for {}", event.email);
});

// Dispatch — listeners fire in priority order (20, then 10)
dispatcher.dispatch(UserCreatedEvent { user_id: 1, email: "alice@example.com".into() }).await;
```

Register the dispatcher as a service for injection into controllers:

```rust
app.register_service(Arc::new(dispatcher));

// In a controller:
async fn create(dispatcher: Inject<EventDispatcher>, ...) -> impl IntoResponse {
    // ... create user ...
    dispatcher.dispatch(UserCreatedEvent { user_id: user.id, email: user.email }).await;
}
```

---

### Message Bus

Async job dispatch — like Symfony Messenger. Messages can be dispatched synchronously (wait for handler) or asynchronously (fire-and-forget via `tokio::spawn`).

```rust
use synfony_messenger::MessageBus;

// Messages are plain structs
struct SendWelcomeEmail { user_id: i32, email: String }
struct NotifyAdmins { user_name: String }

let bus = MessageBus::new();

// Register handlers
bus.register_handler::<SendWelcomeEmail>(|msg: SendWelcomeEmail| async move {
    // Send the email (runs in background)
    mailer::send_welcome(&msg.email).await;
    Ok(())
});

bus.register_handler::<NotifyAdmins>(|msg: NotifyAdmins| async move {
    slack::post(format!("New user: {}", msg.user_name)).await;
    Ok(())
});

app.register_service(Arc::new(bus));

// In a controller:
async fn create(bus: Inject<MessageBus>, ...) -> impl IntoResponse {
    // ... create user ...

    // Sync: wait for handler to complete
    bus.dispatch(SendWelcomeEmail { user_id: 1, email: "a@b.com".into() }).await?;

    // Async: fire and forget (handler runs in tokio task)
    bus.dispatch_async(NotifyAdmins { user_name: "Alice".into() });
}
```

---

### Console

`ConsoleIO` provides Symfony's `SymfonyStyle` output API:

```rust
let io = ConsoleIO::new();

io.title("My Command");
io.section("Processing");
io.info("Loading data...");
io.success("Operation completed!");
io.warning("Some items were skipped");
io.error("Something went wrong");

// Tables
io.table(
    vec!["ID", "Name", "Email"],
    vec![
        vec!["1", "Alice", "alice@example.com"],
        vec!["2", "Bob", "bob@example.com"],
    ],
);

// Progress bars
let pb = io.progress_bar(100);
for i in 0..100 {
    pb.inc(1);
}
pb.finish();

// Interactive prompts
let name = io.ask("What is your name?")?;
let confirmed = io.confirm("Are you sure?", false)?;
let choice = io.choice("Pick one", &["Option A", "Option B", "Option C"])?;
```

---

## Complete Example

The `examples/hello-api` directory contains a fully working API that demonstrates every Synfony feature. Run it with:

```sh
cd examples/hello-api
cargo run

# Or with logging:
RUST_LOG=info cargo run
```

### API endpoints

| Method | Path | Auth | Description |
|---|---|---|---|
| `GET` | `/api/health` | None | Health check |
| `POST` | `/api/login` | None | Get JWT token |
| `GET` | `/api/me` | JWT | Current user info |
| `GET` | `/api/users` | JWT | List users (serialization group: `list`) |
| `GET` | `/api/users/{id}` | JWT | Show user (`detail` or `admin` group) |
| `POST` | `/api/users` | JWT | Create user (validated, dispatches events + messages) |
| `DELETE` | `/api/users/{id}` | JWT | Delete user (dispatches event) |
| `GET` | `/api/admin/dashboard` | JWT + `ROLE_ADMIN` | Admin-only endpoint |

### Test it

```sh
# Login as admin
TOKEN=$(curl -s -X POST http://localhost:8000/api/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"admin"}' | jq -r '.token')

# List users (returns only id + name via "list" group)
curl -s http://localhost:8000/api/users -H "Authorization: Bearer $TOKEN" | jq

# Create user (validates, dispatches events, sends async messages)
curl -s -X POST http://localhost:8000/api/users \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"Charlie","email":"charlie@example.com"}' | jq

# Try invalid data (returns 422 with field errors)
curl -s -X POST http://localhost:8000/api/users \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"X","email":"not-an-email"}' | jq

# Admin dashboard (requires ROLE_ADMIN)
curl -s http://localhost:8000/api/admin/dashboard \
  -H "Authorization: Bearer $TOKEN" | jq
```

---

## Design Choices

### Why Axum as the foundation?

We evaluated Actix-web, Rocket, Poem, and Salvo. Axum won because:
- **Tower ecosystem**: The richest middleware ecosystem in Rust. Every Tower layer works out of the box.
- **Extractor pattern**: Handlers declare what they need as function parameters — a natural fit for Symfony's controller argument resolution.
- **No macros for routing**: Axum's router is pure Rust code, making it composable and debuggable. Our macros layer on top rather than replacing the router.
- **Community momentum**: Maintained by the Tokio team, most popular Rust web framework as of 2025-2026.

### Why compile-time DI instead of runtime reflection?

Rust has no runtime reflection (unlike PHP's `ReflectionClass`). Our DI container uses `TypeId`-keyed `Arc<T>` storage with the `Inject<T>` extractor pattern. This means:
- **Services are resolved at request time** by type, not by name
- **No YAML service definitions** — services are registered in Rust code
- **No autowiring magic** — you explicitly register what you need, but `Inject<T>` makes usage as ergonomic as Symfony's autowiring

### Why SeaORM over Diesel?

- **Async-native**: SeaORM is built on sqlx with first-class async support
- **Active record pattern**: Closest to Doctrine's entity model
- **Code generation**: Can generate entities from an existing database
- **Multiple databases**: SQLite, PostgreSQL, MySQL out of the box

### Why owned events instead of borrowed?

The event dispatcher passes owned (cloned) events to listeners. This avoids lifetime complexity with async closures — Rust's borrow checker doesn't allow `&Event` to live across `.await` points. The trade-off is a clone per listener, which is negligible for typical event payloads.

### Why no `#[controller]` macro on the example app?

The `#[controller]` proc macro exists in `synfony-macros` but the example app uses manual `routes()` methods. This is intentional — the macro generates the same code, but explicit routing is easier to debug and understand during early development. The macro will be refined and documented when the API stabilizes.

---

## Known Technical Debt

### DI Container is runtime, not compile-time
The current container uses `HashMap<TypeId, Box<dyn Any>>` — a runtime service locator. The plan calls for compile-time DI validation via proc macros (like Pavex), but the current implementation panics at runtime if a service isn't registered. This works fine in practice since services are registered in `main()`, but it's not as safe as compile-time checking.

**Impact:** If you forget to `register_service()`, you get a panic on first request, not a compile error.

### `#[controller]` macro parses route attributes via string splitting
The `controller.rs` macro parses `#[route(GET, "/path")]` by splitting on commas and trimming quotes — fragile string manipulation instead of proper syn parsing. This works for simple cases but will break with complex attribute arguments.

**Impact:** Route attributes must follow the exact format `#[route(METHOD, "/path")]`. No support for named routes, middleware annotations, or other Symfony-like attributes yet.

### `#[service]` macro generates `from_container()` but nothing auto-registers
The `#[service]` macro generates a `from_container(&Container) -> Arc<Self>` method, but services still need manual `app.register_service(Arc::new(...))` calls in `main()`. The `inventory`-based auto-discovery is stubbed but not wired up.

**Impact:** Unlike Symfony's autoconfigure, you must manually register every service.

### Firewall pattern matching is simplistic
`path_matches()` uses basic prefix matching (`/api/*` matches anything starting with `/api/`). It doesn't support regex, named parameters, or Symfony's full pattern syntax.

**Impact:** Complex access control patterns like `/api/users/{id}/edit` can't be distinguished from `/api/users/{id}`.

### Firewall ordering depends on HashMap iteration order
Firewalls are stored in a `HashMap<String, FirewallConfig>`, so matching order is non-deterministic. Symfony processes firewalls in declaration order, which matters when patterns overlap.

**Impact:** If you have overlapping patterns (e.g., `/api/login` and `/api/*`), the more specific pattern might not match first. The current workaround is to register public routes without the firewall layer.

### No request-scoped services
All services are singletons (`Arc<T>`). There's no equivalent to Symfony's `service_subscriber` or request-scoped services. Database connections are shared via a connection pool, which works, but per-request state must go through Axum's request extensions.

### Serialization groups use `serde_json::Value` intermediary
The `SerializeGroups` macro serializes each field to a `serde_json::Value` and conditionally includes it. This means every grouped serialization does a full serialize → filter → re-serialize cycle. A more efficient approach would generate group-specific structs at compile time.

### No tests
The framework has no automated tests. All verification was done via end-to-end curl testing against the example app. Unit tests for the DI container, firewall matching, voter system, and event dispatcher should be added.

### `serde_yaml` is deprecated
The workspace uses `serde_yaml = "0.9"` which is deprecated. Should migrate to `serde_yml` or another maintained YAML library.

---

## Roadmap

### Near-term (v0.2)

- [ ] **Test suite** — Unit tests for all crates, integration tests for the example app
- [ ] **Improved `#[controller]` macro** — Proper syn-based parsing, support for route names
- [ ] **`#[is_granted]` macro** — Route-level authorization attribute
- [ ] **`synfony new` CLI tool** — Project scaffolding (like `symfony new`)
- [ ] **`make:controller` / `make:entity`** — Code generators (like MakerBundle)
- [ ] **`debug:router` with metadata** — Show methods, paths, middleware, controller names
- [ ] **Firewall from `security.yaml`** — Load firewall config from YAML instead of Rust code
- [ ] **Fix firewall ordering** — Use `Vec` instead of `HashMap` for deterministic matching
- [ ] **Replace `serde_yaml`** — Migrate to `serde_yml` or `toml` configuration

### Mid-term (v0.3)

- [ ] **Compile-time DI validation** — Proc macro that verifies all `Inject<T>` can be resolved
- [ ] **Service auto-registration** — `#[service]` automatically registers via `inventory`
- [ ] **Kernel events** — `RequestEvent`, `ResponseEvent`, `ExceptionEvent` (like Symfony's kernel events)
- [ ] **Middleware as event subscribers** — CORS, logging, profiling as event listeners
- [ ] **Validation groups** — Like Symfony's validation groups for different contexts
- [ ] **Translatable validation messages** — i18n support for error messages
- [ ] **Messenger transports** — Redis and PostgreSQL-backed persistent queues (via `apalis`)
- [ ] **SeaORM migration commands** — `migration:generate`, `migration:run`, `migration:rollback`

### Long-term (v1.0)

- [ ] **Dev profiler** — JSON endpoint showing request timing, queries, events, services resolved
- [ ] **API Platform-like integration** — Auto-generate OpenAPI specs from entities and DTOs
- [ ] **Rate limiting** — Tower middleware with configurable rules
- [ ] **Caching** — PSR-6/16-like cache abstraction (Redis, in-memory)
- [ ] **Mailer** — SMTP/SendGrid/SES mailer with template support
- [ ] **Hot reload** — `synfony serve` with cargo-watch integration
- [ ] **Bundle system** — Distributable feature packages (like Symfony bundles)
- [ ] **WebSocket support** — Real-time via Axum's WebSocket support
- [ ] **gRPC support** — Via tonic integration
- [ ] **Multi-database** — Doctrine-like entity manager per connection

### Possible Symfony-like extensions

These are components from the Symfony ecosystem that could be ported:

| Symfony Component | Possible Synfony Crate | Priority |
|---|---|---|
| Form | `synfony-form` (HTML form handling) | Low (API-first) |
| Twig | `synfony-template` (Tera integration) | Medium |
| Mailer | `synfony-mailer` | High |
| Notifier | `synfony-notifier` (Slack, email, SMS) | Medium |
| Scheduler | `synfony-scheduler` (cron-like tasks) | Medium |
| Cache | `synfony-cache` | High |
| RateLimiter | `synfony-rate-limiter` | High |
| Lock | `synfony-lock` (distributed locking) | Medium |
| Uid | `synfony-uid` (UUID/ULID generation) | Low |
| Workflow | `synfony-workflow` (state machines) | Low |
| HttpClient | `synfony-http-client` (reqwest wrapper) | Medium |

---

## Project Structure

```
synfony/
├── Cargo.toml                    # Workspace root
├── README.md
├── .gitignore
│
├── synfony/                      # Main framework crate
│   └── src/lib.rs                # Re-exports + prelude
│
├── synfony-core/                 # Application lifecycle
│   └── src/
│       ├── application.rs        # Application (entry point + CLI)
│       ├── error.rs              # ApiError + ErrorResponse
│       ├── kernel.rs             # HTTP Kernel (builds Axum router)
│       └── state.rs              # AppState (shared across handlers)
│
├── synfony-macros/               # Procedural macros
│   └── src/
│       ├── controller.rs         # #[controller("/prefix")]
│       ├── route.rs              # #[route(GET, "/path")]
│       └── service.rs            # #[service]
│
├── synfony-di/                   # Dependency injection
│   └── src/
│       ├── container.rs          # Service container (TypeId → Arc<T>)
│       └── inject.rs             # Inject<T> Axum extractor
│
├── synfony-config/               # Configuration
│   └── src/
│       ├── env.rs                # .env cascading loader
│       └── loader.rs             # YAML config loader
│
├── synfony-console/              # CLI component
│   └── src/
│       ├── io.rs                 # ConsoleIO (tables, progress, prompts)
│       └── style.rs              # ConsoleStyle (colors)
│
├── synfony-security/             # Authentication & authorization
│   └── src/
│       ├── authenticator.rs      # Authenticator trait
│       ├── current_user.rs       # CurrentUser + OptionalUser extractors
│       ├── error.rs              # AuthError
│       ├── firewall.rs           # FirewallLayer (Tower middleware)
│       ├── jwt.rs                # JwtAuthenticator + JwtManager
│       ├── token.rs              # SecurityToken
│       └── voter.rs              # Voter trait + AccessDecisionManager
│
├── synfony-orm/                  # Database / ORM
│   └── src/
│       ├── connection.rs         # DatabaseConfig + connect()
│       ├── entity_manager.rs     # EntityManager (persist/update/remove)
│       └── repository.rs         # Repository<E> trait
│
├── synfony-validation/           # Request validation
│   └── src/
│       ├── error.rs              # ValidationError (422 response)
│       ├── json_body.rs          # JsonBody<T> (deserialize + validate)
│       └── query_params.rs       # QueryParams<T>
│
├── synfony-serializer/           # Serialization groups
│   └── src/
│       └── grouped_json.rs       # GroupedJson response type
│
├── synfony-serializer-macros/    # Serializer proc macros
│   └── src/lib.rs                # #[derive(SerializeGroups)] + #[groups]
│
├── synfony-event/                # Event dispatcher
│   └── src/
│       └── dispatcher.rs         # EventDispatcher + Listener trait
│
├── synfony-messenger/            # Message bus
│   └── src/
│       ├── bus.rs                # MessageBus (sync + async dispatch)
│       └── handler.rs            # MessageHandler trait
│
└── examples/
    └── hello-api/                # Complete example application
        ├── .env
        ├── config/app.yaml
        └── src/
            ├── main.rs           # App bootstrap + wiring
            ├── controller/       # 4 controllers (health, auth, user, admin)
            ├── dto/              # UserDto + CreateUserDto (with validation + groups)
            ├── entity/           # SeaORM User entity
            ├── event/            # UserCreatedEvent, UserDeletedEvent
            ├── message/          # SendWelcomeEmail, NotifyAdminsOfNewUser
            └── repository/       # UserRepository
```

---

## License

MIT

---

## Contributing

Synfony is in early development. Contributions welcome — especially:
- Tests for existing components
- Documentation improvements
- Bug reports from real-world usage
- New Symfony-equivalent components

If you're a Symfony developer who wants to help shape what a Rust framework should feel like for PHP developers, we'd love your input.
