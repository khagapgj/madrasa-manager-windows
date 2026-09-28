import { assertPage, assertUuid, pageRequest } from './core.js';

class BaseService {
  constructor(transport) { this.transport = transport; }
  call(command, payload = {}) { return this.transport.call(command, payload); }
}

export class AppService extends BaseService {
  initialize() { return this.call('app_initialize'); }
  health() { return this.call('database_health'); }
}

export class StudentService extends BaseService {
  nextCodes({ classId = null, className = null, digits = 3 } = {}) { return this.call('student_next_codes', { classId, className, digits }); }
  create(data) { return this.call('student_create', { input: data }); }
  update(id, data, expectedUpdatedAt = null) { return this.call('student_update', { id: assertUuid(id), input: data, expectedUpdatedAt }); }
  softDelete(id) { return this.call('student_soft_delete', { id: assertUuid(id) }); }
  bulkCreate(inputs) { return this.call('student_bulk_create', { inputs }); }
  bulkSoftDelete(studentIds) { return this.call('student_bulk_soft_delete', { studentIds: studentIds.map(id => assertUuid(id)) }); }
  restore(id) { return this.call('student_restore', { id: assertUuid(id) }); }
  getById(id) { return this.call('student_get', { id: assertUuid(id) }); }
  changeStatus(id, status) { return this.call('student_change_status', { id: assertUuid(id), status }); }
  async list(filters = {}) { return assertPage(await this.call('student_list', { request: pageRequest(filters) })); }
  search(query, filters = {}) { return this.list({ ...filters, query }); }
  bulkMove(studentIds, classId) { return this.call('student_bulk_move', { studentIds: studentIds.map(id => assertUuid(id)), classId: assertUuid(classId, 'classId') }); }
}

export class ClassService extends BaseService {
  list(includeDeleted = false) { return this.call('class_list', { includeDeleted }); }
  create(data) { return this.call('class_create', { input: data }); }
  rename(id, name) { return this.call('class_rename', { id: assertUuid(id), name }); }
  reorder(orderedIds) { return this.call('class_reorder', { orderedIds: orderedIds.map(id => assertUuid(id)) }); }
  softDelete(id) { return this.call('class_soft_delete', { id: assertUuid(id) }); }
}

export class FeeService extends BaseService {
  listTypes(filters = {}) { return this.call('fee_type_list', { request: pageRequest(filters) }).then(assertPage); }
  getType(id) { return this.call('fee_type_get', { id: assertUuid(id) }); }
  createType(data) { return this.call('fee_type_create', { input: data }); }
  updateType(id, data) { return this.call('fee_type_update', { id: assertUuid(id), input: data }); }
  softDeleteType(id, policy = 'preserve_history') { return this.call('fee_type_soft_delete', { id: assertUuid(id), policy }); }
  getStudentSummary(studentId, asOf = null) { return this.call('fee_student_summary', { studentId: assertUuid(studentId), asOf }); }
  setOverride(input) { return this.call('fee_set_override', { input }); }
  applyWaiver(input) { return this.call('fee_apply_waiver', { input }); }
}

export class PaymentService extends BaseService {
  preview(input) { return this.call('payment_preview', { input }); }
  create(input) { return this.call('payment_create', { input }); }
  reverse(id, reason) { return this.call('payment_reverse', { id: assertUuid(id), reason }); }
  list(filters = {}) { return this.call('payment_list', { request: pageRequest(filters) }).then(assertPage); }
  getReceipt(id) { return this.call('payment_receipt_get', { id: assertUuid(id) }); }
}

function voucherFacade(prefix) {
  return class extends BaseService {
    list(filters = {}) { return this.call(`${prefix}_list`, { request: pageRequest(filters) }).then(assertPage); }
    getById(id) { return this.call(`${prefix}_get`, { id: assertUuid(id) }); }
    create(data) { return this.call(`${prefix}_create`, { input: data }); }
    update(id, data) { return this.call(`${prefix}_update`, { id: assertUuid(id), input: data }); }
    softDelete(id) { return this.call(`${prefix}_soft_delete`, { id: assertUuid(id) }); }
    summary(filters = {}) { return this.call(`${prefix}_summary`, { filters }); }
    listTypes() { return this.call(`${prefix}_type_list`); }
  };
}
export const ExpenseService = voucherFacade('expense');
export const IncomeService = voucherFacade('income');

export class SettingsService extends BaseService {
  get(key) { return this.call('settings_get', { key }); }
  set(key, value) { return this.call('settings_set', { key, value }); }
  getInstituteInfo() { return this.get('institute.info'); }
  updateInstituteInfo(value) { return this.set('institute.info', value); }
}

export class PhotoService extends BaseService {
  async importForStudent(studentId, file) {
    assertUuid(studentId);
    if (!(file instanceof File)) throw new TypeError('file must be a File');
    const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
    return this.call('photo_import', { input: { studentId, fileName: file.name, mimeType: file.type, bytes } });
  }
}

export class DashboardService extends BaseService {
  getSummary(filters = {}) { return this.call('dashboard_summary', { filters }); }
  getClassRows(filters = {}) { return this.call('dashboard_class_rows', { request: pageRequest(filters) }).then(assertPage); }
  getExpenseBreakdown(filters = {}) { return this.call('dashboard_expense_breakdown', { filters }); }
}

export class BackupService extends BaseService {
  create(destination) { return this.call('backup_create', { destination }); }
  validate(path) { return this.call('backup_validate', { path }); }
  restore(path, confirmationToken) { return this.call('backup_restore', { path, confirmationToken }); }
}

export class MigrationService extends BaseService {
  validateLegacyFile(path) { return this.call('legacy_import_validate', { path }); }
  commitLegacyImport(importId, mode = 'replace') { return this.call('legacy_import_commit', { importId, mode }); }
}

export function createServices(transport) {
  return Object.freeze({
    app: new AppService(transport), students: new StudentService(transport), classes: new ClassService(transport),
    fees: new FeeService(transport), payments: new PaymentService(transport), expenses: new ExpenseService(transport),
    incomes: new IncomeService(transport), settings: new SettingsService(transport), photos: new PhotoService(transport),
    dashboard: new DashboardService(transport), backup: new BackupService(transport), migration: new MigrationService(transport)
  });
}
