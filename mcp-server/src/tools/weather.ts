import { registerAppTool } from '@modelcontextprotocol/ext-apps/server';
import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';

import { BackendClientError } from '../backend-client.ts';
import type { BackendConfig } from '../config.ts';
import type { WeatherToolInput } from '../types.ts';
import { X402WeatherClient } from '../x402/weather-client.ts';

const weatherInputSchema = z.object({
  city: z.string().trim().min(1),
});

export function registerWeatherTool(server: McpServer, config: BackendConfig): void {
  registerAppTool(
    server,
    'weather',
    {
      title: 'Get Weather',
      description: 'Fetch weather data from x402 weather endpoint.',
      inputSchema: weatherInputSchema.shape,
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        openWorldHint: true,
      },
      _meta: {
        securitySchemes: [{ type: 'noauth' }],
        'openai/toolInvocation/invoking': 'Fetching weather...',
        'openai/toolInvocation/invoked': 'Weather fetched',
      },
    },
    async (input: WeatherToolInput) => {
      try {
        const client = new X402WeatherClient(config);
        const weather = await client.getWeather(input.city.trim());

        return {
          structuredContent: {
            city: weather.city,
            weather: weather.weather,
            temperature: weather.temperature,
            source: 'x402server',
            paid: true,
          },
          content: [
            {
              type: 'text' as const,
              text: `${weather.city} is ${weather.weather} (${weather.temperature}F).`,
            },
          ],
          _meta: {
            report: weather,
            endpoint: config.x402WeatherUrl,
          },
        };
      } catch (error) {
        if (error instanceof BackendClientError) {
          return {
            content: [{ type: 'text' as const, text: error.message }],
            _meta: { code: error.code, details: error.details },
            isError: true,
          };
        }

        return {
          content: [{ type: 'text' as const, text: 'An unexpected error occurred while fetching weather.' }],
          _meta: { code: 'unexpected_error' },
          isError: true,
        };
      }
    }
  );
}
