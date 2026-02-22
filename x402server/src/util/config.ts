import { x402Client } from "@x402/axios";
import { HTTPFacilitatorClient, x402ResourceServer } from "@x402/core/server";
import { ExactEvmScheme } from "@x402/evm/exact/server";
import { privateKeyToAccount } from "viem/accounts";
import { facilitatorUrl, network, privateKey } from "./constants";

export const facilitatorClient = new HTTPFacilitatorClient({
  url: facilitatorUrl,
});

// リソースサーバー
export const resourceServer = new x402ResourceServer(
  facilitatorClient,
).register(network, new ExactEvmScheme());

// デモ用のSignerインスタンス
export const signer = privateKeyToAccount(privateKey);

// v2 pattern: Create client and register scheme separately
export const client = new x402Client();
