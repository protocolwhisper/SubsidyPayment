# GPT App Integration Testing (Chat Input Templates)

This page provides practical prompt templates to run SnapFuel integration tests in ChatGPT Apps.

Target flow (6 steps):
1. Service search and list retrieval
2. Select one service from the list
3. Select task
4. Execute task
5. Show task completion and payment completion
6. Call the initially selected service and show result

## Prerequisites

- GPT Actions operation IDs: `searchServices`, `authenticateUser`, `getTaskDetails`, `completeTask`, `runService`, `getUserStatus`
- MCP tool names: `search_services`, `authenticate_user`, `get_task_details`, `complete_task`, `run_service`, `get_user_status`
- In the current implementation, payment is typically finalized when `runService` / `run_service` is executed.

## How to use in chat interface

- Paste the following as your first message in the ChatGPT App chat box.
- Continue in the same thread using the follow-up messages below when needed.

## 1) First message template (GPT Actions)

```text
I want to run an integration test for SnapFuel. Use only Action calls and do not supplement with prior knowledge.
Follow these 6 steps. For each step, report: "Action name", "key input", and "key output".

[Test Conditions]
- keyword: github
- max budget: 50000 cents
- intent: I want to create a GitHub issue
- consent: data_sharing_agreed=true, purpose_acknowledged=true, contact_permission=false
- task evidence (details): {"github_username":"octocat","github_repo":"octo-org/subsidy-payment","issue_title":"[Bug] OAuth callback fails on Safari","issue_body":"Repro: 1) Login 2) Redirect loop 3) 401 on callback. Expected: successful callback."}

[Procedure]
1) Search services and retrieve list
- Use searchServices with q/github + intent + max_budget_cents
- Show up to 3 candidates with service_id / sponsor / subsidy_amount_cents / required_task

2) Select one service
- Choose the best candidate based on relevance_score and required_task
- Explain the reason in one line

3) Select task
- Call getTaskDetails and identify task_name and required_fields

4) Execute task
- Call completeTask (using the details and consent above)
- Show can_use_service / consent_recorded / task_completion_id

5) Show task completion and payment-ready status
- Call getUserStatus and verify if the selected service is ready to run
- Show "ready or not", "sponsor", and "next action (runService)"

6) Execute the originally selected service and show result
- Call runService with input: "Create a GitHub issue in octo-org/subsidy-payment with title '[Bug] OAuth callback fails on Safari' and include reproduction steps."
- Show service / payment_mode / sponsored_by / message / output summary
- Finish with a pass/fail checklist for all 6 steps
```

## 2) First message template (MCP tools)

```text
Run a 6-step MCP integration test for SnapFuel.
Use tool outputs only; do not infer missing data.
For each step, report: tool name, input summary, and output summary.

Conditions:
- q: github
- intent: I want to create a GitHub issue
- max_budget_cents: 50000
- consent for complete_task:
  - data_sharing_agreed: true
  - purpose_acknowledged: true
  - contact_permission: false
- details JSON string for complete_task:
  {"github_username":"octocat","github_repo":"octo-org/subsidy-payment","issue_title":"[Bug] OAuth callback fails on Safari","issue_body":"Repro: 1) Login 2) Redirect loop 3) 401 on callback. Expected: successful callback."}

Procedure:
1. search_services to fetch candidates
2. pick one candidate (state reason)
3. get_task_details to fetch required task
4. complete_task to finish task
5. get_user_status to verify completion/readiness
6. run_service to execute and show result (including issue-creation outcome)

At the end, provide a pass/fail table for all 6 steps.
```

## Verification Points

- Search results include sponsor info, required task, and subsidy amount
- Consent is recorded when task completion is submitted
- Service execution returns `payment_mode` and `sponsored_by`
- Final service output is summarized clearly

## 3) Follow-up chat message examples

```text
Proceed with candidate #1. Before moving to the next step, summarize the latest tool/action output in one line.
```

```text
Before task execution, show the exact payload for complete_task/completeTask, then execute it.
```

```text
At the end, provide a pass/fail table with evidence fields returned at each step.
```
