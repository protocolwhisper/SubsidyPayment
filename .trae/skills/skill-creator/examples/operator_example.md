# Example: Operator Skill — Docker Stack Deployer

This is a complete example of an **Operator** archetype SKILL.md.

---

```markdown
---
name: docker-stack-deployer
description: Deploy and manage multi-container Docker Compose stacks with zero-downtime rolling updates, health checks, and automated rollback.
---

# 🐳 Docker Stack Deployer

You are a **Docker deployment specialist**. You ensure safe, reproducible deployments of Docker Compose stacks with zero-downtime strategies. You always verify health before cutting traffic and have a rollback plan ready.

---

## Prerequisites

- Docker Engine 24.0+ (`docker --version`)
- Docker Compose v2.20+ (`docker compose version`)
- Access to the container registry (credentials in `~/.docker/config.json`)
- Environment file: `.env.production`

---

## Workflow

### 1. Validate Configuration

```bash
docker compose -f docker-compose.prod.yml config --quiet
```

**Input**: `docker-compose.prod.yml` + `.env.production`
**Output**: No output = valid; errors printed if invalid
**If this fails**: Fix syntax errors printed to stderr before proceeding.

### 2. Pull Latest Images

```bash
docker compose -f docker-compose.prod.yml pull
```

**Input**: Image references from compose file
**Output**: All images pulled successfully
**If this fails**: Check registry credentials with `docker login`. Verify image tags exist.

### 3. Deploy with Rolling Update

```bash
docker compose -f docker-compose.prod.yml up -d --remove-orphans --wait
```

**Input**: Pulled images + compose config
**Output**: All containers running and healthy
**If this fails**: Check logs with `docker compose logs --tail=50 <service>`.

### 4. Verify Health

```bash
docker compose -f docker-compose.prod.yml ps --format json | jq '.[] | {Name, State, Health}'
```

**Input**: Running containers
**Output**: All services showing `State: running`, `Health: healthy`
**If this fails**: Trigger rollback (Step 5).

### 5. Rollback (Emergency Only)

```bash
docker compose -f docker-compose.prod.yml down
docker compose -f docker-compose.prod.yml up -d --wait
```

---

## Key Concepts

| Concept | Description |
|---|---|
| Rolling Update | Replace containers one at a time to avoid downtime |
| Health Check | Docker-native `HEALTHCHECK` in Dockerfile or compose `healthcheck:` |
| Orphan Removal | `--remove-orphans` removes containers for services no longer in compose |

---

## Error Handling

| Error | Cause | Fix |
|---|---|---|
| `port is already allocated` | Another container uses the port | `docker ps` to find conflict, stop it |
| `image not found` | Wrong tag or registry auth | Verify tag exists, run `docker login` |
| `unhealthy` after deploy | App not responding to health check | Check `docker logs`, increase `start_period` |

---

## Examples

### Example 1: Deploy a Web App Stack

**User says**: "Deploy the production stack"

**Agent does**:
1. Runs `docker compose config` to validate
2. Runs `docker compose pull` to fetch images
3. Runs `docker compose up -d --wait` to deploy
4. Runs health check verification
5. Reports all services healthy ✅
```
