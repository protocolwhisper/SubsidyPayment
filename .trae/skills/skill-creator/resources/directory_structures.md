# 📁 Directory Structure Reference

Canonical directory layouts for each skill archetype. Use these as starting points and adapt as needed.

---

## Minimal Skill (Any Type)

The absolute minimum for a valid skill:

```
skills/my-skill/
└── SKILL.md              # Required: frontmatter + instructions
```

---

## 🔧 Operator Skill

```
skills/deploy-agent/
├── SKILL.md              # Runbook with deployment steps
├── scripts/
│   ├── deploy.sh         # Main deployment script
│   ├── rollback.sh       # Rollback procedure
│   └── healthcheck.sh    # Post-deploy verification
├── resources/
│   ├── env_reference.md  # Environment variable documentation
│   └── troubleshooting.md
├── templates/
│   └── config.yaml       # Configuration template
└── agents/
    └── openai.yaml       # Platform agent config
```

---

## 📘 Guide Skill

```
skills/hackathon-expert/
├── SKILL.md              # Strategy guide with decision trees
├── resources/
│   ├── decision_matrix.md
│   ├── best_practices.md
│   └── tech_stack.md
└── templates/
    ├── checklist.md
    └── project_plan.md
```

---

## ⚡ Generator Skill

```
skills/blog-writer/
├── SKILL.md              # Writing workflow and style guide
├── templates/
│   ├── tutorial.md       # Tutorial article template
│   ├── troubleshooting.md
│   ├── release.md
│   └── learning_log.md
├── resources/
│   ├── title_patterns.md
│   └── platform_syntax.md
└── examples/
    └── sample_article.md
```

---

## 🔍 Analyst Skill

```
skills/security-auditor/
├── SKILL.md              # Audit methodology
├── resources/
│   ├── vulnerability_db.md
│   ├── severity_guide.md
│   └── evaluation_rubric.md
├── templates/
│   └── audit_report.md
└── scripts/
    └── analyze.sh
```

---

## ✅ Reviewer Skill

```
skills/code-reviewer/
├── SKILL.md              # Review process and standards
├── resources/
│   ├── review_checklist.md
│   ├── severity_levels.md
│   └── style_guide.md
└── templates/
    └── review_report.md
```

---

## 🔀 Hybrid Skill

```
skills/full-stack-builder/
├── SKILL.md              # Multi-phase orchestration
├── phases/
│   ├── 01_planning.md    # Guide phase
│   ├── 02_scaffolding.md # Generator phase
│   ├── 03_review.md      # Reviewer phase
│   └── 04_deployment.md  # Operator phase
├── templates/
│   ├── project_plan.md
│   ├── component.tsx
│   └── config.yaml
├── resources/
│   ├── architecture.md
│   └── tech_stack.md
├── scripts/
│   ├── scaffold.sh
│   └── deploy.sh
└── examples/
    └── sample_project/
        ├── README.md
        └── src/
```

---

## Naming Conventions

| Element | Convention | Example |
|---|---|---|
| Skill directory | `kebab-case` | `skills/solana-deployer` |
| SKILL.md | Always `SKILL.md` (uppercase) | `SKILL.md` |
| Script files | `snake_case.sh` or `kebab-case.sh` | `validate_skill.sh` |
| Template files | `snake_case.md` | `audit_report.md` |
| Resource files | `snake_case.md` | `best_practices.md` |
| Agent configs | `platform_name.yaml` | `openai.yaml` |

---

## Rules of Thumb

1. **Flat over nested** — Don't nest more than 2 levels deep
2. **Fewer files, more content** — Prefer 5 rich files over 20 sparse ones
3. **Predictable names** — A developer should guess filenames correctly
4. **README-free** — SKILL.md IS the README; don't add another
5. **Scripts are optional** — Only add them if they provide real automation value
