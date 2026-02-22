import { wrapAxiosWithPayment } from "@x402/axios";
import { registerExactEvmScheme } from "@x402/evm/exact/client";
import axios from "axios";
import { config } from "dotenv";
import { client, signer } from "../util/config";

config();

/**
 * メイン関数
 */
const main = async () => {
  registerExactEvmScheme(client, { signer });

  // Wrap axios with payment handling
  const api = wrapAxiosWithPayment(
    axios.create({ baseURL: "http://localhost:4021" }),
    client,
  );

  const response = await api.get("/weather");

  console.log("Weather data:", response.data);
};

main().catch((error) => {
  console.error("Error:", error);
});
