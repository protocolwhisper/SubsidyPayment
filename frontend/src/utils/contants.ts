import type { CampaignForm } from "./types";

export const defaultCampaignForm: CampaignForm = {
  name: "",
  sponsor: "",
  target_roles: "developer",
  target_tools: "scraping",
  serviceConfigs: [],
  budget_cents: 500,
  require_human_verification: false
};

export const KPI_OPTIONS = [
  "CPA (Cost per Acquisition)",
  "CPI (Cost per Install)",
  "Cost per Signup",
  "Incremental Conversions",
  "Cost per Qualified Lead"
];

export const LANDING_BACKGROUND_CLASS_NAME = "landing-background-3d";
export const LANDING_BACKGROUND_COLOR = "#0a0a0f";

export const LANDING_CANVAS_CAMERA = { position: [0, 0, 5] as [number, number, number], fov: 50 };
export const LANDING_CANVAS_DPR = [1, 1.5] as [number, number];
export const LANDING_CANVAS_GL = {
  alpha: true,
  antialias: true,
  powerPreference: "low-power" as const
};

export const GET_STARTED_BUTTON_BASE_CLASS = "get-started-3d-btn";
export const GET_STARTED_BUTTON_LABEL = "Get Started";
export const GET_STARTED_BUTTON_ARIA_LABEL = "Get Started";
export const GET_STARTED_BUTTON_LABEL_CLASS = "get-started-3d-label";
export const GET_STARTED_BUTTON_CANVAS_CLASS = "get-started-3d-canvas";

export const GET_STARTED_CANVAS_CAMERA = { position: [0, 0, 2.5] as [number, number, number], fov: 40 };
export const GET_STARTED_CANVAS_DPR = [1, 2] as [number, number];
export const GET_STARTED_CANVAS_GL = { alpha: true, antialias: true };