import { evmAddress, network } from "./util/constants";

// x402 Server Routes configuration
export const routes = {
  "GET /weather": {
    accepts: [
      {
        scheme: "exact",
        price: "$0.001",
        network: network,
        payTo: evmAddress,
      },
    ],
    description: "Weather data",
    mimeType: "application/json",
  },
  "GET /github-issue": {
    accepts: [
      {
        scheme: "exact",
        price: "$0.02",
        network: network,
        payTo: evmAddress,
      },
    ],
    description: "GitHub issue creation",
    mimeType: "application/json",
  },
};
