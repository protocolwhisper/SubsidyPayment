export interface GptSearchResponse {
  services: GptServiceItem[];
  total_count: number;
  message: string;
  applied_filters?: AppliedFilters;
  available_categories?: string[];
}

export interface GptServiceItem {
  service_type: "campaign" | "sponsored_api";
  service_id: string;
  name: string;
  sponsor: string;
  required_task: string | null;
  subsidy_amount_cents: number;
  category: string[];
  active: boolean;
  tags: string[];
  relevance_score: number | null;
}

export interface AppliedFilters {
  budget: number | null;
  intent: string | null;
  category: string | null;
  keyword: string | null;
  preferences_applied: boolean;
}

export interface GptAuthResponse {
  session_token: string;
  user_id: string;
  email: string;
  is_new_user: boolean;
  message: string;
}

export interface GptTaskResponse {
  campaign_id: string;
  campaign_name: string;
  sponsor: string;
  required_task: string;
  task_description: string;
  task_input_format: GptTaskInputFormat;
  already_completed: boolean;
  subsidy_amount_cents: number;
  message: string;
}

export interface GptTaskInputFormat {
  task_type: string;
  required_fields: string[];
  instructions: string;
}

export interface GptCompleteTaskResponse {
  task_completion_id: string;
  campaign_id: string;
  consent_recorded: boolean;
  can_use_service: boolean;
  message: string;
}

export interface GptRunServiceResponse {
  service: string;
  output: string;
  payment_mode: "sponsored" | "user_direct";
  sponsored_by: string | null;
  tx_hash: string | null;
  message: string;
}

export interface ProxyRunServiceResponse {
  service: string;
  output: string;
  payment_mode: "sponsored" | "user_direct";
  sponsored_by: string | null;
  tx_hash: string | null;
}

export interface PaymentRequiredResponse {
  service: string;
  amount_cents: number;
  accepted_header: string;
  payment_required: string;
  message: string;
  next_step: string;
}

export interface GptUserStatusResponse {
  user_id: string;
  email: string;
  completed_tasks: GptCompletedTaskSummary[];
  available_services: GptAvailableService[];
  message: string;
}

export interface GptUserRecordResponse {
  user_id: string;
  email: string;
  history: GptUserRecordEntry[];
  sponsor_summaries: GptUserRecordSponsorSummary[];
  total_subsidy_cents: number;
  message: string;
}

export interface GptUserRecordEntry {
  service: string;
  sponsor: string;
  subsidy_cents: number;
  payment_mode: 'sponsored' | 'user_direct';
  tx_hash: string | null;
  used_at: string;
}

export interface GptUserRecordSponsorSummary {
  sponsor: string;
  services_used: number;
  total_subsidy_cents: number;
}

export interface GptCompletedTaskSummary {
  campaign_id: string;
  campaign_name: string;
  task_name: string;
  completed_at: string;
}

export interface GptAvailableService {
  service: string;
  sponsor: string;
  ready: boolean;
}

export interface GptPreferencesResponse {
  user_id: string;
  preferences: TaskPreference[];
  updated_at: string | null;
  message: string;
}

export interface GptSetPreferencesResponse {
  user_id: string;
  preferences_count: number;
  updated_at: string;
  message: string;
}

export interface TaskPreference {
  task_type: string;
  level: "preferred" | "neutral" | "avoided";
}

export interface BackendErrorResponse {
  error: {
    code: string;
    message: string;
    details?: unknown;
  };
}

export interface SearchServicesParams {
  q?: string;
  category?: string;
  max_budget_cents?: number;
  intent?: string;
  session_token?: string;
}

export interface AuthenticateUserParams {
  email: string;
  region: string;
  roles?: string[];
  tools_used?: string[];
}

export interface GetTaskDetailsParams {
  campaign_id: string;
  session_token: string;
}

export interface GptConsentInput {
  data_sharing_agreed: boolean;
  purpose_acknowledged: boolean;
  contact_permission: boolean;
}

export interface CompleteTaskInput {
  campaign_id: string;
  session_token: string;
  task_name: string;
  details?: string;
  consent: GptConsentInput;
}

export interface RunServiceInput {
  service: string;
  session_token: string;
  input: string;
}

export interface ProxyRunInput {
  user_id: string;
  input: string;
}

export interface GetUserStatusParams {
  session_token: string;
}

export interface GetUserRecordParams {
  session_token: string;
}

export interface GetPreferencesParams {
  session_token: string;
}

export interface SetPreferencesInput {
  session_token: string;
  preferences: TaskPreference[];
}
