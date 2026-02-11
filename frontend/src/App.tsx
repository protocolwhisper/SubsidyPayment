import { FormEvent, useEffect, useMemo, useState } from "react";
import logoImage from "./logo.jpg";

// Extend Window interface for MetaMask
declare global {
  interface Window {
    ethereum?: {
      request: (args: { method: string; params?: any[] }) => Promise<any>;
      isMetaMask?: boolean;
      selectedAddress?: string;
    };
  }
}

type Campaign = {
  id: string;
  name: string;
  sponsor: string;
  target_roles: string[];
  target_tools: string[];
  required_task: string;
  subsidy_per_call_cents: number;
  budget_total_cents: number;
  budget_remaining_cents: number;
  query_urls: string[];
  active: boolean;
  created_at: string;
};

type Profile = {
  id: string;
  email: string;
  region: string;
  roles: string[];
  tools_used: string[];
  created_at: string;
};

type CreatorSummary = {
  total_events: number;
  success_events: number;
  success_rate: number;
  per_skill: Array<{
    skill_name: string;
    total_events: number;
    success_events: number;
    avg_duration_ms: number | null;
    last_seen_at: string;
  }>;
};

type CampaignForm = {
  name: string;
  sponsor: string;
  target_roles: string;
  target_tools: string;
  required_task: string;
  subsidy_per_call_cents: number;
  budget_cents: number;
};

type SponsoredApi = {
  id: string;
  name: string;
  sponsor: string;
  description: string | null;
  upstream_url: string;
  upstream_method: string;
  price_cents: number;
  budget_total_cents: number;
  budget_remaining_cents: number;
  active: boolean;
  created_at: string;
};

type PaymentRequired = {
  service: string;
  amount_cents: number;
  accepted_header: string;
  payment_required: string;
  message: string;
  next_step: string;
};

type ServiceRunResponse = {
  service: string;
  output: string;
  payment_mode: string;
  sponsored_by: string | null;
  tx_hash: string | null;
};

type SponsoredApiRunResponse = {
  api_id: string;
  payment_mode: string;
  sponsored_by: string | null;
  tx_hash: string | null;
  upstream_status: number;
  upstream_body: string;
};

const defaultCampaignForm: CampaignForm = {
  name: "",
  sponsor: "",
  target_roles: "developer",
  target_tools: "scraping",
  required_task: "signup_sponsor",
  subsidy_per_call_cents: 5,
  budget_cents: 500
};

type ServiceCategory = { name: string; services: string[] };
const SERVICE_CATEGORIES: ServiceCategory[] = [
  { name: "DeFi / Web3", services: ["Uniswap", "Aave", "OpenSea", "Lido Finance", "Compound", "Chainlink"] },
  { name: "AI Services", services: ["Claude (Anthropic)", "OpenAI API", "Hugging Face", "Replicate", "Midjourney API"] },
  { name: "API / Data", services: ["CoinGecko", "Alchemy", "The Graph", "Moralis", "Infura"] },
  { name: "Developer Tools", services: ["GitHub Copilot", "Vercel", "Supabase", "Neon (Postgres)", "Render", "Railway"] }
];
const KPI_OPTIONS = [
  "CPA (Cost per Acquisition)",
  "CPI (Cost per Install)",
  "Cost per Signup",
  "Incremental Conversions",
  "Cost per Qualified Lead"
];

