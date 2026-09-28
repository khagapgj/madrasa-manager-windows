import sqlite3, uuid
from pathlib import Path

schema=(Path(__file__).parents[1]/'src-tauri/migrations/001_initial.sql').read_text()
con=sqlite3.connect(':memory:')
con.execute('PRAGMA foreign_keys=ON')
con.executescript(schema)
now='2026-09-28T10:00:00Z'
class_id=str(uuid.uuid4()); student_id=str(uuid.uuid4())
con.execute("INSERT INTO classes(id,name,display_order,status,created_at,updated_at) VALUES(?,?,0,'active',?,?)",(class_id,'হিফজ',now,now))
con.execute("INSERT INTO students(id,student_code,roll_no,name,mobile,class_id,status,created_at,updated_at) VALUES(?,?,?,?,?,?,'active',?,?)",(student_id,'001','01','আব্দুল্লাহ','+8801711111111',class_id,now,now))
con.execute("INSERT INTO students_fts(student_id,student_code,roll_no,name,mobile) VALUES(?,?,?,?,?)",(student_id,'001','01','আব্দুল্লাহ','+8801711111111'))
assert con.execute("SELECT student_id FROM students_fts WHERE students_fts MATCH ?",('"আব্দুল্লাহ"*',)).fetchone()[0]==student_id
rows=con.execute("SELECT id FROM students WHERE class_id=? ORDER BY CAST(student_code AS INTEGER) LIMIT 50 OFFSET 0",(class_id,)).fetchall()
assert rows==[(student_id,)]
con.commit()
# FK blocks accidental class deletion while student exists.
try:
    con.execute('DELETE FROM classes WHERE id=?',(class_id,))
    raise AssertionError('class FK delete unexpectedly succeeded')
except sqlite3.IntegrityError:
    con.rollback()
# Transaction rollback prevents partial student/status save.
before=con.execute('SELECT COUNT(*) FROM students').fetchone()[0]
try:
    with con:
        bad=str(uuid.uuid4())
        con.execute("INSERT INTO students(id,name,class_id,status,created_at,updated_at) VALUES(?,? ,?,'active',?,?)",(bad,'Rollback Test',class_id,now,now))
        con.execute("INSERT INTO student_status_history(id,student_id,status,effective_at,created_at) VALUES(?,?, 'BROKEN',?,?)",(str(uuid.uuid4()),bad,now,now))
except sqlite3.IntegrityError:
    pass
assert con.execute('SELECT COUNT(*) FROM students').fetchone()[0]==before
print('STUDENT_CLASS_SQL_OK')
