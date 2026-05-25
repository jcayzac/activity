# Application Architecture Rules

This rule book contains the enforceable architectural rules for Rust applications in this repository. Use it for high-level design and architectural review. Companion files under section directories in this book carry rationale and source provenance for individual rules.

---

## Make the Architecture Describe the Problem

### `problem-structure/domain-capability-layout`

Organize top-level code by domain and capability, not by framework

**Rule:** Organize top-level crates and modules around the business domain and
the application's capabilities. A reader looking at the workspace should learn
what the system does before learning which web framework, runtime, ORM, or
cloud SDK it uses.

**Bad**

```text
crates/
  api/
  db/
  models/
  services/
  utils/
```

**Good**

```text
crates/
  billing/
  customer/
  reporting/
  support/
```

### `problem-structure/catch-all-shared-modules`

Do not create catch-all shared crates or modules

**Rule:** `common`, `shared`, `base`, `core_utils`, and similarly vague modules
must not become sinks for unrelated code. Shared code must be both cohesive and
explicitly owned: either domain policy shared by multiple features, or
technical infrastructure shared by multiple adapters.

## Separate Policy from Detail

### `policy-detail/technology-independent-policy`

Keep business rules independent of technology

**Rule:** Domain policy, business rules, and use-case logic must not depend
directly on storage engines, transport protocols, UI frameworks, cloud SDKs,
operating-system APIs, or framework-owned types.

**Bad**

```rust
use sqlx::PgPool;

pub async fn place_order(pool: &PgPool, request: CreateOrder) -> Result<OrderId> {
    // business rules and SQL are mixed together
    todo!()
}
```

**Good**

```rust
pub trait OrderStore {
    fn save(&self, order: &Order) -> Result<(), SaveOrderError>;
}

pub struct PlaceOrder<S> {
    store: S,
}

impl<S> PlaceOrder<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: OrderStore> PlaceOrder<S> {
    pub fn execute(&self, request: CreateOrder) -> Result<OrderId, PlaceOrderError> {
        // business rules depend only on the contract they need
        todo!()
    }
}
```

### `policy-detail/edge-details`

Treat frameworks, storage, and delivery mechanisms as edge details

**Rule:** Web handlers, CLI commands, GUI views, message consumers, database
access, external API clients, file-system access, and observability sinks must
sit at the architectural edge. They may drive the application or be driven by
it, but they must not own the core policy.

### `policy-detail/runtime-free-core`

Keep runtime-specific concerns out of the core

**Rule:** Domain and use-case code must not require a particular async runtime,
actor framework, scheduler, or event loop unless concurrent execution semantics
are themselves part of the problem domain. Keep runtime-owned types and control
flow in adapters or composition code.

## Control Dependency Direction Deliberately

### `dependency-direction/detail-policy-direction`

Dependencies must point from volatile details toward stable policy

**Rule:** In a valid dependency graph, source code dependencies must move from
detail to policy. Outer crates or modules may depend on inner policy crates or
modules; policy crates or modules must not depend on the details that implement
them.

**Bad**

```text
domain  --> infrastructure
api     --> infrastructure
```

**Good**

```text
api             --> application
infrastructure  --> application
application     --> domain
```

### `dependency-direction/consumer-owned-traits`

Define boundary traits on the consuming side

**Rule:** Define a trait, interface, or protocol in the module that needs the
capability, not in the module that happens to implement it. Implementations
belong on the detail side of the boundary.

**Bad**

```rust
// infrastructure/src/payments.rs
pub trait StripeGateway {
    fn charge(&self, amount: Money) -> Result<Receipt>;
}
```

**Good**

```rust
// application/src/payments.rs
pub trait PaymentGateway {
    fn charge(&self, amount: Money) -> Result<Receipt, ChargeError>;
}

// infrastructure/src/stripe.rs
impl PaymentGateway for StripeClient {
    fn charge(&self, amount: Money) -> Result<Receipt, ChargeError> {
        todo!()
    }
}
```

### `dependency-direction/acyclic-graph`

Keep the dependency graph acyclic

**Rule:** Crate and module dependencies must be acyclic. If two areas need each
other, extract the shared policy or contract into a third unit that both can
depend on.

### `dependency-direction/real-architectural-seams`

Introduce ports and adapters only at real architectural seams

**Rule:** Add a trait or adapter only when it separates policy from detail,
isolates an independently changing concern, or creates a real test boundary. Do
not wrap local implementation details merely to satisfy a pattern.

## Separate Concerns by Reason to Change

### `change-boundaries/single-change-reason`

Give each crate and module one primary reason to change

**Rule:** Each crate and module must have one primary reason to change. If
policy, persistence, presentation, formatting, and integration change for
different stakeholders or on different schedules, they must live in different
modules.

### `change-boundaries/policy-orchestration-adapters`

Keep domain rules, application orchestration, and adapters distinct

**Rule:** Separate three kinds of code:

1. domain logic that enforces business invariants
2. application or use-case logic that coordinates workflows
3. adapters that translate to and from external systems

Do not collapse these concerns into a single module.

### `change-boundaries/thin-entry-points`

