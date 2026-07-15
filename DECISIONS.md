# Decisions

## What I built

Implemented a Cost-Aware Routing strategy that reorders the eligible connector list after eligibility analysis and before connector execution.

## Why this approach

- Minimal surface area
- No changes to existing routing algorithms
- Reuses existing connector selection flow
- Pure function
- Easy to replace static cost table with configuration later

## What I intentionally skipped

- Database configuration
- Merchant configurable costs
- Dynamic fee lookup
- Admin APIs
- Caching

## Risks

- Static cost table
- Unknown connectors sorted last
- Cost should be configurable

## Future Improvements

- Merchant configurable costs
- Redis cache
- Dashboard configuration
- Routing decision trace