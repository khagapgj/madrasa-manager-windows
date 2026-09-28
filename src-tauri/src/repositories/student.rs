use crate::{db::Database, domain::student::{StudentDto, StudentInput, StudentListRequest, StudentPage}, error::{AppError, AppResult}};
use rusqlite::{named_params, params, Connection, OptionalExtension, Row, Transaction};
use std::collections::BTreeMap;
use uuid::Uuid;

pub struct StudentRepository<'a> { pub db: &'a Database }

fn clean(value: Option<String>) -> Option<String> {
    value.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}
fn validate(input: &StudentInput) -> AppResult<()> {
    if input.name.trim().is_empty() { return Err(AppError::Validation("শিক্ষার্থীর নাম প্রয়োজন।".into())); }
    if input.class_id.as_deref().unwrap_or("").trim().is_empty() && input.class_name.as_deref().unwrap_or("").trim().is_empty() {
        return Err(AppError::Validation("ক্লাস নির্বাচন করুন।".into()));
    }
    let status = input.status.as_deref().unwrap_or("active");
    if !matches!(status, "active" | "disabled" | "inactive") { return Err(AppError::Validation("শিক্ষার্থীর অবস্থা সঠিক নয়।".into())); }
    Ok(())
}
fn now(conn: &Connection) -> AppResult<String> {
    Ok(conn.query_row("SELECT strftime('%Y-%m-%dT%H:%M:%fZ','now')", [], |r| r.get(0))?)
}
fn resolve_class(tx: &Transaction<'_>, input: &StudentInput, stamp: &str) -> AppResult<String> {
    if let Some(id) = clean(input.class_id.clone()) {
        let exists: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM classes WHERE id=?1 AND deleted_at IS NULL)", [&id], |r| r.get(0))?;
        if !exists { return Err(AppError::Validation("নির্বাচিত ক্লাস পাওয়া যায়নি।".into())); }
        return Ok(id);
    }
    let name = input.class_name.as_deref().unwrap_or("").trim();
    if let Some(id) = tx.query_row("SELECT id FROM classes WHERE name=?1 COLLATE NOCASE AND deleted_at IS NULL", [name], |r| r.get(0)).optional()? { return Ok(id); }
    let id = Uuid::new_v4().to_string();
    let order: i64 = tx.query_row("SELECT COALESCE(MAX(display_order),-1)+1 FROM classes WHERE deleted_at IS NULL", [], |r| r.get(0))?;
    tx.execute("INSERT INTO classes(id,name,display_order,status,created_at,updated_at) VALUES(?1,?2,?3,'active',?4,?4)", params![id,name,order,stamp])?;
    Ok(id)
}
fn resolve_type(tx: &Transaction<'_>, input: &StudentInput, stamp: &str) -> AppResult<Option<String>> {
    if let Some(id) = clean(input.student_type_id.clone()) { return Ok(Some(id)); }
    let Some(name) = clean(input.student_type_name.clone()) else { return Ok(None) };
    if let Some(id) = tx.query_row("SELECT id FROM student_types WHERE name=?1 COLLATE NOCASE AND deleted_at IS NULL", [&name], |r| r.get(0)).optional()? { return Ok(Some(id)); }
    let id=Uuid::new_v4().to_string();
    let order:i64=tx.query_row("SELECT COALESCE(MAX(display_order),-1)+1 FROM student_types WHERE deleted_at IS NULL",[],|r|r.get(0))?;
    tx.execute("INSERT INTO student_types(id,name,display_order,is_system,created_at,updated_at) VALUES(?1,?2,?3,0,?4,?4)",params![id,name,order,stamp])?;
    Ok(Some(id))
}
fn save_custom(tx: &Transaction<'_>, student_id: &str, fields: &BTreeMap<String,String>, stamp: &str) -> AppResult<()> {
    tx.execute("DELETE FROM student_custom_field_values WHERE student_id=?1",[student_id])?;
    for (label,value) in fields.iter().filter(|(_,v)|!v.trim().is_empty()) {
        let field_id: String = match tx.query_row("SELECT id FROM custom_field_definitions WHERE field_key=?1 COLLATE NOCASE AND deleted_at IS NULL",[label],|r|r.get(0)).optional()? {
            Some(id)=>id,
            None=>{ let id=Uuid::new_v4().to_string(); let order:i64=tx.query_row("SELECT COALESCE(MAX(display_order),-1)+1 FROM custom_field_definitions WHERE deleted_at IS NULL",[],|r|r.get(0))?; tx.execute("INSERT INTO custom_field_definitions(id,field_key,label,field_type,display_order,is_visible,created_at,updated_at) VALUES(?1,?2,?2,'text',?3,1,?4,?4)",params![id,label,order,stamp])?; id }
        };
        tx.execute("INSERT INTO student_custom_field_values(id,student_id,field_id,value_text,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?5)",params![Uuid::new_v4().to_string(),student_id,field_id,value.trim(),stamp])?;
    }
    Ok(())
}
fn sync_fts(tx: &Transaction<'_>, id:&str) -> AppResult<()> {
    tx.execute("DELETE FROM students_fts WHERE student_id=?1",[id])?;
    tx.execute("INSERT INTO students_fts(student_id,student_code,admission_id,roll_no,name,father_name,mother_name,mobile,address) SELECT id,student_code,admission_id,roll_no,name,father_name,mother_name,mobile,COALESCE(address,notes) FROM students WHERE id=?1 AND deleted_at IS NULL",[id])?;
    Ok(())
}

