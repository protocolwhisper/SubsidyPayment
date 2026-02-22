import { registerAppTool } from '@modelcontextprotocol/ext-apps/server';
import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';

import { TokenVerifier } from '../auth/token-verifier.ts';
import { BackendClient, BackendClientError } from '../backend-client.ts';
import type { BackendConfig } from '../config.ts';
import type { PaymentRequiredResponse } from '../types.ts';
import { resolveOrCreateNoAuthSessionToken } from './session-manager.ts';

const runServiceInputSchema = z.object({
  service: z.string(),
  input: z.string(),
  session_token: z.string().optional(),
});

const DIRECT_PAYMENT_SENTINEL = '__pay_direct__';

function unauthorizedSessionResponse(publicUrl: string) {
  return {
    content: [{ type: 'text' as const, text: 'Login is required to perform this action.' }],
    _meta: {
      'mcp/www_authenticate': [
        `Bearer resource_metadata="${publicUrl}/.well-known/oauth-protected-resource"`,
      ],
    },
    isError: true,
  };
}

function resolveBearerToken(context: any): string | null {
  const authToken = context?.auth?.token ?? context?._meta?.auth?.token ?? null;
  if (typeof authToken === 'string' && authToken.length > 0) {
    return authToken;
  }
  const authorization = context?.headers?.authorization ?? context?._meta?.headers?.authorization ?? null;
  if (typeof authorization === 'string' && authorization.startsWith('Bearer ')) {
    return authorization.slice('Bearer '.length);
  }
  return null;
}

function parsePaymentRequired(details: unknown): PaymentRequiredResponse | null {
  if (!details || typeof details !== 'object') return null;
  const candidate = details as Record<string, unknown>;
  if (
    typeof candidate.service !== 'string' ||
    typeof candidate.amount_cents !== 'number' ||
    typeof candidate.accepted_header !== 'string' ||
    typeof candidate.payment_required !== 'string' ||
    typeof candidate.message !== 'string' ||
    typeof candidate.next_step !== 'string'
  ) {
    return null;
  }

  return {
    service: candidate.service,
    amount_cents: candidate.amount_cents,
    accepted_header: candidate.accepted_header,
    payment_required: candidate.payment_required,
    message: candidate.message,
    next_step: candidate.next_step,
  };
}

function isTaskRequiredMessage(message: string): boolean {
  const normalized = message.toLowerCase();
  return normalized.includes('complete the required task') || normalized.includes('complete sponsor task');
}

function isNoSponsorMessage(message: string): boolean {
  return message.toLowerCase().includes('no sponsored campaign found');
}

function serviceExecutedResult(response: {
  service: string;
  payment_mode: 'sponsored' | 'user_direct';
  sponsored_by: string | null;
  tx_hash: string | null;
  message: string;
  output: string;
}) {
  return {
    structuredContent: {
      mode: 'service_executed',
      service: response.service,
      payment_mode: response.payment_mode,
      sponsored_by: response.sponsored_by,
      tx_hash: response.tx_hash,
      message: response.message,
    },
    content: [
      { type: 'text' as const, text: response.message },
    ],
    _meta: {
      output: response.output,
    },
  };
}

function paymentRequiredResult(payment: PaymentRequiredResponse) {
  return {
    structuredContent: {
      mode: 'payment_required',
      service: payment.service,
      amount_cents: payment.amount_cents,
      accepted_header: payment.accepted_header,
      payment_required: payment.payment_required,
      next_step: payment.next_step,
      payment_mode: 'user_direct',
    },
    content: [
      { type: 'text' as const, text: `x402 payment required. ${payment.next_step}` },
    ],
    _meta: {
      payment_required: payment,
    },
  };
}

async function directPayFallbackResult(
  client: BackendClient,
  service: string,
  inputPayload: string,
  sessionToken: string
) {
  const status = await client.getUserStatus(sessionToken);

  try {
    const response = await client.runProxyService(service, {
      user_id: status.user_id,
      input: inputPayload.trim().toLowerCase() === DIRECT_PAYMENT_SENTINEL ? 'direct-pay-request' : inputPayload,
    });

    return {
      structuredContent: {
        mode: 'service_executed',
        service: response.service,
        payment_mode: response.payment_mode,
        sponsored_by: response.sponsored_by,
        tx_hash: response.tx_hash,
        message: 'Service executed through proxy.',
      },
      content: [
        { type: 'text' as const, text: 'Service executed through proxy.' },
      ],
      _meta: {
        output: response.output,
      },
    };
  } catch (error) {
    if (error instanceof BackendClientError && error.code === 'payment_required') {
      const payment = parsePaymentRequired(error.details);
      if (payment) return paymentRequiredResult(payment);
    }
    throw error;
  }
}

