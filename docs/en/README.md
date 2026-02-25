# PayloadExchange Documentation

PayloadExchange is a marketplace platform that connects AI agent developers with sponsors who subsidize compute costs in exchange for verified usage data and direct user access.

---

## Overview

### What is PayloadExchange?

PayloadExchange is a sponsored compute platform that enables AI agents to monetize their tool usage through micropayments. The platform operates on a reverse advertising model where sponsors pay developers to use their APIs and services instead of generic alternatives.

**Key Concepts:**
- **Sponsored Compute**: Sponsors subsidize operational costs for AI agents
- **Micropayments**: Real-time crypto payments per API call or tool usage
- **Verified Data**: Sponsors receive authenticated usage metrics and attribution

### Problem Statement

AI agent development incurs significant operational costs:
- **LLM API costs**: Token consumption for model inference
- **Tool/API costs**: Third-party service fees (search APIs, database queries, external services)
- **Infrastructure costs**: Compute resources for agent execution

These costs scale with usage, making agent deployment expensive for developers.

### Solution Architecture

PayloadExchange addresses this through a marketplace model:

1. **Sponsors** create campaigns offering micropayments for tool usage
2. **Developers** integrate sponsored tools into their agents
3. **Platform** handles payment settlement via x402 protocol
4. **Verification** ensures legitimate usage and prevents abuse

---

## Current Implementation Status (as of 2026-02-25)

### Rust backend (`src/`)
- Core routes: `/campaigns`, `/proxy/{service}/run`, `/sponsored-apis`, `/payments`, `/creator/metrics`
- GPT routes: `/gpt/services`, `/gpt/auth`, `/gpt/tasks/{campaign_id}`, `/gpt/tasks/{campaign_id}/complete`, `/gpt/tasks/{campaign_id}/zkpassport/init`, `/gpt/user/status`, `/gpt/user/record`, `/gpt/preferences`
- Discovery aliases: `/agent/discovery/services`, `/claude/discovery/services`, `/openclaw/discovery/services`
- zkPassport routes: `/verify/zkpassport`, `/zkpassport/session/{verification_token}`, `/zkpassport/session/{verification_token}/submit`

### MCP server (`mcp-server/`)
- Transport/API: Streamable HTTP on `/mcp`, plus `/health` and OAuth metadata endpoints
- Registered tools (14):
  - `search_services`
  - `authenticate_user`
  - `create_campaign_from_goal`
  - `get_service_tasks`
  - `get_task_details`
  - `start_zkpassport_verification`
  - `complete_task`
  - `run_service`
  - `get_user_status`
  - `get_user_record`
  - `get_preferences`
  - `set_preferences`
  - `weather`
  - `github_issue`
- Widget resources (6):
  - `services-list`
  - `services-list-v2`
  - `service-tasks`
  - `task-form`
  - `service-access`
  - `user-dashboard`

### Database / migration status
- Latest migration: `0014_zkpassport_verifications.sql`
- Smart suggestion migrations (`0011`, `0012`, `0013`) are already applied in the codebase

### Kiro specs progress
- `gpt-apps-integration`: 33/33 completed
- `smart-service-suggestion`: 32/32 completed
- `refactor-to-gpt-app-sdk`: 27/27 completed
- `autonomous-agent-execution`: 0/41 (not implemented yet)

---

## Core Concepts

### Skills and Tools

A **Skill** (also referred to as a "Tool" or "Function") is a callable capability exposed to an LLM. Skills are defined using JSON schemas that describe function signatures, parameters, and return types.

**Example Skill Definition:**
```json
{
  "name": "get_weather",
  "description": "Retrieve current weather conditions",
  "parameters": {
    "type": "object",
    "properties": {
      "city": {
        "type": "string",
        "description": "City name"
      }
    },
    "required": ["city"]
  }
}
```

**Execution Flow:**
1. User submits a prompt to the LLM
2. LLM identifies relevant skills based on the prompt
3. LLM requests skill execution with parameters
4. Skill executes and returns results
5. LLM incorporates results into its response

