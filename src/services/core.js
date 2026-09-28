const DEFAULT_PAGE_SIZE = 50;

export class ServiceError extends Error {
  constructor({ code = 'UNKNOWN_ERROR', message = 'কাজটি সম্পন্ন করা সম্ভব হয়নি।', retryable = false, fieldErrors = {}, traceId = null } = {}) {
    super(message);
    this.name = 'ServiceError';
    this.code = code;
    this.retryable = Boolean(retryable);
    this.fieldErrors = fieldErrors || {};
    this.traceId = traceId;
  }
}

export function normalizeServiceError(error) {
  if (error instanceof ServiceError) return error;
  if (error && typeof error === 'object') {
    return new ServiceError({
      code: error.code,
      message: error.message,
      retryable: error.retryable,
      fieldErrors: error.fieldErrors ?? error.field_errors,
      traceId: error.traceId ?? error.trace_id
    });
  }
  return new ServiceError({ message: typeof error === 'string' ? error : undefined });
}

export function assertUuid(id, field = 'id') {
  if (typeof id !== 'string' || !/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(id)) {
    throw new ServiceError({ code: 'INVALID_ID', message: `${field} সঠিক নয়।`, fieldErrors: { [field]: 'invalid_uuid' } });
  }
  return id;
}

export function pageRequest(input = {}) {
  const page = Number.isInteger(input.page) && input.page > 0 ? input.page : 1;
  const pageSize = [25, 50, 100].includes(input.pageSize) ? input.pageSize : DEFAULT_PAGE_SIZE;
  return { ...input, page, pageSize };
}

export function assertPage(result) {
  if (!result || !Array.isArray(result.items) || !Number.isInteger(result.page) || !Number.isInteger(result.pageSize) || !Number.isInteger(result.total) || typeof result.hasNext !== 'boolean') {
    throw new ServiceError({ code: 'INVALID_SERVICE_RESPONSE', message: 'Database থেকে ভুল response পাওয়া গেছে।' });
  }
  return result;
}

export class InvokeTransport {
  constructor(invokeFn) {
    if (typeof invokeFn !== 'function') throw new TypeError('invokeFn must be a function');
    this.invokeFn = invokeFn;
  }

  async call(command, payload = {}) {
    try {
      return await this.invokeFn(command, payload);
    } catch (error) {
      throw normalizeServiceError(error);
    }
  }
}

export class MockTransport extends InvokeTransport {
  constructor(handlers = {}) {
    const calls = [];
    super(async (command, payload) => {
      calls.push({ command, payload });
      if (!Object.prototype.hasOwnProperty.call(handlers, command)) {
        throw { code: 'MOCK_COMMAND_MISSING', message: `No mock for ${command}` };
      }
      const handler = handlers[command];
      return typeof handler === 'function' ? handler(payload) : structuredClone(handler);
    });
    this.calls = calls;
  }
}
