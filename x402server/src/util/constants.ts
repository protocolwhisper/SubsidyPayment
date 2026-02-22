import { config } from "dotenv";
config();

export const evmAddress = process.env.EVM_ADDRESS as `0x${string}`;
export const facilitatorUrl = process.env.FACILITATOR_URL;
export const privateKey = process.env.EVM_PRIVATE_KEY as `0x${string}`;
export const network = process.env.NETWORK as `${string}:${string}`;
