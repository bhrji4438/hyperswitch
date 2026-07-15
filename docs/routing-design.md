# Cost-Aware Routing Design

## Problem Statement

Merchants often configure multiple payment connectors to improve payment reliability and availability. The current routing pipeline determines eligible connectors using routing rules, hybrid routing, and eligibility analysis. However, once the eligible connector list is produced, there is no optimization based on transaction processing cost.

This proposal introduces a lightweight Cost-Aware Routing strategy that prefers the lowest-cost connector among the already eligible connectors without changing the existing routing behavior or merchant routing configuration.

---

## Goals

- Introduce a Cost-Aware Routing strategy into the existing payment routing pipeline.
- Reuse the current routing flow without introducing a parallel routing framework.
- Keep the implementation isolated and easy to review.
- Preserve existing eligibility, retry, and fallback behavior.
- Keep the change production-safe with minimal surface area.

---

## Non Goals

The following are intentionally out of scope for this assignment:

- Merchant configurable pricing rules.
- Database or schema changes.
- New public APIs or routing configuration models.
- External pricing services.
- Machine learning or weighted routing algorithms.
- Changes to existing routing strategies such as Static, Hybrid, or Eligibility Routing.

---

## Proposed Solution

The Cost-Aware Routing strategy will execute **after eligibility analysis** and **before connector selection**.

At this stage, all routing decisions have already produced the final list of eligible connectors. The strategy will simply reorder those connectors based on predefined processing cost while preserving the existing order for connectors with equal cost.

The strategy does not modify routing rules, connector eligibility, retry logic, or payment execution. It only influences the preferred connector selection among already valid candidates.

---

## Decision Flow

```
Payment Request
        │
        ▼
Static / Hybrid Routing
        │
        ▼
Eligibility Analysis
        │
        ▼
Cost-Aware Routing
        │
        ▼
Connector Selection
        │
        ▼
Payment Execution
```

---

## Acceptance Criteria

The implementation will be considered complete when:

- Cost-aware routing executes as part of the active payment flow.
- Eligible connectors are reordered according to configured cost.
- Equal-cost connectors preserve their original priority.
- Unknown connectors are handled safely without failing the payment.
- Existing payment functionality remains unaffected.
- Unit tests cover both happy path and edge cases.
- Routing decisions are visible through existing logging mechanisms.

---

## Risks

| Risk | Mitigation |
|------|------------|
| Hardcoded cost values become outdated | Isolate cost lookup so it can later be replaced with configuration or a service. |
| Cost routing changes retry order | Apply routing only after eligibility so only valid connectors are reordered. |
| Logging sensitive payment information | Log connector decisions only. Do not log PAN, BIN, CVV or customer data. |
| Future routing strategies require additional business rules | Keep the routing algorithm isolated behind a single strategy entry point. |

---

## Future Improvements

Potential production enhancements include:

- Configuration-driven connector pricing.
- Merchant-specific routing preferences.
- Region and currency aware pricing.
- Dynamic pricing updates from an external service.
- Weighted routing using both cost and success rate.
- Routing decision trace API for operational visibility.

---

## Design Decisions

During architecture exploration, multiple implementation approaches were evaluated.

The selected approach intentionally:

- Reuses the existing payment routing pipeline.
- Avoids changes to public API models.
- Avoids schema and database changes.
- Minimizes the implementation to a small, isolated strategy.
- Favors maintainability and reviewability over introducing a new routing framework.

This follows the principle of making the smallest production-safe change required to deliver the feature.