function App() {
  const [campaigns, setCampaigns] = useState<Campaign[]>([]);
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [creator, setCreator] = useState<CreatorSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [createLoading, setCreateLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [form, setForm] = useState<CampaignForm>(defaultCampaignForm);
  const [selectedTab, setSelectedTab] = useState("All");
  const [darkMode, setDarkMode] = useState(() => {
    const saved = localStorage.getItem("darkMode");
    return saved ? JSON.parse(saved) : false;
  });
  const [isLoggedIn, setIsLoggedIn] = useState(false); // Start logged out (public dashboard)
  const [showProfile, setShowProfile] = useState(false);
  const [currentView, setCurrentView] = useState<"landing" | "signup" | "dashboard" | "create-campaign" | "login" | "caller">("landing");
  const [loginForm, setLoginForm] = useState({ email: "", password: "" });
  const [signupForm, setSignupForm] = useState({ email: "", company: "", password: "", confirmPassword: "" });
  const [sponsoredApis, setSponsoredApis] = useState<SponsoredApi[]>([]);
  const [callerLoading, setCallerLoading] = useState(false);
  const [callerResult, setCallerResult] = useState<any>(null);
  const [callerError, setCallerError] = useState<string | null>(null);
  const [callerForm, setCallerForm] = useState({
    callType: "proxy" as "proxy" | "tool" | "sponsored-api",
    service: "",
    apiId: "",
    input: "",
    userId: ""
  });
  const [paymentRequired, setPaymentRequired] = useState<PaymentRequired | null>(null);
  const [selectedServices, setSelectedServices] = useState<string[]>([]);
  const [selectedKpi, setSelectedKpi] = useState("");
  const [showServiceDropdown, setShowServiceDropdown] = useState(false);

  async function fetchJson<T>(path: string, init?: RequestInit): Promise<T> {
    // 環境変数からAPIのベースURLを取得（開発環境では /api、本番環境では環境変数から）
    const apiBaseUrl = import.meta.env.VITE_API_URL || '/api';
    const response = await fetch(`${apiBaseUrl}${path}`, {
      ...init,
      headers: {
        "content-type": "application/json",
        ...(init?.headers ?? {})
      }
    });

    if (!response.ok) {
      const message = await response.text();
      throw new Error(message || `Request failed (${response.status})`);
    }

    return response.json() as Promise<T>;
  }

  async function loadDashboard(silent = false) {
    setLoading(true);
    if (!silent) {
    setError(null);
    }

    try {
      const [campaignData, profileData, creatorData] = await Promise.all([
        fetchJson<Campaign[]>("/campaigns", { method: "GET" }),
        fetchJson<Profile[]>("/profiles", { method: "GET" }),
        fetchJson<CreatorSummary>("/creator/metrics", { method: "GET" })
      ]);
      setCampaigns(campaignData);
      setProfiles(profileData);
      setCreator(creatorData);
    } catch (err) {
      // Only show error if not silent mode (for user-initiated actions)
      if (!silent) {
      setError(err instanceof Error ? err.message : "Unknown error");
      } else {
        // Silent mode: just set empty data, don't show error
        setCampaigns([]);
        setProfiles([]);
        setCreator(null);
      }
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void loadDashboard(true); // Silent mode - don't show errors on initial load
    void loadSponsoredApis();
  }, []);

  async function loadSponsoredApis() {
    try {
      const apis = await fetchJson<SponsoredApi[]>("/sponsored-apis", { method: "GET" });
      setSponsoredApis(apis);
    } catch (err) {
      // Silent fail
    }
  }

  useEffect(() => {
    localStorage.setItem("darkMode", JSON.stringify(darkMode));
    document.documentElement.setAttribute("data-theme", darkMode ? "dark" : "light");
  }, [darkMode]);

  useEffect(() => {
    // Save login state to localStorage when it changes
    localStorage.setItem("isLoggedIn", JSON.stringify(isLoggedIn));
  }, [isLoggedIn]);

  const toggleDarkMode = () => {
    setDarkMode((prev: boolean) => !prev);
  };

  const handleLogin = (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    // Simple login - in production, this would call an API
    if (loginForm.email && loginForm.password) {
      setIsLoggedIn(true);
      localStorage.setItem("isLoggedIn", "true");
      setShowProfile(true);
      setLoginForm({ email: "", password: "" });
      // If we were trying to create a campaign, go there after login
      if (currentView === "login") {
        setCurrentView("create-campaign");
      } else {
        setCurrentView("dashboard");
      }
    }
  };

  const handleLogout = () => {
    setIsLoggedIn(false);
    localStorage.setItem("isLoggedIn", "false");
    setShowProfile(false);
    setCurrentView("landing"); // Show landing page after logout
  };

  const handleWalletConnect = async () => {
    setError(null);
    
    try {
      // Check if MetaMask is installed
      if (typeof window.ethereum === "undefined") {
        setError("Please install MetaMask or another Web3 wallet to continue");
        return;
      }

      // Request account access
      const accounts = await window.ethereum.request({
        method: "eth_requestAccounts"
      });

      if (accounts.length === 0) {
        setError("No wallet accounts found. Please connect your wallet.");
        return;
      }

      // Get the connected address
      const address = accounts[0];
      console.log("Connected wallet:", address);

      // Create a message to sign for authentication
      const message = `Sign in to SubsidyPayment\n\nWallet: ${address}\nTimestamp: ${Date.now()}`;
      
      // Convert message to hex (browser-compatible)
      const messageHex = "0x" + Array.from(new TextEncoder().encode(message))
        .map(b => b.toString(16).padStart(2, "0"))
        .join("");
      
      // Request signature for authentication
      const signature = await window.ethereum.request({
        method: "personal_sign",
        params: [messageHex, address]
      });

      if (!signature) {
        setError("Signature required for authentication. Please sign the message.");
        return;
      }

      console.log("Signature received:", signature);

      // Now sign in after successful wallet connection AND signature
      setIsLoggedIn(true);
      localStorage.setItem("isLoggedIn", "true");
      localStorage.setItem("walletAddress", address);
      localStorage.setItem("walletSignature", signature);
      setShowProfile(true);
      
      // If we were trying to create a campaign, go there after login
      if (currentView === "login") {
        setCurrentView("create-campaign");
      } else {
        setCurrentView("dashboard");
      }
      
      // Load dashboard data silently in background (don't show errors)
      void loadDashboard(true);
    } catch (err: any) {
      // Handle user rejection or other errors
      if (err.code === 4001) {
        setError("Request rejected. Please connect and sign to continue.");
      } else if (err.code === -32602) {
        setError("Invalid signature request. Please try again.");
      } else {
        setError(err.message || "Failed to connect wallet. Please try again.");
      }
    }
  };

  const handleBack = () => {
    setCurrentView("landing");
  };

  const totals = useMemo(() => {
    const activeCampaigns = campaigns.filter((item) => item.active).length;
    const subsidyCents = campaigns.reduce(
      (acc, item) => acc + item.subsidy_per_call_cents,
      0
    );
    const budgetCents = campaigns.reduce(
      (acc, item) => acc + item.budget_remaining_cents,
      0
    );
    const totalPayouts = campaigns.reduce(
      (acc, item) => acc + item.budget_total_cents - item.budget_remaining_cents,
      0
    );

    return {
      activeCampaigns,
      subsidyDollars: (subsidyCents / 100).toFixed(2),
      budgetDollars: (budgetCents / 100).toFixed(2),
      profiles: profiles.length,
      creatorSuccessRate: ((creator?.success_rate ?? 0) * 100).toFixed(1),
      totalPayouts: (totalPayouts / 100).toFixed(2)
    };
  }, [campaigns, profiles.length, creator?.success_rate]);

  async function onCreateCampaign(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setCreateLoading(true);
    setError(null);

    try {
      // Log new fields to console (not sent to API yet)
      console.log("Selected Services:", selectedServices);
      console.log("Selected KPI:", selectedKpi);
      await fetchJson<Campaign>("/campaigns", {
        method: "POST",
        body: JSON.stringify({
          ...form,
          target_roles: splitCsv(form.target_roles),
          target_tools: splitCsv(form.target_tools)
        })
      });
      setForm(defaultCampaignForm);
      setSelectedServices([]);
      setSelectedKpi("");
      // Go back to dashboard after successful creation
      setCurrentView("dashboard");
      // Reload dashboard data silently
      void loadDashboard(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
    } finally {
      setCreateLoading(false);
    }
  }

  async function handleApiCall(paymentSignature?: string) {
    setCallerLoading(true);
    setCallerError(null);
    setCallerResult(null);
    setPaymentRequired(null);

    try {
      const userId = callerForm.userId || localStorage.getItem("walletAddress") || "00000000-0000-0000-0000-000000000000";
      let path = "";
      let body: any = {};

      if (callerForm.callType === "sponsored-api") {
        if (!callerForm.apiId) {
          throw new Error("Please select a sponsored API");
        }
        path = `/sponsored-apis/${callerForm.apiId}/run`;
        body = {
          caller: userId,
          input: callerForm.input ? JSON.parse(callerForm.input) : {}
        };
      } else {
        if (!callerForm.service) {
          throw new Error("Please enter a service name");
        }
        path = `/${callerForm.callType}/${callerForm.service}/run`;
        body = {
          user_id: userId,
          input: callerForm.input || ""
        };
      }

      const headers: Record<string, string> = {
        "content-type": "application/json"
      };

      if (paymentSignature) {
        headers["payment-signature"] = paymentSignature;
      }

      // 環境変数からAPIのベースURLを取得
      const apiBaseUrl = import.meta.env.VITE_API_URL || '/api';
      const response = await fetch(`${apiBaseUrl}${path}`, {
        method: "POST",
        headers,
        body: JSON.stringify(body)
      });

      if (response.status === 402) {
        // Payment required
        const paymentData = await response.json();
        setPaymentRequired(paymentData);
        setCallerError("Payment required to access this service");
        return;
      }

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(errorText || `Request failed (${response.status})`);
      }

      const result = await response.json();
      setCallerResult(result);
      setPaymentRequired(null);
    } catch (err) {
      setCallerError(err instanceof Error ? err.message : "Unknown error");
    } finally {
      setCallerLoading(false);
    }
  }

  async function handlePayment() {
    if (!paymentRequired) return;

    try {
      // Decode payment requirement
      const paymentReqBase64 = paymentRequired.payment_required;
      const paymentReqJson = atob(paymentReqBase64);
      const paymentReqs = JSON.parse(paymentReqJson);
      const paymentReq = paymentReqs[0]; // X402PaymentRequirement

      if (typeof window.ethereum === "undefined") {
        setCallerError("Please install MetaMask or another Web3 wallet to make payments");
        return;
      }

      // Request account access
      const accounts = await window.ethereum.request({
        method: "eth_requestAccounts"
      });

      if (accounts.length === 0) {
        setCallerError("No wallet accounts found");
        return;
      }

      const address = accounts[0];

      // Create payment signature using X402 protocol
      // For now, we'll use a simplified approach - in production, you'd use the X402 SDK
      const message = `Pay ${paymentReq.maxAmountRequired} ${paymentReq.asset} to ${paymentReq.payTo} for ${paymentReq.description}`;
      const messageHex = "0x" + Array.from(new TextEncoder().encode(message))
        .map(b => b.toString(16).padStart(2, "0"))
        .join("");

      const signature = await window.ethereum.request({
        method: "personal_sign",
        params: [messageHex, address]
      });

      if (!signature) {
        setCallerError("Payment signature required");
        return;
      }

      // Retry API call with payment signature
      await handleApiCall(signature);
    } catch (err: any) {
      if (err.code === 4001) {
        setCallerError("Payment request rejected");
      } else {
        setCallerError(err.message || "Failed to process payment");
      }
    }
  }

  // Task breakdown colors for stacked bar
  const taskBreakdownColors = ["#4A9EFF", "#79F8C6", "#F59E0B", "#EF4444", "#8B5CF6"];

  return (
    <div className="dashboard">
      <header className="header">
        <div className="header-left">
          <div className="logo" onClick={() => setCurrentView(isLoggedIn ? "dashboard" : "landing")} style={{ cursor: "pointer" }}>
            <img src={logoImage} alt="SubsidyPayment" className="logo-icon" />
            <span className="logo-text">SubsidyPayment</span>
          </div>
          {!["landing", "login", "signup"].includes(currentView) && (
            <nav className="header-nav-tabs">
              <button className={`nav-tab ${currentView === "dashboard" ? "active" : ""}`} onClick={() => setCurrentView("dashboard")}>Dashboard</button>
              <button className={`nav-tab ${currentView === "create-campaign" ? "active" : ""}`} onClick={() => { if (isLoggedIn) setCurrentView("create-campaign"); else setCurrentView("login"); }}>Create Campaign</button>
              <button className={`nav-tab ${currentView === "caller" ? "active" : ""}`} onClick={() => setCurrentView("caller")}>API Caller</button>
            </nav>
          )}
        </div>
        <div className="header-right">
          <button className="icon-btn">
            <span className="notification-dot"></span>
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"></path>
              <path d="M13.73 21a2 2 0 0 1-3.46 0"></path>
            </svg>
          </button>
          <div className="date-badge">
            <span>Mon, Feb 9</span>
            <span className="badge">12</span>
          </div>
          <button className="icon-btn">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="11" cy="11" r="8"></circle>
              <path d="m21 21-4.35-4.35"></path>
            </svg>
          </button>
          <button className="icon-btn" onClick={toggleDarkMode} title="Toggle dark mode">
            {darkMode ? (
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <circle cx="12" cy="12" r="5"></circle>
                <line x1="12" y1="1" x2="12" y2="3"></line>
                <line x1="12" y1="21" x2="12" y2="23"></line>
                <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
                <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
                <line x1="1" y1="12" x2="3" y2="12"></line>
                <line x1="21" y1="12" x2="23" y2="12"></line>
                <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
                <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
              </svg>
            ) : (
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
              </svg>
            )}
          </button>
          <button 
            className="avatar" 
            onClick={() => {
              if (isLoggedIn) {
                setShowProfile(!showProfile);
              } else {
                setCurrentView("login");
              }
            }}
            title={isLoggedIn ? "Profile" : "Login"}
          >
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
              <circle cx="12" cy="7" r="4"></circle>
            </svg>
          </button>
          {isLoggedIn && (
            <button 
              className="logout-btn" 
              onClick={handleLogout}
              title="Logout"
            >
              Logout
            </button>
          )}
        </div>
      </header>

      {currentView === "landing" ? (
        /* Landing Page */
        <main className="landing-page">
          <section className="lp-hero">
            <img src={logoImage} alt="SubsidyPayment" className="lp-hero-logo" />
            <h1 className="lp-hero-title">SubsidyPayment</h1>
            <p className="lp-hero-subtitle">Sponsor API calls for developers. Track performance. Pay only for results.</p>
            <div className="lp-hero-cta">
              <button className="primary-btn-large" onClick={() => setCurrentView("signup")}>Get Started</button>
              <button className="ghost-btn-large" onClick={() => setCurrentView("login")}>Sign In</button>
            </div>
          </section>

          <section className="lp-features">
            <div className="lp-feature-card">
              <div className="lp-feature-icon">
                <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="8.5" cy="7" r="4"/><polyline points="17 11 19 13 23 9"/></svg>
              </div>
              <h3>For Developers</h3>
              <p>Access premium APIs at zero cost through sponsored campaigns. Focus on building, not billing.</p>
            </div>
            <div className="lp-feature-card">
              <div className="lp-feature-icon">
                <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="1" y="4" width="22" height="16" rx="2" ry="2"/><line x1="1" y1="10" x2="23" y2="10"/></svg>
              </div>
              <h3>For Sponsors</h3>
              <p>Reach targeted developer segments with subsidized API access. Pay only for completed tasks.</p>
            </div>
            <div className="lp-feature-card">
              <div className="lp-feature-icon">
                <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>
              </div>
              <h3>Seamless Payments</h3>
              <p>Blockchain-powered payments with X402 protocol. Transparent, instant, and verifiable.</p>
            </div>
          </section>

          <section className="lp-stats">
            <div className="lp-stat">
              <span className="lp-stat-value">$12,450</span>
              <span className="lp-stat-label">Active Subsidies</span>
            </div>
            <div className="lp-stat">
              <span className="lp-stat-value">1,247</span>
              <span className="lp-stat-label">Developers</span>
            </div>
            <div className="lp-stat">
              <span className="lp-stat-value">96.2%</span>
              <span className="lp-stat-label">Completion Rate</span>
            </div>
          </section>

          <section className="lp-final-cta">
            <h2>Ready to get started?</h2>
            <p>Create your free account and launch your first campaign in minutes.</p>
            <button className="primary-btn-large" onClick={() => setCurrentView("signup")}>Create Free Account</button>
          </section>
        </main>
      ) : currentView === "signup" ? (
        /* Signup Page */
        <div className="login-page">
          <button
            className="back-button"
            onClick={() => setCurrentView("landing")}
            title="Go back"
          >
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M19 12H5"></path>
              <path d="M12 19l-7-7 7-7"></path>
            </svg>
            Back
          </button>
          <div className="login-container">
            <div className="login-card">
              <div className="login-header">
                <div className="login-logo" onClick={() => setCurrentView("landing")}>
                  <img src={logoImage} alt="SubsidyPayment" className="logo-icon-large" />
                  <h1>SubsidyPayment</h1>
                </div>
                <p className="login-subtitle">Create your account</p>
              </div>

              <form className="login-form" onSubmit={(e) => {
                e.preventDefault();
                if (signupForm.password !== signupForm.confirmPassword) {
                  setError("Passwords do not match");
                  return;
                }
                setError(null);
                setIsLoggedIn(true);
                localStorage.setItem("isLoggedIn", "true");
                setSignupForm({ email: "", company: "", password: "", confirmPassword: "" });
                setCurrentView("create-campaign");
              }}>
                <div className="form-group">
                  <label>Email Address</label>
                  <input
                    type="email"
                    required
                    value={signupForm.email}
                    onChange={(e) => setSignupForm((prev) => ({ ...prev, email: e.target.value }))}
                    placeholder="you@example.com"
                  />
                </div>

                <div className="form-group">
                  <label>Company Name <span style={{ color: "var(--text-tertiary)", fontWeight: 400 }}>(optional)</span></label>
                  <input
                    type="text"
                    value={signupForm.company}
                    onChange={(e) => setSignupForm((prev) => ({ ...prev, company: e.target.value }))}
                    placeholder="Your company"
                  />
                </div>

                <div className="form-group">
                  <label>Password</label>
                  <input
                    type="password"
                    required
                    value={signupForm.password}
                    onChange={(e) => setSignupForm((prev) => ({ ...prev, password: e.target.value }))}
                    placeholder="Create a password"
                  />
                </div>

                <div className="form-group">
                  <label>Confirm Password</label>
                  <input
                    type="password"
                    required
                    value={signupForm.confirmPassword}
                    onChange={(e) => setSignupForm((prev) => ({ ...prev, confirmPassword: e.target.value }))}
                    placeholder="Confirm your password"
                  />
                </div>

                {error && <div className="error-message">{error}</div>}

                <button type="submit" className="login-submit-btn">
                  Create Account
                </button>

                <div className="login-divider">
                  <span>or</span>
                </div>

                <button type="button" className="wallet-login-btn" onClick={handleWalletConnect}>
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <rect x="1" y="4" width="22" height="16" rx="2" ry="2"></rect>
                    <line x1="1" y1="10" x2="23" y2="10"></line>
                  </svg>
                  Connect Wallet
                </button>

                <p className="login-footer">
                  Already have an account? <a href="#" onClick={(e) => { e.preventDefault(); setCurrentView("login"); }}>Sign in</a>
                </p>
              </form>
            </div>
          </div>
        </div>
      ) : currentView === "login" ? (
        /* Login Page */
        <div className="login-page">
          <button 
            className="back-button"
            onClick={handleBack}
            title="Go back"
          >
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M19 12H5"></path>
              <path d="M12 19l-7-7 7-7"></path>
            </svg>
            Back
          </button>
          <div className="login-container">
            <div className="login-card">
              <div className="login-header">
                <div className="login-logo" onClick={() => setCurrentView("landing")}>
                  <img src={logoImage} alt="SubsidyPayment" className="logo-icon-large" />
                  <h1>SubsidyPayment</h1>
                </div>
                <p className="login-subtitle">Sign in to manage your campaigns</p>
              </div>
              
              <form className="login-form" onSubmit={handleLogin}>
                <div className="form-group">
                  <label>Email Address</label>
                  <input
                    type="email"
                    required
                    value={loginForm.email}
                    onChange={(e) => setLoginForm((prev) => ({ ...prev, email: e.target.value }))}
                    placeholder="you@example.com"
                  />
                </div>
                
                <div className="form-group">
                  <label>Password</label>
                  <input
                    type="password"
                    required
                    value={loginForm.password}
                    onChange={(e) => setLoginForm((prev) => ({ ...prev, password: e.target.value }))}
                    placeholder="Enter your password"
                  />
                </div>
                
                <div className="login-options">
                  <label className="checkbox-label">
                    <input type="checkbox" />
                    <span>Remember me</span>
                  </label>
                  <a href="#" className="forgot-link">Forgot password?</a>
                </div>
                
                <button type="submit" className="login-submit-btn">
                  Sign In
                </button>
                
                <div className="login-divider">
                  <span>or</span>
                </div>
                
                <button type="button" className="wallet-login-btn" onClick={handleWalletConnect}>
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <rect x="1" y="4" width="22" height="16" rx="2" ry="2"></rect>
                    <line x1="1" y1="10" x2="23" y2="10"></line>
                  </svg>
                  Connect Wallet
                </button>
                
                <p className="login-footer">
                  Don't have an account? <a href="#" onClick={(e) => { e.preventDefault(); setCurrentView("signup"); }}>Sign up</a>
                </p>
              </form>
            </div>
          </div>
        </div>
      ) : currentView === "create-campaign" ? (
        /* Create Campaign Page */
        <main className="main-content">
          <div className="create-campaign-page">
            <div className="page-header">
              <button 
                className="back-button-inline"
                onClick={() => setCurrentView("dashboard")}
                title="Back to dashboard"
              >
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M19 12H5"></path>
                  <path d="M12 19l-7-7 7-7"></path>
                </svg>
                Back to Dashboard
              </button>
              <h2>Create New Campaign</h2>
              <p>Launch a payout stream for target developer segments</p>
            </div>

            <div className="card create-campaign-card">
              <div className="card-content">
                {error && <div className="error-message">{error}</div>}
                <div className="profile-card">
                  <div className="profile-avatar">
                    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
                      <circle cx="12" cy="7" r="4"></circle>
                    </svg>
                  </div>
                  <div className="profile-details">
                    <h4>Your Profile</h4>
                    <div className="profile-stats">
                      <span>Active: {totals.activeCampaigns}</span>
                      <span>Total: {campaigns.length}</span>
                      <span>Budget: ${totals.budgetDollars}</span>
                    </div>
                  </div>
                </div>
                <form className="campaign-form" onSubmit={onCreateCampaign}>
                  <div className="form-group">
                    <label>Campaign Name</label>
                    <input
                      required
                      value={form.name}
                      onChange={(e) => setForm((prev) => ({ ...prev, name: e.target.value }))}
                      placeholder="Enter campaign name"
                    />
                  </div>
                  <div className="form-group">
                    <label>Sponsor</label>
                    <input
                      required
                      value={form.sponsor}
                      onChange={(e) => setForm((prev) => ({ ...prev, sponsor: e.target.value }))}
                      placeholder="Enter sponsor name"
                    />
                  </div>

                  {/* NEW: Sponsored Tools / Services multi-select */}
                  <div className="form-group">
                    <label>Sponsored Tools / Services <span className="new-badge">NEW</span></label>
                    {selectedServices.length > 0 && (
                      <div className="service-chips">
                        {selectedServices.map((service) => (
                          <span key={service} className="service-chip">
                            {service}
                            <button type="button" onClick={() => setSelectedServices((prev) => prev.filter((s) => s !== service))}>&times;</button>
                          </span>
                        ))}
                      </div>
                    )}
                    <div className="service-dropdown-wrapper">
                      <button type="button" className="service-dropdown-toggle" onClick={() => setShowServiceDropdown(!showServiceDropdown)}>
                        {selectedServices.length === 0 ? "Select services..." : `${selectedServices.length} selected`}
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="6 9 12 15 18 9"></polyline></svg>
                      </button>
                      {showServiceDropdown && (
                        <div className="service-dropdown">
                          {SERVICE_CATEGORIES.map((category) => (
                            <div key={category.name} className="service-category">
                              <div className="service-category-header">{category.name}</div>
                              {category.services.map((service) => (
                                <label key={service} className="service-option">
                                  <input type="checkbox" checked={selectedServices.includes(service)} onChange={(e) => { if (e.target.checked) { setSelectedServices((prev) => [...prev, service]); } else { setSelectedServices((prev) => prev.filter((s) => s !== service)); } }} />
                                  <span>{service}</span>
                                </label>
                              ))}
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  </div>

                  {/* AI Suggestion Box */}
                  <div className="ai-suggestion-box">
                    <div className="ai-suggestion-content">
                      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M12 2a4 4 0 0 1 4 4c0 1.95-1.4 3.58-3.25 3.93L12 22"/><path d="M12 2a4 4 0 0 0-4 4c0 1.95 1.4 3.58 3.25 3.93"/><path d="M8.56 13.68C5.2 14.42 3 16.33 3 18.5 3 21 7.03 23 12 23s9-2 9-4.5c0-2.17-2.2-4.08-5.56-4.82"/></svg>
                      <div>
                        <p className="ai-suggestion-text">Not sure which services align with your campaign target? Ask AI to help you identify the best options.</p>
                        <button type="button" className="ai-suggestion-btn" onClick={() => alert("Coming soon! AI-powered service recommendations will be available in the next update.")}>Ask AI</button>
                      </div>
                    </div>
                  </div>

                  <div className="form-group">
                    <label>Target Roles (comma-separated)</label>
                    <input
                      value={form.target_roles}
                      onChange={(e) => setForm((prev) => ({ ...prev, target_roles: e.target.value }))}
                      placeholder="developer, designer, etc."
                    />
                  </div>
                  <div className="form-group">
                    <label>Target Tools (comma-separated)</label>
                    <input
                      value={form.target_tools}
                      onChange={(e) => setForm((prev) => ({ ...prev, target_tools: e.target.value }))}
                      placeholder="scraping, search, etc."
                    />
                  </div>
                  <div className="form-group">
                    <label>Required Task</label>
                    <input
                      required
                      value={form.required_task}
                      onChange={(e) => setForm((prev) => ({ ...prev, required_task: e.target.value }))}
                      placeholder="signup_sponsor"
                    />
                  </div>

                  {/* NEW: KPI to Track & Compare */}
                  <div className="form-group">
                    <label>KPI to Track & Compare <span className="new-badge">NEW</span></label>
                    <select className="kpi-select" value={selectedKpi} onChange={(e) => setSelectedKpi(e.target.value)}>
                      <option value="">Select a KPI...</option>
                      {KPI_OPTIONS.map((kpi) => (<option key={kpi} value={kpi}>{kpi}</option>))}
                    </select>
                  </div>

                  <div className="form-row">
                    <div className="form-group">
                      <label>Subsidy / Call (cents)</label>
                      <input
                        required
                        type="number"
                        min={1}
                        value={form.subsidy_per_call_cents}
                        onChange={(e) =>
                          setForm((prev) => ({
                            ...prev,
                            subsidy_per_call_cents: Number(e.target.value)
                          }))
                        }
                      />
                    </div>
                    <div className="form-group">
                      <label>Budget (cents)</label>
                      <input
                        required
                        type="number"
                        min={1}
                        value={form.budget_cents}
                        onChange={(e) =>
                          setForm((prev) => ({
                            ...prev,
                            budget_cents: Number(e.target.value)
                          }))
                        }
                      />
                    </div>
                  </div>
                  <button type="submit" className="submit-btn" disabled={createLoading}>
                    {createLoading ? "Creating..." : "Create Campaign"}
                  </button>
                </form>
              </div>
            </div>
          </div>
        </main>
      ) : currentView === "caller" ? (
        /* Caller Page */
        <main className="main-content">
          <div className="create-campaign-page">
            <div className="page-header">
              <button 
                className="back-button-inline"
                onClick={() => setCurrentView("dashboard")}
                title="Back to dashboard"
              >
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M19 12H5"></path>
                  <path d="M12 19l-7-7 7-7"></path>
                </svg>
                Back to Dashboard
              </button>
              <h2>API Caller</h2>
              <p>Call APIs and handle payments automatically</p>
            </div>

            <div className="card create-campaign-card">
              <div className="card-content">
                <form className="campaign-form" onSubmit={(e) => { e.preventDefault(); void handleApiCall(); }}>
                  <div className="form-group">
                    <label>Call Type</label>
                    <select
                      value={callerForm.callType}
                      onChange={(e) => setCallerForm((prev) => ({ ...prev, callType: e.target.value as any }))}
                    >
                      <option value="proxy">Proxy Service</option>
                      <option value="tool">Direct Tool</option>
                      <option value="sponsored-api">Sponsored API</option>
                    </select>
                  </div>

                  {callerForm.callType === "sponsored-api" ? (
                    <div className="form-group">
                      <label>Sponsored API</label>
                      <select
                        value={callerForm.apiId}
                        onChange={(e) => setCallerForm((prev) => ({ ...prev, apiId: e.target.value }))}
                      >
                        <option value="">Select an API</option>
                        {sponsoredApis.map((api) => (
                          <option key={api.id} value={api.id}>
                            {api.name} - ${(api.price_cents / 100).toFixed(2)} per call
                            {api.active && api.budget_remaining_cents > 0 ? " (Sponsored)" : " (Paid)"}
                          </option>
                        ))}
                      </select>
                    </div>
                  ) : (
                    <div className="form-group">
                      <label>Service Name</label>
                      <input
                        required
                        value={callerForm.service}
                        onChange={(e) => setCallerForm((prev) => ({ ...prev, service: e.target.value }))}
                        placeholder="e.g., scraping, design, storage"
                      />
                    </div>
                  )}

                  <div className="form-group">
                    <label>User ID (optional, defaults to wallet address)</label>
                    <input
                      value={callerForm.userId}
                      onChange={(e) => setCallerForm((prev) => ({ ...prev, userId: e.target.value }))}
                      placeholder="Leave empty to use wallet address"
                    />
                  </div>

                  <div className="form-group">
                    <label>Input {callerForm.callType === "sponsored-api" ? "(JSON)" : "(text)"}</label>
                    <textarea
                      rows={6}
                      value={callerForm.input}
                      onChange={(e) => setCallerForm((prev) => ({ ...prev, input: e.target.value }))}
                      placeholder={callerForm.callType === "sponsored-api" ? '{"key": "value"}' : "Enter input text"}
                    />
                  </div>

                  {paymentRequired && (
                    <div className="payment-required-box">
                      <div className="payment-header">
                        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                          <circle cx="12" cy="12" r="10"></circle>
                          <line x1="12" y1="8" x2="12" y2="12"></line>
                          <line x1="12" y1="16" x2="12.01" y2="16"></line>
                        </svg>
                        <h4>Payment Required</h4>
                      </div>
                      <div className="payment-details">
                        <p><strong>Service:</strong> {paymentRequired.service}</p>
                        <p><strong>Amount:</strong> ${(paymentRequired.amount_cents / 100).toFixed(2)}</p>
                        <p><strong>Message:</strong> {paymentRequired.message}</p>
                        <p><strong>Next Step:</strong> {paymentRequired.next_step}</p>
                      </div>
                      <button
                        type="button"
                        className="wallet-login-btn"
                        onClick={handlePayment}
                        disabled={callerLoading}
                      >
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                          <rect x="1" y="4" width="22" height="16" rx="2" ry="2"></rect>
                          <line x1="1" y1="10" x2="23" y2="10"></line>
                        </svg>
                        Pay with Wallet
                      </button>
                    </div>
                  )}

                  {callerError && <div className="error-message">{callerError}</div>}

                  <button type="submit" className="submit-btn" disabled={callerLoading}>
                    {callerLoading ? "Calling..." : "Call API"}
                  </button>
                </form>

                {callerResult && (
                  <div className="caller-result">
                    <h4>Result</h4>
                    <div className="result-box">
                      <pre>{JSON.stringify(callerResult, null, 2)}</pre>
                    </div>
                    {callerResult.payment_mode && (
                      <div className="payment-info">
                        <p><strong>Payment Mode:</strong> {callerResult.payment_mode}</p>
                        {callerResult.sponsored_by && (
                          <p><strong>Sponsored By:</strong> {callerResult.sponsored_by}</p>
                        )}
                        {callerResult.tx_hash && (
                          <p><strong>Transaction Hash:</strong> {callerResult.tx_hash}</p>
                        )}
                      </div>
                    )}
                  </div>
                )}
              </div>
            </div>
          </div>
        </main>
      ) : (
        /* Dashboard */
        <main className="main-content">
          {/* A. Top Metrics Row */}
          <div className="metrics-row">
            <div className="metric-card">
              <div className="metric-card-label">Remaining Budget</div>
              <div className="metric-card-value">$12,450.00</div>
              <div className="metric-card-sub">75.3% of $16,540 total</div>
            </div>
            <div className="metric-card">
              <div className="metric-card-label">Total Subsidized Amount</div>
              <div className="metric-card-value">$4,090.00</div>
              <div className="metric-card-sub positive">+18.2% vs last week</div>
            </div>
            <div className="metric-card">
              <div className="metric-card-label">Users Subsidized</div>
              <div className="metric-card-value">1,247</div>
              <div className="metric-card-sub positive">+89 new this week</div>
            </div>
            <div className="metric-card">
              <div className="metric-card-label">Burn Rate</div>
              <div className="metric-card-value">$127.50/day</div>
              <div className="metric-card-sub">~98 days until depletion, Apr 19 2026</div>
            </div>
          </div>

          {/* B. Second Row */}
          <div className="two-col-row">
            <div className="card">
              <div className="card-header"><div className="card-title"><h3>Subsidy Consumption per User</h3></div></div>
              <div className="card-content">
                <div className="inner-box-row">
                  <div className="inner-box">
                    <div className="inner-box-label">Frequency</div>
                    <div className="inner-box-value">4.7 calls</div>
                    <div className="inner-box-detail">Median 3.2 &middot; P90 11.4</div>
                  </div>
                  <div className="inner-box">
                    <div className="inner-box-label">Intensity</div>
                    <div className="inner-box-value">$0.69/call</div>
                    <div className="inner-box-detail">Range $0.05 - $2.40</div>
                  </div>
                </div>
              </div>
            </div>
            <div className="card">
              <div className="card-header"><div className="card-title"><h3>Budget Pacing & Depletion Forecast</h3></div></div>
              <div className="card-content">
                <div className="progress-bar-container">
                  <div className="progress-bar-fill" style={{ width: "24.7%" }}></div>
                </div>
                <div className="progress-bar-label">24.7% spent</div>
                <div className="inner-box-row" style={{ marginTop: "16px" }}>
                  <div className="inner-box">
                    <div className="inner-box-label">Daily Burn</div>
                    <div className="inner-box-value">$127.50/day</div>
                  </div>
                  <div className="inner-box">
                    <div className="inner-box-label">Forecast Depletion</div>
                    <div className="inner-box-value">Apr 19 2026</div>
                    <div className="inner-box-detail">~98 days</div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* C. Third Row */}
          <div className="two-col-row">
            <div className="card">
              <div className="card-header"><div className="card-title"><h3>CPA Efficiency vs Sponsor Target</h3></div></div>
              <div className="card-content">
                <div className="cpa-row"><span className="cpa-label">Cost per Signup</span><span className="cpa-values">$5.20 vs $8.50 target</span><span className="cpa-badge">-38.8%</span></div>
                <div className="cpa-row"><span className="cpa-label">Cost per Activated User</span><span className="cpa-values">$9.80 vs $12.00</span><span className="cpa-badge">-18.3%</span></div>
                <div className="cpa-row"><span className="cpa-label">Cost per Qualified Lead</span><span className="cpa-values">$18.50 vs $22.00</span><span className="cpa-badge">-15.9%</span></div>
              </div>
            </div>
            <div className="card">
              <div className="card-header"><div className="card-title"><h3>Subsidized Task Breakdown</h3></div></div>
              <div className="card-content">
                {[
                  { label: "Signup to sponsor services", pct: 34.2 },
                  { label: "Survey responses", pct: 27.8 },
                  { label: "Email sharing", pct: 21.5 },
                  { label: "Ad views", pct: 11.3 },
                  { label: "Social media sharing", pct: 5.2 }
                ].map((task, i) => (
                  <div key={task.label} className="task-item">
                    <span className="task-item-dot" style={{ background: taskBreakdownColors[i] }}></span>
                    <span className="task-item-label">{task.label}</span>
                    <span className="task-item-pct">{task.pct}%</span>
                  </div>
                ))}
                <div className="task-bar-stack">
                  {[34.2, 27.8, 21.5, 11.3, 5.2].map((pct, i) => (
                    <div key={i} className="task-bar-segment" style={{ width: `${pct}%`, background: taskBreakdownColors[i] }}></div>
                  ))}
                </div>
              </div>
            </div>
          </div>

          {/* D. Task Completion Metrics */}
          <div className="card full-width">
            <div className="card-header"><div className="card-title"><h3>Task Completion Metrics</h3></div></div>
            <div className="card-content">
              <div className="metrics-row metrics-row-inner">
                <div className="metric-card metric-card-compact"><div className="metric-card-label">Avg Time-to-Complete</div><div className="metric-card-value">2m 34s</div><div className="metric-card-sub positive">-12% faster</div></div>
                <div className="metric-card metric-card-compact"><div className="metric-card-label">Avg Delay</div><div className="metric-card-value">18.3s</div><div className="metric-card-sub positive">under 30s SLA</div></div>
                <div className="metric-card metric-card-compact"><div className="metric-card-label">Completion Rate</div><div className="metric-card-value">94.7%</div><div className="metric-card-sub positive">+2.1%</div></div>
              </div>
            </div>
          </div>

          {/* E. Comparative Performance Table */}
          <div className="card full-width">
            <div className="card-header"><div className="card-title"><h3>Comparative Performance</h3></div></div>
            <div className="table-container">
              <table className="comparison-table">
                <thead><tr><th>Service</th><th>Users</th><th>Total Subsidy</th><th>CPA</th><th>Completion</th><th>Trend</th></tr></thead>
                <tbody>
                  <tr><td>Uniswap</td><td>482</td><td>$1,580</td><td>$3.28</td><td>96.2%</td><td><span className="trend-positive">+12%</span></td></tr>
                  <tr><td>Aave</td><td>351</td><td>$1,240</td><td>$3.53</td><td>93.8%</td><td><span className="trend-positive">+8%</span></td></tr>
                  <tr><td>OpenSea</td><td>289</td><td>$890</td><td>$3.08</td><td>91.4%</td><td><span className="trend-negative">-3%</span></td></tr>
                </tbody>
              </table>
            </div>
          </div>

          {/* F. User Base Ranking */}
          <div className="card full-width">
            <div className="card-header"><div className="card-title"><h3>User Base Ranking</h3></div></div>
            <div className="card-content">
              {[
                { rank: 1, name: "Uniswap", pct: 78.4 },
                { rank: 2, name: "Aave", pct: 62.1 },
                { rank: 3, name: "OpenSea", pct: 45.6 },
                { rank: 4, name: "Lido Finance", pct: 33.2 },
                { rank: 5, name: "Compound", pct: 24.8 }
              ].map((item) => (
                <div key={item.rank} className="ranking-item">
                  <span className="ranking-number">{item.rank}</span>
                  <span className="ranking-name">{item.name}</span>
                  <div className="ranking-bar-bg">
                    <div className="ranking-bar-fill" style={{ width: `${item.pct}%` }}></div>
                  </div>
                  <span className="ranking-pct">{item.pct}%</span>
                </div>
              ))}
            </div>
          </div>

        {/* Campaigns Table Card */}
        <div className="card full-width">
          <div className="card-header">
            <div className="card-title">
              <h3>Campaign Details</h3>
            </div>
            <div className="card-actions">
              <button 
                className="primary-btn" 
                onClick={() => {
                  if (isLoggedIn) {
                    setCurrentView("create-campaign");
                  } else {
                    setCurrentView("login");
                  }
                }}
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <line x1="12" y1="5" x2="12" y2="19"></line>
                  <line x1="5" y1="12" x2="19" y2="12"></line>
                </svg>
                Create Campaign
              </button>
              <button 
                className="primary-btn" 
                onClick={() => setCurrentView("caller")}
                style={{ marginLeft: "8px" }}
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z"></path>
                </svg>
                API Caller
              </button>
              <button className="ghost-btn" onClick={() => void loadDashboard(false)}>
                Refresh
              </button>
            </div>
          </div>
          <div className="table-container">
              <table>
                <thead>
                  <tr>
                    <th>Name</th>
                    <th>Sponsor</th>
                    <th>Targets</th>
                    <th>Subsidy</th>
                    <th>Budget Left</th>
                    <th>Status</th>
                  </tr>
                </thead>
                <tbody>
                  {loading ? (
                    <tr>
                      <td colSpan={6}>Loading dashboard...</td>
                    </tr>
                  ) : campaigns.length === 0 ? (
                    <tr>
                      <td colSpan={6}>No campaigns yet.</td>
                    </tr>
                  ) : (
                    campaigns.map((campaign) => (
                      <tr key={campaign.id}>
                        <td>{campaign.name}</td>
                        <td>{campaign.sponsor}</td>
                        <td>
                          {campaign.target_roles.join(", ")} / {campaign.target_tools.join(", ")}
                        </td>
                        <td>${(campaign.subsidy_per_call_cents / 100).toFixed(2)}</td>
                        <td>${(campaign.budget_remaining_cents / 100).toFixed(2)}</td>
                        <td>
                        <span className={campaign.active ? "status-badge active" : "status-badge paused"}>
                            {campaign.active ? "ACTIVE" : "PAUSED"}
                          </span>
                        </td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
            </div>

          {/* User Profile Section - Only shown when logged in */}
          {isLoggedIn && showProfile && (
          <div className="card profile-card">
            <div className="card-header">
              <div className="card-title">
                <h3>My Profile</h3>
              </div>
              <button 
                className="icon-btn-small"
                onClick={() => setShowProfile(false)}
                title="Close"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <line x1="18" y1="6" x2="6" y2="18"></line>
                  <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
              </button>
            </div>
            <div className="card-content">
              {/* Profile Header */}
              <div className="profile-section">
                <div className="profile-header">
                  <div className="profile-avatar-large">
                    <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
                      <circle cx="12" cy="7" r="4"></circle>
                    </svg>
                  </div>
                  <div className="profile-info">
                    <h4>Campaign Creator</h4>
                    <p>Manage your sponsored campaigns and track performance</p>
                  </div>
                </div>
                <div className="profile-stats">
                  <div className="profile-stat">
                    <span className="stat-number">{totals.activeCampaigns}</span>
                    <span className="stat-label">Active</span>
                  </div>
                  <div className="profile-stat">
                    <span className="stat-number">{campaigns.length}</span>
                    <span className="stat-label">Total</span>
                  </div>
                  <div className="profile-stat">
                    <span className="stat-number">${totals.budgetDollars}</span>
                    <span className="stat-label">Budget</span>
                  </div>
                </div>
              </div>

              {/* Create Campaign Button */}
              <div className="profile-form-section">
                <div className="section-divider">
                  <h4>Create New Campaign</h4>
                  <p>Launch a payout stream for target developer segments</p>
                </div>
                <button 
                  className="primary-btn-large" 
                  onClick={() => {
                    if (isLoggedIn) {
                      setCurrentView("create-campaign");
                    } else {
                      setCurrentView("login");
                    }
                  }}
                >
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <line x1="12" y1="5" x2="12" y2="19"></line>
                    <line x1="5" y1="12" x2="19" y2="12"></line>
                  </svg>
                  Go to Create Campaign
                </button>
              </div>
            </div>
            </div>
          )}
        </main>
      )}
    </div>
  );
}

function splitCsv(raw: string): string[] {
  return raw
    .split(",")
    .map((value) => value.trim())
    .filter(Boolean);
}

export default App;
