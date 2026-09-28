import test from 'node:test';
import assert from 'node:assert/strict';
import { MockTransport, ServiceError } from '../../src/services/core.js';
import { createServices } from '../../src/services/facades.js';

const ID = '550e8400-e29b-41d4-a716-446655440000';
const CLASS_ID = '6ba7b810-9dad-41d1-80b4-00c04fd430c8';
const emptyPage = { items: [], page: 1, pageSize: 50, total: 0, hasNext: false };

test('student list enforces paginated contract and default size', async () => {
  const transport = new MockTransport({ student_list: ({ request }) => ({ ...emptyPage, page: request.page, pageSize: request.pageSize }) });
  const services = createServices(transport);
  const result = await services.students.search('রহিম', { classId: CLASS_ID });
  assert.deepEqual(result, emptyPage);
  assert.deepEqual(transport.calls[0], { command: 'student_list', payload: { request: { query: 'রহিম', classId: CLASS_ID, page: 1, pageSize: 50 } } });
});

test('student get rejects non-UUID before IPC', async () => {
  const transport = new MockTransport({});
  const services = createServices(transport);
  assert.throws(() => services.students.getById('student name'), error => error instanceof ServiceError && error.code === 'INVALID_ID');
  assert.equal(transport.calls.length, 0);
});

test('payment create uses one command payload (transaction boundary)', async () => {
  const transport = new MockTransport({ payment_create: ({ input }) => ({ id: ID, ...input }) });
  const services = createServices(transport);
  const input = { studentId: ID, amountMinor: 10000, allocations: [] };
  const result = await services.payments.create(input);
  assert.equal(result.id, ID);
  assert.deepEqual(transport.calls, [{ command: 'payment_create', payload: { input } }]);
});

test('settings facade never accesses browser storage', async () => {
  const transport = new MockTransport({ settings_get: { name: 'মাদরাসা' }, settings_set: null });
  const services = createServices(transport);
  assert.deepEqual(await services.settings.getInstituteInfo(), { name: 'মাদরাসা' });
  await services.settings.updateInstituteInfo({ name: 'নতুন নাম' });
  assert.deepEqual(transport.calls.map(c => c.command), ['settings_get', 'settings_set']);
});

test('backend errors become stable ServiceError', async () => {
  const transport = new MockTransport({ database_health: () => { throw { code: 'DATABASE_BUSY', message: 'আবার চেষ্টা করুন', retryable: true, traceId: ID }; } });
  const services = createServices(transport);
  await assert.rejects(() => services.app.health(), error => error instanceof ServiceError && error.code === 'DATABASE_BUSY' && error.retryable === true);
});

test('class CRUD facade uses permanent UUID commands', async () => {
  const transport = new MockTransport({ class_create: { id: CLASS_ID, name: 'হিফজ' }, class_rename: { id: CLASS_ID, name: 'নাজেরা' }, class_soft_delete: null });
  const services = createServices(transport);
  await services.classes.create({ name: 'হিফজ' });
  await services.classes.rename(CLASS_ID, 'নাজেরা');
  await services.classes.softDelete(CLASS_ID);
  assert.deepEqual(transport.calls.map(c => c.command), ['class_create','class_rename','class_soft_delete']);
});

test('student status and bulk operations remain single transaction commands', async () => {
  const transport = new MockTransport({ student_change_status: { uid: ID, status: 'inactive' }, student_bulk_soft_delete: 1, student_bulk_move: 1 });
  const services = createServices(transport);
  await services.students.changeStatus(ID, 'inactive');
  await services.students.bulkSoftDelete([ID]);
  await services.students.bulkMove([ID], CLASS_ID);
  assert.deepEqual(transport.calls.map(c => c.command), ['student_change_status','student_bulk_soft_delete','student_bulk_move']);
});

test('all required facade groups exist', () => {
  const services = createServices(new MockTransport({}));
  assert.deepEqual(Object.keys(services), ['app','students','classes','fees','payments','expenses','incomes','settings','photos','dashboard','backup','migration']);
  assert.ok(Object.isFrozen(services));
});