**PayloadExchange Integration**: Sponsors offer payment incentives for developers to use their specific skills instead of competing alternatives.

### Model Context Protocol (MCP)

The **Model Context Protocol** is a standardized interface for connecting AI tools to language models. MCP provides a universal connector that works across different LLM providers (OpenAI, Anthropic, Google, etc.) without requiring provider-specific integrations.

**Key Benefits:**
- **Interoperability**: Write once, use across multiple LLM platforms
- **Standardization**: Consistent interface for tool integration
- **Extensibility**: Easy to add new tools and capabilities

**PayloadExchange Implementation**: Sponsored tools are distributed as MCP servers, enabling instant integration across any MCP-compatible agent framework.

---

## System Architecture

### Transaction Flow

```
┌─────────────────┐
│ Sponsor Company │
└────────┬────────┘
         │ Funds Campaign
         ↓
┌─────────────────┐
│ PayloadExchange │
│   Marketplace   │
└────────┬────────┘
         │ Lists Sponsored Tool
         ↓
┌─────────────────┐
│   Developer/    │
│      Agent      │
└────────┬────────┘
         │ Uses Tool via MCP
         ↓
┌─────────────────┐
│ Service Provider│
└────────┬────────┘
         │ Triggers x402 Request
         ↓
┌─────────────────┐
│ PayloadExchange │
│ Payment Layer   │
└────────┬────────┘
         │
    ┌────┴────┐
    ↓         ↓
┌────────┐ ┌──────────────┐
│Payment │ │ Usage Data   │
│Settlement│ Attribution │
└────────┘ └──────────────┘
```

### Transaction Lifecycle

1. **Campaign Creation**: Sponsor defines campaign parameters (target audience, budget, payout per call, API endpoint)

2. **Tool Discovery**: Developer browses marketplace and selects sponsored tool

3. **Integration**: Developer installs MCP server or SDK wrapper for the sponsored tool

4. **Execution**: Agent invokes sponsored tool during normal operation

5. **Payment Processing**: x402 protocol validates request and transfers payment from sponsor wallet to developer wallet

6. **Data Attribution**: Platform logs usage metrics and sends verified data to sponsor

---

## Platform Features

### Sponsor Portal

**Campaign Management**
- Define target audience and targeting criteria
- Set budget limits and payout schedules
- Configure API endpoints and integration requirements
- Monitor campaign performance in real-time

**Analytics Dashboard**
- Verified tool usage metrics
- Budget consumption and burn rate
- User engagement and attribution data
- ROI analysis and optimization recommendations

### Developer Portal

**Marketplace**
- Browse available sponsored tools
- Filter by payout rate, category, and requirements
- View integration documentation and examples
- Track earnings and payment history

**Wallet Integration**
- Connect EVM-compatible wallet (Ethereum, Polygon, etc.)
- Receive micropayments in real-time
- View transaction history and earnings
- Manage multiple sponsored tool integrations

### Integration Layer

**MCP Server Distribution**
- Standardized MCP servers for sponsored tools
- Automatic x402 payment header injection
- Payment validation and verification
- Usage tracking and reporting

**SDK Support**
- Language-specific SDKs for non-MCP integrations
- Simplified payment flow handling
- Built-in error handling and retry logic
- Developer-friendly API wrappers

### MCP Tool Spec (MVP)

This tool auto-creates campaigns by taking a purpose and target audience and selecting relevant services and tasks.

**Tool Name**
- `create_campaign_from_goal`

**Goal**
- Generate campaign fields from a purpose/target input and call `POST /campaigns`

**Tool Definition (MCP)**
```json
{
  "name": "create_campaign_from_goal",
  "description": "Create a sponsor campaign from a purpose and target audience.",
  "parameters": {
    "type": "object",
    "properties": {
      "purpose": { "type": "string" },
      "sponsor": { "type": "string" },
      "target_roles": { "type": "array", "items": { "type": "string" } },
      "target_tools": { "type": "array", "items": { "type": "string" } },
      "budget_cents": { "type": "number" },
      "query_urls": { "type": "array", "items": { "type": "string" } },
      "region": { "type": "string" },
      "intent": { "type": "string" },
      "max_budget_cents": { "type": "number" }
    },
    "required": ["purpose", "sponsor", "target_roles", "budget_cents"]
  }
}
```

