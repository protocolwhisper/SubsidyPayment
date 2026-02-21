create table if not exists zkpassport_verifications (
  id uuid primary key,
  verification_token uuid not null unique,
  campaign_id uuid not null references campaigns(id) on delete cascade,
  user_id uuid not null references users(id) on delete cascade,
  status text not null check (status in ('pending', 'verified', 'failed', 'expired')),
  min_age integer not null,
  allowed_country_labels text[] not null,
  consent_data_sharing_agreed boolean not null default false,
  consent_purpose_acknowledged boolean not null default false,
  consent_contact_permission boolean not null default false,
  proofs jsonb,
  query_result jsonb,
  unique_identifier_hash text,
  verification_errors jsonb,
  failure_reason text,
  created_at timestamptz not null default now(),
  expires_at timestamptz not null,
  completed_at timestamptz
);

create index if not exists zkpassport_verifications_campaign_user_idx
  on zkpassport_verifications(campaign_id, user_id);

create index if not exists zkpassport_verifications_status_expires_idx
  on zkpassport_verifications(status, expires_at);
