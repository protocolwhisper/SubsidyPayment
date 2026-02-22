import { wrapAxiosWithPayment, x402Client } from '@x402/axios';
import { ExactEvmScheme } from '@x402/evm';
import axios, { type AxiosError, isAxiosError } from 'axios';
import { privateKeyToAccount } from 'viem/accounts';

import { BackendClientError } from '../backend-client.ts';
import type { BackendConfig } from '../config.ts';
import type { WeatherReport } from '../types.ts';

type WeatherApiResponse = {
  report?: Partial<WeatherReport>;
};

function parseWeatherReport(payload: unknown, fallbackCity: string): WeatherReport {
  const body = payload as WeatherApiResponse | null;
  const report = body?.report;

  if (!report || typeof report !== 'object') {
    throw new BackendClientError(
      'weather_invalid_response',
      'Weather API returned an invalid response payload.',
      payload
    );
  }

  const weather = report.weather;
  const temperature = report.temperature;
  const city = report.city;

  if (typeof weather !== 'string' || typeof temperature !== 'number') {
    throw new BackendClientError(
      'weather_invalid_response',
      'Weather API response is missing weather details.',
      payload
    );
  }

  return {
    city: typeof city === 'string' && city.length > 0 ? city : fallbackCity,
    weather,
    temperature,
  };
}

function toBackendClientError(error: AxiosError): BackendClientError {
  if (error.code === 'ECONNABORTED') {
    return new BackendClientError('backend_timeout', 'Weather API request timed out.', {
      message: error.message,
    });
  }

  if (error.response) {
    return new BackendClientError('weather_request_failed', `Weather API request failed with status ${error.response.status}.`, {
      status: error.response.status,
      data: error.response.data,
    });
  }

  return new BackendClientError('backend_unavailable', 'Weather API is unavailable.', {
    message: error.message,
  });
}

export class X402WeatherClient {
  private readonly api;
  private readonly weatherUrl: string;

  constructor(config: BackendConfig) {
    if (!config.x402PrivateKey) {
      throw new BackendClientError(
        'x402_config_error',
        'X402_PRIVATE_KEY is required to call x402 weather endpoint. Set it in mcp-server/.env or process environment.'
      );
    }

    this.weatherUrl = config.x402WeatherUrl;

    const account = privateKeyToAccount(config.x402PrivateKey as `0x${string}`);
    const paymentClient = new x402Client().register(config.x402Network, new ExactEvmScheme(account));

    this.api = wrapAxiosWithPayment(
      axios.create({
        timeout: config.x402RequestTimeoutMs,
      }),
      paymentClient
    );
  }

  async getWeather(city: string): Promise<WeatherReport> {
    try {
      const response = await this.api.get(this.weatherUrl, {
        params: { city },
      });
      return parseWeatherReport(response.data, city);
    } catch (error) {
      if (isAxiosError(error)) {
        throw toBackendClientError(error);
      }
      if (error instanceof BackendClientError) {
        throw error;
      }
      throw new BackendClientError('backend_unavailable', 'Weather API is unavailable.', error);
    }
  }
}
