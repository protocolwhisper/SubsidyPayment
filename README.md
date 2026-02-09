# SubsidyPayment

Payload Exchange Extended (Campaign + Sponsor Subsidy Layer for x402)

Purpose:  
Intercept the x402 402 Paywall via a proxy and enable sponsors to cover payments in exchange for user task execution and/or data provision.  
Additionally, sponsors can create and distribute “campaigns,” allowing users to naturally utilize this mechanism through major AI / developer UIs.

---

## Product Scope

### What this product enables
- Provides the following payment methods for access to x402-protected resources (APIs, data, digital content, etc.):
  - Sponsors cover all or part of the payment (in exchange for requesting user tasks or data provision).
  - Users pay directly (fallback).
- Sponsors can issue campaigns defining:
  - Which users to target  
  - What they want users to do  
  - How much they will subsidize
- Users can select sponsor-backed x402-enabled services based on:
  - Service name, or  
  - Desired functionality / features they want agents to use
- Users can store and reuse profiles and survey responses so they don’t have to repeat the same inputs each time.
- When credits run out during agent execution, users can be notified and guided to complete tasks.

---

## Core Concepts

- Resource: An upstream paid endpoint protected by x402  
- Proxy: Intercepts 402 responses and presents the paywall and task flow  
- Sponsor: The entity covering the payment (companies, and in the future, agents as well)  
- Campaign: A recruitment unit created by sponsors (target, objective, budget, tasks, registration to sponsor services, data requests, consent conditions)  
- Offer: Sponsor terms at the resource level (discount rate, cap, required tasks, collected data)  
- Action Plugin: An extensible plugin layer for adding tasks or data collection  
- Consent Vault: A layer managing explicit consent, usage purpose, retention period, and contact permissions  

---

## Requirements (Prioritized)

### P0: Must work first (without breaking upstream compatibility)

- x402 Proxy
  - Proxies requests to upstream x402 resources and displays a paywall upon receiving a 402 response
  - When sponsor payment succeeds, executes payment upstream and returns the resource response to the user

- Paywall UI
  - Displays active sponsor conditions (if any) and allows task selection and execution
  - Clearly indicates the presence of a sponsor to the user

- Action Plugin System
  - Preserves existing actions while allowing additional tasks to be added later
  - Provides a consistent interface for task start / verification / completion

- Resource Discovery
  - Allows users to search and browse available x402 resources

- Direct Payment (Fallback)
  - Maintains a direct payment path if no sponsor is available or if the user declines

- ChatGPT Integration (MCP + Widget)
  - Provides an MCP server enabling Paywall / Resource widget display
  - Maintains direct payment fallback if no sponsor is available or consent is declined

- Claudecode / OpenClaw Integration (MCP + Widget)
  - Provides Skills enabling Paywall / Resource widget display

- Deployment
  - Treats Vercel deployment as first-class; iframe asset loading must not break
  - Required environment variables are documented in the README and deployments are reproducible

Completion Criteria (Minimum)
- End-to-end flow works locally and in production:  
  402 → Paywall → Action → Sponsor payment → Resource delivery
- Sponsor display and consent UI are visible and verifiable

---

### P1: Core additional requirements (product value layer)

#### ToB: Sponsor Campaign Builder (Chat AI UI)

- Sponsors can create campaigns through natural language Q&A
  - From target attributes, recommended x402-enabled services are ranked and suggested
  - From stated objectives, recommended task sets, subsidy amounts, discount rates, and caps are proposed
  - Creators can publish by selecting proposals

- Sponsor Dashboard
  - Campaign list (status, spend, completion count)
  - Data Inbox (received data count, content review, export)

#### ToC: Practical user entry points

- Users can search by service name and see sponsor availability
- Users can search by function and choose sponsor-backed tools  
  (e.g., scraping, design, storage)

- Profile Vault
  - Stores basic data such as email, region, IP type, and services in use
  - Saves survey responses for reuse during task execution

- Consent / Compliance
  - No data is transferred to sponsors without explicit opt-in consent
  - Usage purpose, retention period, and sponsor contact permissions must be displayed

- Notification
  - When credits are exhausted (e.g., during agent operation), users can be notified and guided to tasks

Completion Criteria (Minimum)
- Sponsors can create and publish campaigns visible in user search and paywalls
- Sponsors can review results in the Data Inbox
- Users can shorten inputs via profile storage and reuse
- Notifications can route users to paywalls

---

### P2: Scale requirements (longer-term leverage)

- Advanced Recommendation Engine
  - Evolve from rule / tag-based logic to outcome-driven weighting
  - Gradual introduction of embeddings and collaborative filtering

- Fraud / Low-quality mitigation
  - Progressive human verification
  - Stronger task proof mechanisms (external integrations, webhooks, revalidation)

- Multi-client integration expansion
  - Provide a common HTTP API and SDK for clients beyond ChatGPT  
    (Claude, Codex, OpenClaw, etc.)
  - Enable completion via “link-based paywalls” even in environments where UI embedding is not possible

- Analytics and audit
  - Funnel tracking (view / start / complete / payment)
  - Sponsor viewing / export audit logs
  - Data redaction and minimization operations

---

## Data Collection Framework (Provided as Action Plugins)

### Basic personal data
- Email
- Region
- IP type
- Optional signup requests to sponsor pages
- Minimum human verification to ensure the user is not a bot

### Survey data
- Demographics
- Goals / KPIs
- Organization size
- Prompts (only when necessary)
- Agents / skills usually used
- Media consumption
- Competitor usage
- Satisfaction with current services
- Price sensitivity
- Switching triggers
- Alternative comparisons

---

## Compliance (Must)

- Explicit user consent  
  (manageable at item level and sponsor level)
- Usage purpose disclosure
- Data retention disclosure
- Sponsor contact disclosure  
  (user can opt in or opt out)

---

## Non-Goals (Out of scope for initial phase)

- Full KYC or heavy identity verification
- Advanced recommendation models from the start  
  (begin with rules / tags)
- Native UI integrations for all clients from the start  
  (absorb via MCP + HTTP first)
- Heavy data infrastructure integrations  
  (begin with export + audit logs)

---

## Suggested Milestones

- M0: End-to-end flow with upstream compatibility (P0)
- M1: Service search, sponsor visibility, real usage routes (P1 ToC first half)
- M2: Campaign Builder Chat, publishing, Data Inbox (P1 ToB)
- M3: Profile Vault + Consent completion (P1 operational requirements)
- M4: Notifications and multi-client API / SDK (P1 latter half)

---
