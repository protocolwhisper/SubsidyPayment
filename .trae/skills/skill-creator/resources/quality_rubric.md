# 📊 Skill Quality Rubric

Score each dimension 1-5. A production-ready skill must score **≥ 4 on every dimension**.

---

## Scoring Scale

| Score | Meaning |
|---|---|
| **5** | Exceptional — could be a reference example |
| **4** | Strong — meets all expectations, minor polish possible |
| **3** | Adequate — works but has gaps |
| **2** | Weak — missing key elements |
| **1** | Missing — dimension not addressed |

---

## 8 Dimensions

### 1. Clarity (Can the agent understand the skill immediately?)

| Score | Criteria |
|---|---|
| 5 | Crystal clear: no ambiguity, every term defined |
| 4 | Clear with minor assumptions that are reasonable |
| 3 | Some sections require re-reading to understand |
| 2 | Significant ambiguity in key instructions |
| 1 | The agent cannot determine what to do |

**Check**: Read the SKILL.md cold. Can you follow it on the first pass?

### 2. Completeness (Does the skill cover all necessary steps?)

| Score | Criteria |
|---|---|
| 5 | Every step from start to finish, including edge cases |
| 4 | All main steps covered, minor edge cases omitted |
| 3 | Main happy path covered, alt paths missing |
| 2 | Key steps missing or implied |
| 1 | Only a partial workflow |

**Check**: Walk through the workflow mentally. Are there gaps where you'd need to guess?

### 3. Actionability (Are the instructions executable?)

| Score | Criteria |
|---|---|
| 5 | Every step has exact commands, inputs, and expected outputs |
| 4 | Most steps have exact commands, some have descriptions |
| 3 | Mix of exact commands and vague instructions |
| 2 | Mostly descriptive ("configure the server") |
| 1 | Abstract guidance only |

**Check**: Could you copy-paste the commands and have them work?

### 4. Reusability (Can the skill be used across projects?)

| Score | Criteria |
|---|---|
| 5 | Parameterized, project-agnostic, composable |
| 4 | Works across most similar projects with minor tweaks |
| 3 | Works for a category of projects |
| 2 | Somewhat project-specific |
| 1 | Hardcoded to a single project |

**Check**: Would this skill work if dropped into a different repo?

### 5. Resource Quality (Are templates/resources useful and well-structured?)

| Score | Criteria |
|---|---|
| 5 | Rich templates with coaching comments, tables, and examples |
| 4 | Good templates that are easy to fill in |
| 3 | Basic templates that need interpretation |
| 2 | Minimal resources, mostly placeholders |
| 1 | No supporting resources |

**Check**: Could a junior developer use the templates without additional guidance?

### 6. Examples (Are there enough worked examples?)

| Score | Criteria |
|---|---|
| 5 | 3+ examples covering happy path, edge cases, and errors |
| 4 | 2 examples covering happy path and one variant |
| 3 | 1 complete example |
| 2 | Partial example or pseudocode only |
| 1 | No examples |

**Check**: Can you learn how to use the skill just from the examples?

### 7. Error Handling (Does the skill anticipate failures?)

| Score | Criteria |
|---|---|
| 5 | Every step has "If X fails, do Y" blocks |
| 4 | Common failures covered with recovery steps |
| 3 | Some error cases mentioned |
| 2 | Errors acknowledged but no recovery guidance |
| 1 | No error handling |

**Check**: What happens when step 3 fails? Does the skill tell you?

### 8. Delivery Format (Is the package well-organized?)

| Score | Criteria |
|---|---|
| 5 | Clean directory structure, self-validating, with agent configs |
| 4 | Good structure with all required files |
| 3 | Reasonable structure, some files misplaced |
| 2 | Flat structure, hard to navigate |
| 1 | Single file dump |

**Check**: Does `validate_skill.sh` pass? Is the directory tree logical?

---

## Scorecard Template

```markdown
| Dimension        | Score | Notes |
|------------------|-------|-------|
| Clarity          |   /5  |       |
| Completeness     |   /5  |       |
| Actionability    |   /5  |       |
| Reusability      |   /5  |       |
| Resource Quality |   /5  |       |
| Examples         |   /5  |       |
| Error Handling   |   /5  |       |
| Delivery Format  |   /5  |       |
| **Total**        |  /40  |       |
```

**Verdict**:
- **36-40**: 🏆 Ship it — exceptional quality
- **32-35**: ✅ Ready — minor polish optional
- **24-31**: ⚠️ Needs work — address gaps before delivery
- **< 24**: ❌ Not ready — significant rework required