**Input (JSON Schema Overview)**
- `purpose`: Purpose summary (required)
- `sponsor`: Sponsor name (required)
- `target_roles`: Target roles array (required)
- `target_tools`: Target tools array (optional)
- `budget_cents`: Budget in cents (required)
- `query_urls`: Upstream URLs array (optional)
- `region`: Target region (optional)
- `intent`: Detailed intent (optional)
- `max_budget_cents`: Per-call budget cap (optional)

**MVP Flow**
1. Search candidate services via `GET /gpt/services`
2. Pick `required_task` and `target_tools` from top candidates
3. Create campaign via `POST /campaigns`

**Output**
- `campaign_id`: Created campaign ID
- `campaign`: Created campaign payload
- `selected_service_key`: Selected service key
- `selected_offer`: Selected offer (campaign_id / sponsor / required_task / subsidy_amount_cents)
- `selected_services`: Candidate services used for selection
- `selected_task`: Chosen task details (required_task / subsidy_per_call_cents)
- `rationale`: Summary of selection reasoning

**Example Input**
```json
{
  "purpose": "Improve AI chat assistance",
  "sponsor": "Acme Corp",
  "target_roles": ["customer-support", "product-manager"],
  "budget_cents": 25000,
  "intent": "Improve FAQ response quality",
  "max_budget_cents": 150
}
```

**Example Output (structuredContent)**
```json
{
  "campaign_id": "2f6d2c0b-3c7a-4a0a-9a2e-6f2b6b7e8d90",
  "campaign": {
    "id": "2f6d2c0b-3c7a-4a0a-9a2e-6f2b6b7e8d90",
    "name": "Acme Corp Improve AI chat assistance",
    "sponsor": "Acme Corp",
    "sponsor_wallet_address": null,
    "target_roles": ["customer-support", "product-manager"],
    "target_tools": ["faq_search"],
    "required_task": "share_feedback",
    "subsidy_per_call_cents": 120,
    "budget_total_cents": 25000,
    "budget_remaining_cents": 25000,
    "query_urls": [],
    "active": true,
    "created_at": "2026-02-23T09:00:00Z"
  },
  "selected_service_key": "faq_search",
  "selected_offer": {
    "campaign_id": "5d6a4b27-2f6c-4c5f-9d9e-0bb0f8870d29",
    "campaign_name": "FAQ Search Sponsors",
    "sponsor": "Acme Corp",
    "required_task": "share_feedback",
    "subsidy_amount_cents": 120
  },
  "selected_services": [],
  "selected_task": {
    "required_task": "share_feedback",
    "subsidy_per_call_cents": 120
  },
  "rationale": "Selected the highest-subsidy offer from candidate services."
}
```

**Failure Handling**
- If the purpose is too vague, return a validation error with missing details
- If the budget is insufficient, return a validation error explaining the shortfall
- If target tools cannot be determined, return a validation error

**Error Response Examples (MCP)**
```json
{
  "content": [
    {
      "type": "text",
      "text": "No suitable sponsored services found. Try a more specific purpose or adjust the budget."
    }
  ],
  "_meta": {
    "code": "no_candidate_service",
    "details": {
      "services": [],
      "total_count": 0,
      "message": "No services matched"
    }
  },
  "isError": true
}
```

```json
{
  "content": [
    {
      "type": "text",
      "text": "Budget is below the selected subsidy amount. Increase budget or adjust purpose."
    }
  ],
  "_meta": {
    "code": "budget_too_low",
    "details": {
      "budget_cents": 80,
      "subsidy_per_call_cents": 120
    }
  },
  "isError": true
}
```