Keep entry points thin

**Rule:** HTTP handlers, CLI subcommands, queue consumers, schedulers, and UI
event handlers may parse input, invoke a use case, and format output. They must
not own business decisions.

**Bad**

```rust
async fn create_user(Json(req): Json<CreateUserRequest>, State(db): State<Db>) -> Response {
    if req.age < 18 {
        return StatusCode::FORBIDDEN.into_response();
    }

    // more business rules here
    todo!()
}
```

**Good**

```rust
async fn create_user(
    Json(req): Json<CreateUserRequest>,
    State(app): State<AppState>,
) -> Response {
    match app.create_user.execute(req.into()) {
        Ok(user) => Json(UserResponse::from(user)).into_response(),
        Err(err) => map_error(err),
    }
}
```

## Make Boundaries Explicit and Narrow

### `boundaries/application-operations`

Define application operations explicitly

**Rule:** Expose the system's capabilities through explicit use cases,
application services, or similarly named boundary types or functions. External
actors must call the application through those operations rather than reaching
into internal modules piecemeal.

### `boundaries/data-translation`

Translate data at every architectural boundary

**Rule:** When data crosses a boundary, convert it into a representation owned
by the receiving side. Do not pass ORM rows, HTTP request types, protobuf
messages, database schemas, or framework-specific error types into policy code.

**Bad**

```rust
pub fn execute(row: sqlx::postgres::PgRow) -> Result<Account> {
    // application code now knows about sqlx row layout
    todo!()
}
```

**Good**

```rust
pub struct LoadAccountInput(AccountId);

impl LoadAccountInput {
    pub fn new(account_id: AccountId) -> Self {
        Self(account_id)
    }
}

pub fn execute(input: LoadAccountInput) -> Result<Account, LoadAccountError> {
    todo!()
}
```

### `boundaries/adapter-metadata-isolation`

Keep adapter-specific traits, derives, and annotations out of core types

**Rule:** Domain and use-case types must not depend on serialization, ORM, RPC,
or web-framework annotations merely to satisfy an adapter. If an external
boundary needs a different shape or set of derives, define a boundary type and
map to or from the core type.

### `boundaries/purposeful-small-contracts`

Keep boundary contracts purposeful and small

**Rule:** A port or service contract must describe a purposeful conversation or
capability. Do not create "god traits", kitchen-sink services, or generic
repositories that erase the domain language.

## Compose at the Edge

### `composition/composition-root`

Wire concrete implementations in a composition root

**Rule:** Construct adapters, read configuration, and choose concrete
implementations at the process edge: `main`, startup code, integration
harnesses, or an equivalent composition root.

### `composition/configuration-use-separation`

Separate configuration from use

**Rule:** Policy code must declare required capabilities, not discover or
construct them. Service configuration must be separate from service use.

### `composition/explicit-dependencies`

Make dependencies explicit in types and function signatures

**Rule:** Pass dependencies through constructors, function parameters, or
explicit builder or factory code. Policy code must not reach for globals,
singletons, ambient service locators, or hidden thread-local context.

## Design for Replacement and Testability

### `replacement-testing/infrastructure-free-core`

Core logic must run without infrastructure

**Rule:** Domain and use-case code must be executable in unit tests without a
real database, network, file system, clock service, UI, or background
scheduler.

### `replacement-testing/swappable-external-dependencies`

Every external dependency must be swappable

**Rule:** Any capability owned outside the current boundary must be reachable
through a contract that can be implemented by production and test adapters.

### `replacement-testing/boundary-focused-tests`

Test policy and adapters at their own boundaries

**Rule:** Tests for policy code must exercise public use-case and domain
contracts. Tests for adapters must verify translation and integration at the
boundary. Do not couple tests to private wiring when the architectural contract
can be tested directly.

## Keep Cross-Cutting Concerns Orthogonal

### `cross-cutting/technical-policy-separation`

Keep technical cross-cutting concerns out of business policy

**Rule:** Logging, tracing, metrics, retries, caching, authentication plumbing,
and transport concerns must wrap, observe, or support policy code without
becoming the place where business decisions live.

### `cross-cutting/shared-module-classification`

Shared modules must declare whether they are policy or infrastructure

**Rule:** A shared crate or module must be clearly one of two things: domain
policy shared by multiple capabilities, or technical infrastructure shared by
multiple adapters. It must not mix both.

## Record and Enforce the Architecture

### `governance/architecture-decision-records`

Record architecture-significant decisions

**Rule:** Decisions that affect dependency direction, boundary ownership, data
contracts, runtime model, or cross-cutting strategy must be captured as
architecture decision records and kept with the code.

### `governance/mechanical-boundary-enforcement`

Enforce important boundaries mechanically

**Rule:** If a boundary matters, encode it in workspace dependencies, module
visibility, CI checks, lint rules, or architecture tests. Do not rely on tribal
knowledge.

## Prefer the Simplest Design That Preserves These Rules

### `simplicity/least-complex-design`

Choose the least complex design that preserves the rules

**Rule:** If more than one design satisfies the rules above, choose the one
with fewer concepts, fewer layers, and fewer moving parts.
