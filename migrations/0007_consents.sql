create table if not exists consents (
  id uuid primary key,
  user_id uuid not null references users(id) on delete cascade,
  campaign_id uuid not null references campaigns(id) on delete cascade,
  consent_type text not null check (consent_type in ('data_sharing', 'contact', 'retention')),
  granted boolean not null,
  purpose text,
  retention_days integer,
  created_at timestamptz not null default now()
);

create index if not exists consents_user_campaign_idx
  on consents(user_id, campaign_id);

create index if not exists consents_user_id_idx
  on consents(user_id);