function deriveTaskOptions(taskInputFormat: { task_type?: string; required_fields?: string[] } | null | undefined): string[] {
  if (!taskInputFormat) return [];

  const options: string[] = [];
  const taskType = taskInputFormat.task_type;
  if (typeof taskType === 'string' && taskType.trim().length > 0) {
    options.push(taskType.trim());
  }

  const fields = taskInputFormat.required_fields;
  if (Array.isArray(fields)) {
    for (const field of fields) {
      if (typeof field === 'string' && field.trim().length > 0) {
        options.push(field.trim());
      }
    }
  }

  return options;
}

async function taskRequiredResult(client: BackendClient, service: string, sessionToken: string) {
  const searchResponse = await client.searchServices({
    q: service,
    session_token: sessionToken,
  });
  const campaign = searchResponse.services.find((item) => item.service_type === 'campaign' && item.active);
  if (!campaign) return null;

  const task = await client.getTaskDetails(campaign.service_id, sessionToken);
  const taskOptions = deriveTaskOptions(task.task_input_format);

  return {
    structuredContent: {
      mode: 'task_required',
      service,
      campaign_id: task.campaign_id,
      campaign_name: task.campaign_name,
      sponsor: task.sponsor,
      required_task: task.required_task,
      task_description: task.task_description,
      task_input_format: task.task_input_format,
      subsidy_amount_cents: task.subsidy_amount_cents,
      task_options: taskOptions,
      payment_mode: 'sponsored',
    },
    content: [
      {
        type: 'text' as const,
        text: `Free option available. Complete '${task.required_task}' to unlock sponsor coverage, or switch to direct x402 payment.`,
      },
    ],
    _meta: {
      full_response: task,
    },
  };
}

export function registerRunServiceTool(server: McpServer, config: BackendConfig): void {
  const client = new BackendClient(config);
  const verifier = new TokenVerifier({
    domain: config.auth0Domain,
    audience: config.auth0Audience,
  });

  registerAppTool(
    server,
    'run_service',
    {
      title: 'Run Service',
      description: 'Execute a service with sponsor-backed payment.',
      inputSchema: runServiceInputSchema.shape,
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        openWorldHint: true,
      },
      _meta: {
        securitySchemes: config.authEnabled
          ? [{ type: 'oauth2', scopes: ['services.execute'] }]
          : [{ type: 'noauth' }],
        ui: { resourceUri: 'ui://widget/service-access.html' },
        'openai/resultCanProduceWidget': true,
        'openai/widgetAccessible': true,
        'openai/toolInvocation/invoking': 'Running service...',
        'openai/toolInvocation/invoked': 'Service run completed',
        'openai/outputTemplate': 'ui://widget/service-access.html',
      },
    },
    async (input, context: any) => {
      if (config.authEnabled) {
        const bearerToken = resolveBearerToken(context);
        const authInfo = bearerToken ? await verifier.verify(bearerToken) : null;
        if (!authInfo) {
          return unauthorizedSessionResponse(config.publicUrl);
        }
      }

      const sessionToken = await resolveOrCreateNoAuthSessionToken(client, config, input, context);
      if (!sessionToken) {
        return unauthorizedSessionResponse(config.publicUrl);
      }


      if (input.input.trim().toLowerCase() === DIRECT_PAYMENT_SENTINEL) {
        try {
          return await directPayFallbackResult(client, input.service, input.input, sessionToken);
        } catch (error) {
          if (error instanceof BackendClientError) {
            return {
              content: [{ type: 'text' as const, text: error.message }],
              _meta: { code: error.code, details: error.details },
              isError: true,
            };
          }
          return {
            content: [{ type: 'text' as const, text: 'An unexpected error occurred while preparing direct payment.' }],
            _meta: { code: 'unexpected_error' },
            isError: true,
          };
        }
      }

      try {
        const response = await client.runService(input.service, {
          service: input.service,
          session_token: sessionToken,
          input: input.input,
        });

        return serviceExecutedResult(response);
      } catch (error) {
        if (error instanceof BackendClientError) {
          if (error.code === 'payment_required') {
            const payment = parsePaymentRequired(error.details);
            if (payment) return paymentRequiredResult(payment);
          }

          if (error.code === 'precondition_required') {
            if (isTaskRequiredMessage(error.message)) {
              try {
                const taskResult = await taskRequiredResult(client, input.service, sessionToken);
                if (taskResult) return taskResult;
              } catch {
                // fall through to backend error
              }
            }

            if (isNoSponsorMessage(error.message)) {
              try {
                return await directPayFallbackResult(client, input.service, input.input, sessionToken);
              } catch {
                // fall through to backend error
              }
            }
          }

          return {
            content: [{ type: 'text' as const, text: error.message }],
            _meta: { code: error.code, details: error.details },
            isError: true,
          };
        }

        return {
          content: [{ type: 'text' as const, text: 'An unexpected error occurred while running the service.' }],
          _meta: { code: 'unexpected_error' },
          isError: true,
        };
      }
    }
  );
}
