# 🧬 Skill Design Patterns

Six proven archetypes for structuring agent skills. Choose the one that best matches your use case, or combine them into a Hybrid.

---

## 1. 🔧 Operator

**Purpose**: Run infrastructure, execute deployments, manage APIs, operate systems.

**When to Use**:
- Deploying applications to cloud/blockchain
- Managing CI/CD pipelines
- Operating APIs with specific auth flows
- Database migrations and maintenance

**Key Features**:
- Step-by-step runbooks with exact commands
- Environment variable specifications
- Health check and rollback procedures
- Idempotent operations (safe to re-run)

**Typical Resources**:
- `scripts/deploy.sh`, `scripts/rollback.sh`
- `resources/env_var_reference.md`
- `templates/config_template.yaml`

**Example Skills**: "Kubernetes Deployer", "Solana Program Publisher", "AWS Lambda Manager"

---

## 2. 📘 Guide

**Purpose**: Teach strategies, provide best practices, mentor through complex decisions.

**When to Use**:
- Hackathon preparation and strategy
- Learning new technologies
- Architecture decision records
- Writing style guides

**Key Features**:
- Decision trees ("If X, then do Y")
- Checklists and rubrics
- Tiered advice (beginner → advanced)
- Reference tables and comparison matrices

**Typical Resources**:
- `resources/decision_matrix.md`
- `resources/best_practices.md`
- `templates/checklist_template.md`

**Example Skills**: "ETH Global Hackathon Expert", "System Design Advisor", "Career Growth Guide"

---

## 3. ⚡ Generator

**Purpose**: Create code, content, templates, or artifacts from specifications.

**When to Use**:
- Scaffolding new projects
- Generating boilerplate code
- Creating documentation from code
- Producing marketing content

**Key Features**:
- Template library with rich placeholders
- Style guides and conventions
- Output format specifications
- Validation rules for generated content

**Typical Resources**:
- `templates/` (many templates)
- `resources/style_guide.md`
- `examples/` (sample outputs)
- `scripts/generate.sh`

**Example Skills**: "Technical Blog Writer", "Smart Contract Scaffolder", "API Documentation Generator"

---

## 4. 🔍 Analyst

**Purpose**: Research, audit, evaluate, and report on codebases or systems.

**When to Use**:
- Security auditing
- Performance analysis
- Dependency evaluation
- Market research

**Key Features**:
- Scoring rubrics and evaluation matrices
- Report templates with structured sections
- Comparison frameworks
- Data collection checklists

**Typical Resources**:
- `resources/evaluation_rubric.md`
- `templates/report_template.md`
- `resources/benchmark_data.md`
- `scripts/analyze.sh`

**Example Skills**: "Smart Contract Auditor", "Performance Benchmarker", "Tech Stack Evaluator"

---

## 5. ✅ Reviewer

**Purpose**: Review code, documents, or processes against quality standards.

**When to Use**:
- Code review automation
- Documentation quality checks
- Compliance verification
- Pull request analysis

**Key Features**:
- Checklist-driven review process
- Severity classification (critical/major/minor/nit)
- Inline feedback formatting
- Summary report generation

**Typical Resources**:
- `resources/review_checklist.md`
- `resources/severity_guide.md`
- `templates/review_report.md`

**Example Skills**: "Solidity Code Reviewer", "API Design Reviewer", "Accessibility Checker"

---

## 6. 🔀 Hybrid

**Purpose**: Complex multi-phase projects requiring multiple skill types.

**When to Use**:
- End-to-end project delivery (Guide + Generator + Operator)
- Research → Implementation flows (Analyst + Generator)
- Review → Fix → Deploy cycles (Reviewer + Generator + Operator)

**Key Features**:
- Phased workflow with clear handoff points
- Sub-skills for each phase
- Progress tracking across phases
- Composable with existing skills

**Design Principle**: A Hybrid is not a monolith. Each phase should reference or embed a simpler archetype. Break it down:

```
Hybrid Skill
├── Phase 1: Guide (strategy & planning)
├── Phase 2: Generator (code scaffolding)
├── Phase 3: Reviewer (quality check)
└── Phase 4: Operator (deployment)
```

**Example Skills**: "Full-Stack App Builder", "Hackathon Sprint Partner", "Migration Assistant"

---

## 📊 Decision Matrix

| Question | → Archetype |
|---|---|
| Does the user need to **run/deploy** something? | Operator |
| Does the user need to **learn/decide** something? | Guide |
| Does the user need to **create/generate** something? | Generator |
| Does the user need to **understand/evaluate** something? | Analyst |
| Does the user need to **check/validate** something? | Reviewer |
| Does the user need **multiple of the above**? | Hybrid |