```json
{
  "content": [
    {
      "type": "text",
      "text": "Target tools could not be determined. Provide target_tools explicitly."
    }
  ],
  "_meta": {
    "code": "missing_target_tools",
    "details": {
      "service_key": "",
      "offer": {
        "campaign_id": "5d6a4b27-2f6c-4c5f-9d9e-0bb0f8870d29",
        "campaign_name": "FAQ Search Sponsors",
        "sponsor": "Acme Corp",
        "required_task": "share_feedback",
        "subsidy_amount_cents": 120
      },
      "source": "service"
    }
  },
  "isError": true
}
```

**structuredContent Detailed Example**
```json
{
  "campaign_id": "2f6d2c0b-3c7a-4a0a-9a2e-6f2b6b7e8d90",
  "campaign": {
    "id": "2f6d2c0b-3c7a-4a0a-9a2e-6f2b6b7e8d90",
    "name": "Acme Corp Improve AI chat assistance",
    "sponsor": "Acme Corp",
    "sponsor_wallet_address": null,
    "target_roles": ["customer-support", "product-manager"],
    "target_tools": ["faq_search"],
    "required_task": "share_feedback",
    "subsidy_per_call_cents": 120,
    "budget_total_cents": 25000,
    "budget_remaining_cents": 25000,
    "query_urls": ["https://example.com/faq"],
    "active": true,
    "created_at": "2026-02-23T09:00:00Z"
  },
  "selected_service_key": "faq_search",
  "selected_offer": {
    "campaign_id": "5d6a4b27-2f6c-4c5f-9d9e-0bb0f8870d29",
    "campaign_name": "FAQ Search Sponsors",
    "sponsor": "Acme Corp",
    "required_task": "share_feedback",
    "subsidy_amount_cents": 120
  },
  "selected_services": [
    {
      "service_key": "faq_search",
      "display_name": "FAQ Search",
      "reason": "Matches the purpose",
      "offer_count": 2,
      "offers": [
        {
          "campaign_id": "5d6a4b27-2f6c-4c5f-9d9e-0bb0f8870d29",
          "campaign_name": "FAQ Search Sponsors",
          "sponsor": "Acme Corp",
          "required_task": "share_feedback",
          "subsidy_amount_cents": 120
        }
      ]
    }
  ],
  "selected_task": {
    "required_task": "share_feedback",
    "subsidy_per_call_cents": 120
  },
  "rationale": "Selected the highest-subsidy offer from candidate services."
}
```

**Prompt Examples to Invoke the Tool**
- "Create a campaign to improve FAQ answer quality for customer support. Sponsor is Acme Corp, budget is $250."
- "Create a campaign for B2B onboarding improvements. Target roles are product managers, budget is $300."
- "Auto-select the tasks and tools for an AI chat improvement campaign and create it."

### Verification System

**Proof of Action**
The platform validates tool usage through x402 payment success signals. Successful payment settlement serves as cryptographic proof that:
- The tool was actually invoked
- The request was legitimate (not spoofed)
- Payment was processed correctly

**Anti-Abuse Measures**
- Rate limiting and usage caps
- Bot detection and filtering
- Reputation scoring for developers
- Quality filters for sponsors

---

## Monetization

### Revenue Model

**Transaction Fees**
- Platform takes a percentage of each transaction (e.g., 20% take rate)
- Example: Sponsor pays $0.05 per call → Developer receives $0.04 → Platform keeps $0.01
- Competitive with traditional advertising CPC rates ($2-$5 per click)

**Data Access Fees**
- Premium analytics and detailed usage reports
- User attribution and engagement metrics
- Custom data exports and API access
- Requires developer privacy consent

**Verification Services**
- Quality filtering for high-reputation developers
- Bot detection and spam prevention
- Custom verification rules per campaign
- Monthly SaaS subscription model

### Value Proposition

**For Sponsors:**
- Verified user engagement (not just impressions)
- Direct access to AI agent usage patterns
- Lower cost per engagement than traditional ads
- Real-time campaign optimization

**For Developers:**
- Subsidized operational costs
- Potential profit from agent usage
- Access to premium APIs at no cost
- Passive income from agent deployments

---

## Integration Guide

### Sponsored Skill Schema

When integrating a sponsored tool, developers receive a JSON schema containing:

