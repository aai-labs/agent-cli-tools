# Code guidelines

Summarized from internal conventions, [dev.tasubo.com](https://dev.tasubo.com/) (Tadas Šubonis), [next-fastapi-boilerplate](https://bitbucket.org/tdisolutions/next-fastapi-boilerplate), and [owlang-study](https://github.com/tadas-subonis/owlang-study/blob/master/AGENTS.md).

## Project-defining rules (read first)

These are easy for agents to get wrong; treat them as **MUST** unless the user overrides.

- **CLI contracts are public API** — preserve command names, flags, exit behavior, and JSON output shapes unless a breaking change is intentional and documented.
- **Provider terminology wins** — expose resource names and fields using the provider's own API terms; do not invent aliases that make documentation and API responses harder to correlate.
- **Service modules own provider behavior** — keep provider-specific paths, versions, query parameters, bodies, pagination, and response shaping in `src/services/<provider>.rs`.
- **Shared HTTP owns cross-provider mechanics** — auth application, request execution, and common HTTP behavior belong in `src/http.rs`; service modules MUST NOT duplicate them.
- **Structured output** — successful results go to stdout as JSON; failures go to stderr as structured JSON. Diagnostic prose MUST NOT corrupt stdout.
- **Preserve provider responses** — return provider JSON directly where practical. When aggregating pages, replace only the page-local result array and retain the surrounding response shape. Reserve `_aai` for CLI metadata; bare provider arrays MAY be wrapped under `results` so `_aai.pagination` remains visible.
- **JSON and typed flags compose** — create/update commands that accept `--json` SHOULD merge typed flags predictably, with explicit typed flags taking precedence.
- **Credentials stay local** — secrets MUST NOT appear in source, fixtures, logs, command examples, or committed config. Use ignored files under `local/` for live profiles.
- **Errors** — use typed errors and actionable context; assertions are for true invariants, not provider or user input failures.

## Rule language (strictness)

- **MUST**: mandatory unless the user explicitly overrides.
- **SHOULD**: strong default; diverge only with a clear reason.
- **MAY**: optional and situational.

## Complexity (smells checklist)

- **Change amplification** — small change touches many files.
- **High cognitive load** — too many concepts before a safe edit.
- **Unknown unknowns** — unclear where to change or what breaks.

**Causes:** tangled **dependencies** and **obscurity**. Prefer zero tolerance for unnecessary duplication. Invest ~**10–20%** in design/refactor, not only “tests green.” For agents: simplify and dedupe; **propose** a small refactor instead of a local patch when design clearly wins.

## API and platform boundaries

Keep routers thin; business logic in services. Use **dependency injection**; avoid manual wiring in domain code. Preserve **async** boundaries for IO-heavy work. Logging **SHOULD** stay contextual (org id, resource ids). **Never** bypass organization scoping. **Explicit HTTP errors** at API boundaries.

## Backend layering (HTTP services)

- Reuse the project’s **DI** and shared infrastructure; do not construct clients ad hoc in handlers.

**New domain vertical** (adjust paths): models → repository → service → routes → register router → migration if schema changed → tests (**`TESTING.md`**).

Example layout:

```text
api/domains/<domain>/
  models.py
  repository.py
  service.py
  routes.py
```

## Domain and architecture (heuristics)

- **Ubiquitous language**; **package by feature**; separate **domain** from **infrastructure** (mental check: could you swap DB/HTTP/CLI — not a mandate to over-abstract).
- **Entities** have identity (prefer client-generated **UUID/ULID** when it fits). **Value objects** describe concepts without identity.
- **Aggregates** — invariants at root; load/save through root; **no object references between different aggregate roots** — link by ID (or explicit documented snapshot).
- **Repositories** — narrow names (`find_one_by_id`, `save`, …); persist at aggregate root. **Application services** orchestrate cross-entity processes; avoid **anemic** entities + **transactional script** services.
- **Factories** only when construction is non-trivial; avoid hiding factories inside repositories.

## HTTP and REST

Resources at stable URIs; nesting reflects ownership (`/users/{id}/orders/{id}`). **GET/POST/PUT-PATCH/DELETE** for intended semantics; GET must not mutate state (aside from logs). Prefer **JSON**, **HTTPS**, simple auth unless a stronger standard exists. **Version** under `/api/v1/...` (or deliberate alternative); prefix `/api` for non-API routes. Default **full resource** responses; optional `fields=` only when measured. Avoid **breaking** changes (additive OK). No **verbs** in path segments (`/create`, `/delete`). Richardson **level 2** is enough for most internal APIs.

| Code | Use |
|------|-----|
| `200` | Successful read or update **with** body |
| `201` | Resource created |
| `204` | Success **without** body (e.g. delete) |
| `400` | Business precondition failed (or `422` if reserved for schema) |
| `401` / `403` | Unauthenticated / unauthorized |
| `404` | Missing or not visible to caller |
| `409` | Conflict / uniqueness / stale state |
| `422` | Request shape validation (e.g. FastAPI/Pydantic) |

## Tests

Conventions, stacks, and examples: **`TESTING.md`** (single source of truth).

## Immutability and functional style

Prefer **immutable** data and transform-and-return. Combine **OOP for vocabulary** with **functional transitions** (immutable records + `replace` / `with*`). In ETL-style code, prefer **linear pipelines** over deep nesting; use small **types** instead of long tuples when arity grows.

Prefer **immutable** data and “transform and return” over mutating inputs; reduces surprises under concurrency and unclear collaborators.
Combine **OOP for domain vocabulary** with **functional transitions**: methods return new state (e.g. immutable records + `replace` / `with*` wrapped in domain-named methods).
In data/ETL-style code, prefer **linear pipelines** (`map`/`filter`/`reduce`) over deep nested calls; keep steps **loosely coupled** so steps can be added or dropped without editing hidden call chains.
When arity grows, use small **types** (dataclasses/value objects) instead of long tuples and `starmap` soup

## Design heuristics (compressed)

- **Encapsulation** — behavior on the type that owns the state; static “services” that only take arguments → consider **move method**. Intent on the domain type (`order.getTotalPrice()`), not `OrderManager.getTotalPrice(order)`.
- **Manager/Util/Helper** dumping grounds → move behavior to domain unless wrapping third-party APIs.
- **Composition over inheritance**; avoid deep `extends` / god bases; small composable objects.
- **Deep modules** — small public surface, rich internals; avoid shallow DB-table mirrors.
- **Splitting logic** — prefer a small **public class/service** when extracted logic has real responsibility; use **nested functions** only for pure readability inside one method.
- **Law of Demeter** — avoid `a.getB().getC().getD()`; prefer intention-revealing methods on the receiver. Fix **feature envy**, **message chains** (**hide delegate**), **middle man**, **shotgun surgery**. Goal: one concept → **one** module to touch.
- **Many `if`s on one concept** → polymorphism or **Strategy** (OOP or function table).
- **Excessive conditionals** → named predicates (`order.isShippable()`). **Guard clauses** for shallow nesting.

## Functions and parameters

A function **SHOULD** do **one thing** at one abstraction level; if comments separate sections → extract.

| Count | Quality |
|-------|---------|
| 0 | Best |
| 1–2 | Good |
| 3 | Acceptable |
| 4+ | Avoid |

**When arity grows:** parameter object; preserve whole object; replace parameter with query.

**Anti-patterns:** flag arguments (split methods); output arguments (prefer return); pass-through classes; dead code.

## Naming and comments

**Ubiquitous language**; names reflect **side effects** where relevant (`getOrCreateUser`). Longer scope → more descriptive. Avoid `data`, `result`, `temp`, vague `Manager`/`Helper`. Hard naming → fuzzy design → fix design first.

Prefer **clear code** over comments. Comments explain **why**, invariants, trade-offs — not **what**. Delete commented-out dead blocks.

## Control flow

Prefer **flat pipelines** over nested if/else:

```typescript
function processOrder(orderId: string) {
  const order    = loadOrder(orderId);
  const ready    = ensureReadyForProcessing(order);
  const paid     = chargeOrder(ready);
  const notified = notifyCustomer(paid);
  saveOrder(notified);
}
```

Repeated `switch` on one concept → types/polymorphism. **Guard clauses**; avoid double negatives.

## Parameters, validation, and nulls

**Fail fast** at boundaries; avoid repeating validation everywhere — **value objects** / validated DTOs from factories so inner layers receive valid types. Cautious with framework beans in invalid states. Prefer **Optional**, empty collections, or **Null Object** over raw null in your own APIs.

## Events and integration

Use an **event bus** when subsystems must stay **loose**, publisher must not block on slow handlers, fire-and-forget, or **multiple** reactions are needed. Mind **listener lifecycle** (leaks). **Shared event types** in a focused module when they cross features.

## Refactoring discipline

1. Do **not** mix a large refactor with a new feature in one change.  
2. Tests or smoke cover behavior before you move it.  
3. **Small, reversible** steps; run tests after each.  
4. Re-align if the plan changes mid-flight.

Non-trivial work: seriously consider **two designs** before committing.

## Code review priorities

1. Correctness and regressions  
2. Data contracts and schema safety  
3. Cache keys and invalidation (when applicable)  
4. Auth and permissions  
5. Loading and async UX (when applicable)  
6. Test coverage gaps  

Do not lead with style-only feedback unless it affects correctness or maintainability.

## Definition of done

- Happy path + important edge cases  
- Validation and authorization correct on touched surfaces  
- Tests added/updated and passing  
- Lint and types pass for touched areas  
- Migrations when schema changes  
- No unrelated refactors or format churn  

## Red flags checklist

If several apply, consider a focused refactor before merge:

- [ ] Hard to describe the module in one sentence  
- [ ] Shallow module (interface as complex as implementation)  
- [ ] Deep inheritance `A → B → C → D` without strong reason  
- [ ] Nested workflows where a flat pipeline would do  
- [ ] Same domain rule in several places  
- [ ] Same term, different meanings across modules  
- [ ] Aggregate roots hold direct references to other roots  
- [ ] Anemic entities + rules only in services  
- [ ] Business rules in HTTP controllers  
- [ ] Verbs in REST URLs  
- [ ] Breaking API without version strategy  
- [ ] Large classes / long functions mixing abstraction levels  
- [ ] Flag arguments or long parameter lists  
- [ ] Message chains or middle-man classes  
- [ ] Shared mutable state scattered  
- [ ] Comments restating **what** not **why**  
- [ ] Logic only testable via HTTP or DB  