fn insert_student(tx:&Transaction<'_>,mut input:StudentInput,id:&str)->AppResult<()> {
    validate(&input)?;let stamp=now(tx)?;let class_id=resolve_class(tx,&input,&stamp)?;let type_id=resolve_type(tx,&input,&stamp)?;let status=input.status.take().unwrap_or_else(||"active".into());
    tx.execute("INSERT INTO students(id,student_code,admission_id,roll_no,name,father_name,mother_name,mobile,class_id,student_type_id,notes,status,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?13)",params![id,clean(input.student_code.clone()),clean(input.admission_id.clone()),clean(input.roll_no.clone()),input.name.trim(),clean(input.father_name.clone()),clean(input.mother_name.clone()),clean(input.mobile.clone()),class_id,type_id,clean(input.notes.clone()),status,stamp])?;
    tx.execute("INSERT INTO student_status_history(id,student_id,status,effective_at,created_at) VALUES(?1,?2,?3,?4,?4)",params![Uuid::new_v4().to_string(),id,status,stamp])?;save_custom(tx,id,&input.custom_fields,&stamp)?;sync_fts(tx,id)?;Ok(())
}

impl StudentRepository<'_> {
    pub fn next_codes(&self, class_id: Option<&str>, class_name: Option<&str>, digits: i64) -> AppResult<(String,String)> {
        self.db.read(|conn| {
            let resolved = if let Some(id) = class_id { Some(id.to_string()) } else if let Some(name) = class_name { conn.query_row("SELECT id FROM classes WHERE name=?1 COLLATE NOCASE AND deleted_at IS NULL",[name],|r|r.get(0)).optional()? } else { None };
            let next_roll: i64 = match resolved { Some(id) => conn.query_row("SELECT COALESCE(MAX(CAST(roll_no AS INTEGER)),0)+1 FROM students WHERE class_id=?1 AND deleted_at IS NULL",[id],|r|r.get(0))?, None => 1 };
            let next_code: i64 = conn.query_row("SELECT COALESCE(MAX(CAST(student_code AS INTEGER)),0)+1 FROM students WHERE deleted_at IS NULL",[],|r|r.get(0))?;
            Ok((format!("{:02}",next_roll),format!("{:0width$}",next_code,width=digits.clamp(1,12) as usize)))
        })
    }

    pub fn create(&self, input: StudentInput) -> AppResult<StudentDto> {
        let id=Uuid::new_v4().to_string();self.db.transaction(|tx|insert_student(tx,input,&id))?;self.get(&id)?.ok_or_else(||AppError::Internal("created student not found".into()))
    }
    pub fn bulk_create(&self, inputs:Vec<StudentInput>)->AppResult<u64>{let count=inputs.len() as u64;self.db.transaction(|tx|{for input in inputs{let id=Uuid::new_v4().to_string();insert_student(tx,input,&id)?;}Ok(())})?;Ok(count)}

    pub fn update(&self,id:&str,mut input:StudentInput,expected:Option<&str>)->AppResult<StudentDto>{
        validate(&input)?;
        self.db.transaction(|tx|{ let stamp=now(tx)?; let class_id=resolve_class(tx,&input,&stamp)?; let type_id=resolve_type(tx,&input,&stamp)?; let status=input.status.take().unwrap_or_else(||"active".into());
            let old:(String,String)=tx.query_row("SELECT status,updated_at FROM students WHERE id=?1 AND deleted_at IS NULL",[id],|r|Ok((r.get(0)?,r.get(1)?))).optional()?.ok_or_else(||AppError::Validation("শিক্ষার্থী পাওয়া যায়নি।".into()))?;
            if expected.is_some_and(|e|e!=old.1){return Err(AppError::Validation("তথ্যটি অন্য জায়গা থেকে পরিবর্তিত হয়েছে। আবার খুলুন।".into()));}
            tx.execute("UPDATE students SET student_code=?2,admission_id=?3,roll_no=?4,name=?5,father_name=?6,mother_name=?7,mobile=?8,class_id=?9,student_type_id=?10,notes=?11,status=?12,updated_at=?13 WHERE id=?1",params![id,clean(input.student_code.clone()),clean(input.admission_id.clone()),clean(input.roll_no.clone()),input.name.trim(),clean(input.father_name.clone()),clean(input.mother_name.clone()),clean(input.mobile.clone()),class_id,type_id,clean(input.notes.clone()),status,stamp])?;
            if old.0!=status {tx.execute("INSERT INTO student_status_history(id,student_id,status,effective_at,created_at) VALUES(?1,?2,?3,?4,?4)",params![Uuid::new_v4().to_string(),id,status,stamp])?;}
            save_custom(tx,id,&input.custom_fields,&stamp)?;sync_fts(tx,id)?;Ok(())})?;
        self.get(id)?.ok_or_else(||AppError::Internal("updated student not found".into()))
    }

    pub fn soft_delete(&self,id:&str)->AppResult<()> {self.db.transaction(|tx|{let stamp=now(tx)?;let changed=tx.execute("UPDATE students SET deleted_at=?2,updated_at=?2 WHERE id=?1 AND deleted_at IS NULL",params![id,stamp])?;if changed==0{return Err(AppError::Validation("শিক্ষার্থী পাওয়া যায়নি।".into()));}sync_fts(tx,id)?;Ok(())})}
    pub fn restore(&self,id:&str)->AppResult<()> {self.db.transaction(|tx|{let stamp=now(tx)?;tx.execute("UPDATE students SET deleted_at=NULL,updated_at=?2 WHERE id=?1",params![id,stamp])?;sync_fts(tx,id)?;Ok(())})}
    pub fn change_status(&self,id:&str,status:&str)->AppResult<StudentDto>{if !matches!(status,"active"|"disabled"|"inactive"){return Err(AppError::Validation("অবস্থা সঠিক নয়।".into()));}self.db.transaction(|tx|{let stamp=now(tx)?;tx.execute("UPDATE students SET status=?2,updated_at=?3 WHERE id=?1 AND deleted_at IS NULL",params![id,status,stamp])?;tx.execute("INSERT INTO student_status_history(id,student_id,status,effective_at,created_at) VALUES(?1,?2,?3,?4,?4)",params![Uuid::new_v4().to_string(),id,status,stamp])?;Ok(())})?;self.get(id)?.ok_or_else(||AppError::Validation("শিক্ষার্থী পাওয়া যায়নি।".into()))}
    pub fn bulk_soft_delete(&self,ids:&[String])->AppResult<u64>{self.db.transaction(|tx|{let stamp=now(tx)?;let mut count=0;for id in ids{count+=tx.execute("UPDATE students SET deleted_at=?2,updated_at=?2 WHERE id=?1 AND deleted_at IS NULL",params![id,stamp])?;sync_fts(tx,id)?;}Ok(count as u64)})}
    pub fn bulk_move(&self,ids:&[String],class_id:&str)->AppResult<u64>{self.db.transaction(|tx|{let exists:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM classes WHERE id=?1 AND deleted_at IS NULL)",[class_id],|r|r.get(0))?;if !exists{return Err(AppError::Validation("ক্লাস পাওয়া যায়নি।".into()));}let stamp=now(tx)?;let mut count=0;for id in ids{count+=tx.execute("UPDATE students SET class_id=?2,updated_at=?3 WHERE id=?1 AND deleted_at IS NULL",params![id,class_id,stamp])?;}Ok(count as u64)})}

    pub fn get(&self,id:&str)->AppResult<Option<StudentDto>>{self.db.read(|conn|{let mut stmt=conn.prepare(&Self::select_sql("s.id=:id"))?;let mut rows=stmt.query(named_params!{":id":id})?;match rows.next()?{Some(row)=>Ok(Some(self.row_to_dto(conn,row)?)),None=>Ok(None)}})}

    pub fn list(&self,r:&StudentListRequest)->AppResult<StudentPage>{
        let page=r.page.max(1);let size=r.page_size.clamp(1,100);let offset=(page-1)*size;let query=r.query.as_deref().unwrap_or("").trim();let fts=if query.is_empty(){"__empty__".to_string()}else{query.split_whitespace().map(|t|format!("\"{}\"*",t.replace('"',""))).collect::<Vec<_>>().join(" AND ")};
        let class_id=r.class_id.as_deref().unwrap_or("");let class_name=r.class_name.as_deref().unwrap_or("");let status=r.status.as_deref().unwrap_or("");
        let where_sql="s.deleted_at IS NULL AND (:class_id='' OR s.class_id=:class_id) AND (:class_name='' OR c.name=:class_name COLLATE NOCASE) AND (:status='' OR s.status=:status) AND (:query='' OR s.id IN (SELECT student_id FROM students_fts WHERE students_fts MATCH :fts) OR EXISTS(SELECT 1 FROM student_custom_field_values cv WHERE cv.student_id=s.id AND cv.value_text LIKE '%'||:query||'%'))";
        let order=match r.sort_key.as_deref(){Some("name")=>"s.name COLLATE NOCASE",Some("roll")=>"CAST(s.roll_no AS INTEGER),s.roll_no",Some("date")=>"s.created_at",_=>"CAST(s.student_code AS INTEGER),s.student_code"};let direction=if r.sort_direction.as_deref()==Some("desc"){"DESC"}else{"ASC"};
        self.db.read(|conn|{let total:i64=conn.query_row(&format!("SELECT COUNT(*) FROM students s JOIN classes c ON c.id=s.class_id WHERE {where_sql}"),named_params!{":class_id":class_id,":class_name":class_name,":status":status,":query":query,":fts":fts},|row|row.get(0))?;
            let sql=format!("{} WHERE {where_sql} ORDER BY {order} {direction},s.id ASC LIMIT :limit OFFSET :offset",Self::select_sql("1=1").replace(" WHERE 1=1",""));let mut stmt=conn.prepare(&sql)?;let mut rows=stmt.query(named_params!{":class_id":class_id,":class_name":class_name,":status":status,":query":query,":fts":fts,":limit":size,":offset":offset})?;let mut items=Vec::new();while let Some(row)=rows.next()?{items.push(self.row_to_dto(conn,row)?);}Ok(StudentPage{items,page,page_size:size,total,has_next:offset+size<total})})
    }

    fn select_sql(predicate:&str)->String{format!("SELECT s.id,s.student_code,s.admission_id,s.roll_no,s.name,s.father_name,s.mother_name,s.mobile,s.class_id,c.name,s.student_type_id,st.name,s.notes,s.status,p.relative_path,s.updated_at FROM students s JOIN classes c ON c.id=s.class_id LEFT JOIN student_types st ON st.id=s.student_type_id LEFT JOIN student_photos p ON p.student_id=s.id AND p.is_primary=1 AND p.deleted_at IS NULL WHERE {predicate} AND s.deleted_at IS NULL")}
    fn row_to_dto(&self,conn:&Connection,row:&Row<'_>)->AppResult<StudentDto>{let id:String=row.get(0)?;let mut fields=BTreeMap::new();let mut stmt=conn.prepare("SELECT d.label,v.value_text FROM student_custom_field_values v JOIN custom_field_definitions d ON d.id=v.field_id WHERE v.student_id=?1 AND d.deleted_at IS NULL ORDER BY d.display_order")?;let values=stmt.query_map([&id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?)))?;for value in values{let(k,v)=value?;fields.insert(k,v);}Ok(StudentDto{uid:id,id:row.get(1)?,admission_id:row.get(2)?,roll:row.get(3)?,name:row.get(4)?,fathers_name:row.get(5)?,mothers_name:row.get(6)?,mobile:row.get(7)?,class_id:row.get(8)?,class:row.get(9)?,student_type_id:row.get(10)?,student_type:row.get(11)?,notes:row.get(12)?,status:row.get(13)?,custom_fields:fields,photo_path:row.get(14)?,updated_at:row.get(15)?})}
}