```json
{
  "skill_id": "supersearch_v1",
  "name": "SuperSearch API",
  "sponsor": "Acme Corp",
  "payout_per_call": "0.05",
  "currency": "USDC",
  "mcp_server_url": "https://mcp.payloadexchange.com/supersearch",
  "function_schema": {
    "name": "search",
    "description": "Search the web using SuperSearch",
    "parameters": {
      "type": "object",
      "properties": {
        "query": {
          "type": "string",
          "description": "Search query"
        }
      },
      "required": ["query"]
    }
  },
  "x402_endpoint": "https://api.supersearch.com/v1/search",
  "verification": {
    "method": "x402_payment_success",
    "required_headers": ["X-402-Payment-Token"]
  }
}
```

### Integration Workflow

1. **Discovery**: Browse marketplace and identify sponsored tools
2. **Installation**: Install MCP server or SDK wrapper
3. **Configuration**: Link wallet address and configure agent settings
4. **Deployment**: Deploy agent with sponsored tool integration
5. **Monitoring**: Track usage and earnings through developer dashboard

### MCP Server Implementation

```typescript
import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { ListToolsRequestSchema, CallToolRequestSchema } from "@modelcontextprotocol/sdk/types.js";

const server = new Server({
  name: "supersearch-sponsored",
  version: "1.0.0",
});

// Register available tools
server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: [{
    name: "search",
    description: "Search using SuperSearch (sponsored)",
    inputSchema: {
      type: "object",
      properties: {
        query: { 
          type: "string",
          description: "Search query"
        }
      },
      required: ["query"]
    }
  }]
}));

// Handle tool execution with x402 payment
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { query } = request.params.arguments;
  
  // Obtain payment token from PayloadExchange
  const paymentToken = await getPaymentToken();
  
  // Make API request with x402 headers
  const response = await fetch("https://api.supersearch.com/v1/search", {
    method: "POST",
    headers: {
      "X-402-Payment-Token": paymentToken,
      "Content-Type": "application/json"
    },
    body: JSON.stringify({ query })
  });
  
  // Payment is automatically processed by x402 layer
  const data = await response.json();
  
  return { 
    content: [
      {
        type: "text",
        text: JSON.stringify(data)
      }
    ]
  };
});
```

---

## Getting Started

### For Developers

1. **Wallet Setup**: Configure an EVM-compatible wallet (MetaMask, WalletConnect, etc.)
2. **Account Creation**: Sign up on PayloadExchange and link your wallet
3. **Browse Marketplace**: Explore available sponsored tools and payout rates
4. **Integration**: Install MCP server or SDK for selected tools
5. **Deployment**: Deploy your agent and start earning from usage

### For Sponsors

1. **Account Setup**: Create sponsor account and connect funding wallet
2. **Campaign Creation**: Define campaign parameters, budget, and targeting
3. **Tool Registration**: Register your API endpoint and integration requirements
4. **Monitoring**: Track campaign performance through analytics dashboard
5. **Optimization**: Adjust targeting and budget based on performance data

### For Platform Contributors

1. **Protocol Development**: Contribute to x402 protocol implementation
2. **MCP Server Templates**: Build reference implementations for common use cases
3. **SDK Development**: Create language-specific SDKs and wrappers
4. **Documentation**: Improve integration guides and API references
5. **Testing**: Help test and validate payment flows and verification systems

---

## Vision and Roadmap

PayloadExchange is built on the **x402 protocol**, a payment standard supported by Google, Visa, and Cloudflare. The platform enables just-in-time resource acquisition using stablecoins, eliminating the need for pre-registration between buyers and sellers.

**Core Mission**: Enable an internet where AI agents can operate profitably through sponsored compute, while sponsors gain verified engagement and direct user access—creating a sustainable alternative to traditional advertising models.

**Future Enhancements**:
- Multi-chain payment support
- Advanced targeting and segmentation
- Real-time bidding for tool placement
- Developer reputation and certification system
- Automated campaign optimization

---

*This documentation is open source. Contribute on [GitHub](https://github.com/yourusername/payloadexchange-docs).*
