import { serve } from "@hono/node-server";
import { config } from "dotenv";
import { Hono } from "hono";
import { cors } from "hono/cors";
import type { Network, Resource } from "x402-hono";
import { paymentMiddleware } from "x402-hono";
// Import walrus functions from relative paths

config();

const facilitatorUrl = process.env.FACILITATOR_URL as Resource;
const payTo = process.env.ADDRESS as `0x${string}`;
const network = process.env.NETWORK as Network;

if (!facilitatorUrl || !payTo || !network) {
  console.error("Missing required environment variables");
  process.exit(1);
}

const app = new Hono();

console.log("Server is running");

// CORS
app.use(
  cors({
    origin: ["*"],
    allowHeaders: ["Content-Type", "Authorization"],
    allowMethods: ["GET", "POST", "PUT", "DELETE", "OPTIONS"],
    exposeHeaders: ["Content-Length", "X-Requested-With"],
    maxAge: 600,
    credentials: true,
  }),
);

// Health check endpoint (before payment middleware)
app.get("/health", async (c) => {
  return c.json({
    status: "ok",
    timestamp: new Date().toISOString(),
    port,
  });
});

app.use(
  // /weather endpoint にアクセスした時に requires paymentを発動させる
  paymentMiddleware(
    payTo,
    {
      "/weather": {
        price: "$0.001",
        network,
      },
    },
    {
      url: facilitatorUrl,
    },
  ),
);

// get weather report API
app.get("/weather", async (c) => {
  // 任意の処理をここに入れる
  console.log("Payment received, processing weather report request...");
  // ステーブルコインを支払った結果としてダミーの天気データを返す
  return c.json({
    report: {
      weather: "sunny",
      temperature: 70,
    },
  });
});

// Use PORT from environment variable for Cloud Run compatibility
const port = process.env.PORT ? Number.parseInt(process.env.PORT, 10) : 4021;

serve({
  fetch: app.fetch,
  port,
});

console.log(`Server is running on port ${port}`);
