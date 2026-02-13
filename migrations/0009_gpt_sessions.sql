create extension if not exists pgcrypto;

create table if not exists gpt_sessions (
  token uuid primary key default gen_random_uuid(),
  user_id uuid not null references users(id) on delete cascade,
  created_at timestamptz not null default now(),
  expires_at timestamptz not null default now() + interval '30 days'
);

create index if not exists gpt_sessions_user_id_idx
  on gpt_sessions(user_id);

create index if not exists gpt_sessions_expires_at_idx
  on gpt_sessions(expires_at);
