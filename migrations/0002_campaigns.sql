create table if not exists campaigns (
  id uuid primary key,
  name text not null,
  sponsor text not null,
  target_roles text[] not null default '{}',
  target_tools text[] not null default '{}',
  required_task text not null,
  subsidy_per_call_cents bigint not null,
  budget_total_cents bigint not null,
  budget_remaining_cents bigint not null,
  query_urls text[] not null default '{}',
  active boolean not null default true,
  created_at timestamptz not null default now()
);

create index if not exists campaigns_active_created_idx
  on campaigns(active, created_at desc);
