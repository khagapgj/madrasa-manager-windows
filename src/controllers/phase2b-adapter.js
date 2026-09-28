/* Phase 2B compatibility controller. No styles or visual markup are defined here. */
(function () {
  const state = { services: null, classRows: [], classByName: new Map(), page: 1, pageSize: 50, total: 0, hasNext: false, loading: false, observer: null };
  const originalRenderStudents = window.renderStudents;
  const originalSetFilter = window.setFilter;
  const originalGetStudentPhoto = window.getStudentPhoto;

  function errorMessage(error) { return error?.message || 'Data সংরক্ষণ করা সম্ভব হয়নি। আবার চেষ্টা করুন।'; }
  function fail(error) { console.error('[Phase 2B]', error); if (typeof showToast === 'function') showToast(errorMessage(error), 'error'); }
  function photoUrl(path) { return path && window.__TAURI__?.core?.convertFileSrc ? window.__TAURI__.core.convertFileSrc(path) : path; }
  function hydrate(row) {
    return {
      ...row,
      uid: row.uid,
      id: row.id || '', roll: row.roll || '', fathersName: row.fathersName || '', mothersName: row.mothersName || '',
      class: row.class, classId: row.classId, type: row.type || '', notes: row.notes || '', custom_fields: row.customFields || {},
      photoUrl: photoUrl(row.photoPath), statusHistory: row.statusHistory || []
    };
  }
  function queryText() { return (document.getElementById('search-input')?.value || document.getElementById('student-search')?.value || '').trim(); }
  function classIdFor(name) { return state.classByName.get((name || '').toLowerCase())?.id || null; }
  function sortRequest() { const c = window.studentSortConfig || { key: 'id', direction: 'asc' }; return { sortKey: c.key, sortDirection: c.direction }; }

  async function loadClasses() {
    state.classRows = await state.services.classes.list(false);
    state.classByName = new Map(state.classRows.map(row => [row.name.toLowerCase(), row]));
    classes = state.classRows.map(row => row.name);
    if (typeof renderFilters === 'function') renderFilters();
    if (typeof renderClassFilters === 'function') renderClassFilters();
    if (typeof renderFeesClassFilters === 'function') renderFeesClassFilters();
    if (typeof updateClassSuggestions === 'function') updateClassSuggestions();
  }

  async function refreshStudents(reset = true) {
    if (!state.services || state.loading || (!reset && !state.hasNext)) return;
    state.loading = true;
    try {
      const nextPage = reset ? 1 : state.page + 1;
      const response = await state.services.students.list({
        query: queryText(), classId: currentFilter && currentFilter !== 'All' ? classIdFor(currentFilter) : null,
        status: (typeof currentAdvStudentStatusFilter !== 'undefined' && currentAdvStudentStatusFilter !== 'all') ? currentAdvStudentStatusFilter : null,
        page: nextPage, pageSize: state.pageSize, ...sortRequest()
      });
      const rows = response.items.map(hydrate);
      students = reset ? rows : students.concat(rows.filter(row => !students.some(old => old.uid === row.uid)));
      state.page = response.page; state.total = response.total; state.hasNext = response.hasNext;
      originalRenderStudents();
      const count = document.getElementById('total-count'); if (count) count.textContent = `মোট ${state.total} জন শিক্ষার্থী`;
      installSentinel();
    } catch (error) { fail(error); }
    finally { state.loading = false; }
  }

  function installSentinel() {
    const grid = document.getElementById('student-grid'); if (!grid) return;
    document.getElementById('student-page-sentinel')?.remove();
    if (!state.hasNext) return;
    const sentinel = document.createElement('div'); sentinel.id = 'student-page-sentinel'; sentinel.setAttribute('aria-hidden', 'true'); sentinel.style.cssText = 'height:1px;width:100%;grid-column:1/-1;pointer-events:none;'; grid.appendChild(sentinel);
    state.observer?.disconnect();
    state.observer = new IntersectionObserver(entries => { if (entries.some(entry => entry.isIntersecting)) refreshStudents(false); }, { rootMargin: '300px' });
    state.observer.observe(sentinel);
  }

  function readStudentForm() {
    const custom = {};
    document.querySelectorAll('.custom-field-input').forEach(input => { const value = input.value.trim(); if (value) custom[input.dataset.field] = value; });
    const className = document.getElementById('inp-class').value.trim();
    return {
      studentCode: document.getElementById('inp-id').value.trim() || null,
      rollNo: document.getElementById('inp-roll').value.trim() || null,
      name: document.getElementById('inp-name').value.trim(),
      classId: classIdFor(className), className,
      mobile: typeof getMobileFromForm === 'function' ? getMobileFromForm() : document.getElementById('inp-mobile-number')?.value,
      fatherName: document.getElementById('inp-father')?.value.trim() || null,
      motherName: document.getElementById('inp-mother')?.value.trim() || null,
      notes: document.getElementById('inp-notes')?.value.trim() || null,
      studentTypeName: document.getElementById('inp-type')?.value || null,
      status: document.getElementById('inp-status')?.checked ? 'active' : 'disabled', customFields: custom
    };
  }

  async function calculateCodes(className) {
    if (!className) return;
    try {
      const result = await state.services.students.nextCodes({ classId: classIdFor(className), className, digits: Number(instituteInfo?.idLimit || 3) });
      const roll = document.getElementById('inp-roll'); const code = document.getElementById('inp-id');
      if (roll) roll.value = result.roll; if (code) code.value = result.studentCode;
    } catch (error) { fail(error); }
  }

  async function submitStudent(event) {
    event.preventDefault();
    const form = readStudentForm();
    if (!form.name || !form.mobile || !form.className) return showToast('অনুগ্রহ করে প্রয়োজনীয় তথ্য দিন');
    try {
      const uid = document.getElementById('edit-uid').value;
      const existing = uid ? students.find(s => s.uid === uid) : null;
      const saved = uid ? await state.services.students.update(uid, form, existing?.updatedAt || null) : await state.services.students.create(form);
      const photo = document.getElementById('inp-photo')?.files?.[0];
      if (photo) await state.services.photos.importForStudent(saved.uid, photo);
      await loadClasses(); await refreshStudents(true);
      if (typeof clearDraft === 'function') clearDraft();
      event.target.reset(); document.getElementById('edit-uid').value = '';
      document.getElementById('preview-photo')?.classList.add('hidden'); document.getElementById('preview-birth-cert')?.classList.add('hidden');
      if (document.getElementById('auto-gen-check')) document.getElementById('auto-gen-check').checked = false;
      if (typeof toggleAutoGenerate === 'function') toggleAutoGenerate();
      closeModal('student-modal');
      showToast(uid ? 'তথ্য আপডেট করা হয়েছে' : 'শিক্ষার্থী যোগ করা হয়েছে');
    } catch (error) { fail(error); }
  }

  async function syncPhotos(event) {
    const files = Array.from(event.target?.files || event.dataTransfer?.files || []); if (!files.length) return;
    let matched = 0; const unknown = [];
    try {
      for (const file of files) {
        const base = file.name.replace(/\.[^.]+$/, ''); const parts = base.split(/[ _.\-]+/).filter(Boolean); const studentCode = parts[0] || '';
        const result = await state.services.students.search(studentCode, { page: 1, pageSize: 25 });
        const row = result.items.find(item => String(item.id || '').toLowerCase() === studentCode.toLowerCase());
        if (row) { await state.services.photos.importForStudent(row.uid, file); matched++; }
        else unknown.push({ file, studentCode, name: parts.slice(1).join(' ').trim() });
      }
      if (unknown.length && confirm(`${matched} টি ছবি সিঙ্ক হয়েছে।\n\n${unknown.length} টি নতুন স্টুডেন্ট পাওয়া গেছে।\nআপনি কি যোগ করতে চান?`)) {
        let cls = currentFilter !== 'All' ? state.classByName.get(currentFilter.toLowerCase()) : state.classRows[0];
        if (!cls) cls = await state.services.classes.create({ name: 'Default' });
        for (const item of unknown) { if (!item.studentCode || !item.name) continue; const row = await state.services.students.create({ studentCode: item.studentCode, name: item.name, classId: cls.id, mobile: null, status: 'active', customFields: {} }); await state.services.photos.importForStudent(row.uid, item.file); matched++; }
        await loadClasses();
      }
      await refreshStudents(true); showToast(`${matched} টি ছবি স্থায়ীভাবে সংরক্ষণ করা হয়েছে।`);
    } catch (error) { fail(error); }
    finally { if (event.target) event.target.value = ''; }
  }

  async function deleteOne(uid) {
    if (!await customConfirm('আপনি কি নিশ্চিত যে আপনি এই শিক্ষার্থীকে মুছে ফেলতে চান?', 'শিক্ষার্থী মুছুন')) return;
    try { await state.services.students.softDelete(uid); closeStudentDetailPanel?.(); await refreshStudents(true); showToast('শিক্ষার্থী মুছে ফেলা হয়েছে'); } catch (error) { fail(error); }
  }
  async function changeStatus(uid) {
    const row = students.find(s => s.uid === uid); if (!row) return;
    const next = row.status === 'active' ? 'inactive' : 'active';
    try { await state.services.students.changeStatus(uid, next); await refreshStudents(true); showToast(`${row.name} এর স্ট্যাটাস আপডেট করা হয়েছে।`); } catch (error) { fail(error); }
  }
  async function deleteSelected() {
    if (!selectedStudents.length) return showToast('কোনো শিক্ষার্থী নির্বাচন করা হয়নি');
    if (!await customConfirm(`আপনি কি নির্বাচিত ${selectedStudents.length} জন শিক্ষার্থীকে মুছে ফেলতে চান?`, 'শিক্ষার্থী মুছুন')) return;
    try { await state.services.students.bulkSoftDelete(selectedStudents); selectedStudents = []; selectionMode = false; await refreshStudents(true); showToast('নির্বাচিত শিক্ষার্থী মুছে ফেলা হয়েছে'); } catch (error) { fail(error); }
  }
  async function moveSelected() {
    if (!selectedStudents.length) return showToast('কোনো শিক্ষার্থী নির্বাচন করা হয়নি');
    const value = await customPrompt(`${selectedStudents.length} জন শিক্ষার্থীকে কোন ক্লাসে সরাতে চান ?`, currentFilter !== 'All' ? currentFilter : '', 'ক্লাস স্থানান্তর');
    if (!value?.trim()) return;
    try {
      let cls = state.classByName.get(value.trim().toLowerCase()); if (!cls) cls = await state.services.classes.create({ name: value.trim() });
      await state.services.students.bulkMove(selectedStudents, cls.id); await loadClasses(); await refreshStudents(true); toggleSelectionMode?.(); showToast(`${selectedStudents.length} জন শিক্ষার্থী স্থানান্তর করা হয়েছে`);
    } catch (error) { fail(error); }
  }

  async function importStudentsFromPreview() {
    const mappings = Array.from(document.querySelectorAll('.column-mapping')).map(select => select.value); const hasHeader = document.getElementById('has-header')?.checked; const className = document.getElementById('import-target-class')?.value.trim();
    if (!className || !mappings.length || !pendingImportData?.length) return showToast('ইম্পোর্ট তথ্য সঠিক নয়।');
    try {
      let cls = state.classByName.get(className.toLowerCase()); if (!cls) cls = await state.services.classes.create({ name: className }); const inputs = [];
      for (const row of pendingImportData.slice(hasHeader ? 1 : 0)) {
        const input = { classId: cls.id, status: 'active', customFields: {} };
        mappings.forEach((type,index) => { const value=String(row[index]||'').trim(); if(!value||type==='ignore')return; if(type==='name')input.name=value;else if(type==='id')input.studentCode=value;else if(type==='roll')input.rollNo=value;else if(type==='mobile')input.mobile=value;else if(type==='fname')input.fatherName=value;else if(type==='mname')input.motherName=value;else if(type==='note')input.notes=value;else input.customFields[type]=value; });
        if (input.name) inputs.push(input);
      }
      if (!inputs.length) return showToast('কোনো বৈধ শিক্ষার্থী পাওয়া যায়নি।');
      await state.services.students.bulkCreate(inputs); await loadClasses(); await refreshStudents(true); closeModal('import-preview-modal'); document.getElementById('import-area').value=''; showToast(`${inputs.length} জন শিক্ষার্থী সফলভাবে ইম্পোর্ট করা হয়েছে!`);
    } catch (error) { fail(error); }
  }

  async function addClasses() {
    const raw = document.getElementById('new-class-names').value; const names = [...new Set(raw.split(/\n/).map(v => v.trim()).filter(Boolean))]; let count = 0, first = null;
    try { for (const name of names) { if (!state.classByName.has(name.toLowerCase())) { await state.services.classes.create({ name }); count++; first ||= name; } } await loadClasses(); closeModal('class-modal'); if (count) { showToast(`${count} টি ক্লাস যোগ করা হয়েছে`); if (count === 1) setFilter(first); } else if (raw.trim()) showToast('ক্লাসগুলো ইতিমধ্যে আছে'); } catch (error) { fail(error); }
  }
  async function renameOne(oldName) {
    const value = await customPrompt('ক্লাসের নাম পরিবর্তন করুন: ', oldName, 'ক্লাস এডিট'); if (!value?.trim() || value.trim() === oldName) return;
    const row = state.classByName.get(oldName.toLowerCase()); if (!row) return;
    try { await state.services.classes.rename(row.id, value.trim()); const wasCurrent = currentFilter === oldName; await loadClasses(); if (wasCurrent) currentFilter = value.trim(); renderFilters(); await refreshStudents(true); showToast('ক্লাসের নাম পরিবর্তন করা হয়েছে'); } catch (error) { fail(error); }
  }
  async function deleteOneClass(name) {
    const row = state.classByName.get(name.toLowerCase()); if (!row) return;
    if (!await customConfirm(`আপনি কি নিশ্চিত যে আপনি "${name}" ক্লাসটি মুছে ফেলতে চান ?`, 'ক্লাস মুছুন')) return;
    try { await state.services.classes.softDelete(row.id); if (currentFilter === name) currentFilter = 'All'; await loadClasses(); await refreshStudents(true); showToast(`"${name}" ক্লাস মুছে ফেলা হয়েছে`); } catch (error) { fail(error); }
  }
  async function saveOrder() {
    const ids = Array.from(document.querySelectorAll('#class-edit-list [data-index]')).map(item => state.classRows[Number(item.dataset.index)]?.id).filter(Boolean);
    try { await state.services.classes.reorder(ids); await loadClasses(); closeModal('edit-classes-modal'); showToast('ক্লাসের নতুন সিরিয়াল সেভ করা হয়েছে।'); } catch (error) { fail(error); }
  }

  function bind() {
    window.getFilteredStudents = () => students;
    window.renderStudents = () => refreshStudents(true);
    window.calculateAutoValues = calculateCodes;
    window.handleFormSubmit = submitStudent;
    window.handlePhotoSync = syncPhotos;
    window.deleteStudent = deleteOne;
    window.deleteSelectedStudents = deleteSelected;
    window.toggleStudentStatus = changeStatus;
    window.moveSelectedStudents = moveSelected;
    window.finalizeImport = importStudentsFromPreview;
    window.confirmAddClass = addClasses;
    window.editClass = renameOne;
    window.renameClass = () => renameOne(currentFilter);
    window.deleteClass = deleteOneClass;
    window.deleteCurrentClass = () => currentFilter !== 'All' ? deleteOneClass(currentFilter) : showToast('কোনো ক্লাস নির্বাচিত নেই');
    window.saveClassOrder = saveOrder;
    window.saveData = () => refreshStudents(true);
    window.saveClasses = () => loadClasses();
    window.getStudentPhoto = student => student?.photoUrl || originalGetStudentPhoto?.(student) || '';
    window.setFilter = function (name) { originalSetFilter(name); refreshStudents(true); };
    const search = () => { clearTimeout(search.timer); search.timer = setTimeout(() => refreshStudents(true), 250); };
    document.getElementById('search-input')?.addEventListener('input', search);
    document.getElementById('student-search')?.addEventListener('input', search);
    window.MadrasaPhase2B = Object.freeze({ refreshStudents, loadClasses });
  }

  document.addEventListener('madrasa:services-ready', async () => {
    state.services = window.AppServices; bind();
    try { await loadClasses(); await refreshStudents(true); } catch (error) { fail(error); }
  }, { once: true });
})();
