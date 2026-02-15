CREATE TABLE IF NOT EXISTS user_task_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    task_type TEXT NOT NULL,
    level TEXT NOT NULL CHECK (level IN ('preferred', 'neutral', 'avoided')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, task_type)
);

CREATE INDEX IF NOT EXISTS user_task_preferences_user_id_idx
    ON user_task_preferences(user_id);
