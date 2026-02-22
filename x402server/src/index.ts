import { serve } from "@hono/node-server";
import { paymentMiddleware } from "@x402/hono";
import { config } from "dotenv";
import { Hono } from "hono";
import { routes } from "./routes";
import { resourceServer } from "./util/config";
config();

// Honoアプリケーションの作成
const app = new Hono();

// ミドルウェアの設定(ルーティングとリソースサーバーを設定)
app.use(paymentMiddleware(routes, resourceServer));

// ヘルスチェック用のエンドポイント
app.get("/health", (c) => {
  return c.json({ status: "ok" });
});

// 支払いが必要なエンドポイントの定義
app.get("/weather", (c) => {
  return c.json({
    report: {
      city: "San Francisco",
      weather: "sunny",
      temperature: 70,
    },
  });
});

serve({
  fetch: app.fetch,
  port: 4021,
});

console.log(`Server listening at http://localhost:4021`);
