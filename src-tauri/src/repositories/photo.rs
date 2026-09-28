use crate::{db::Database,error::{AppError,AppResult}};
use rusqlite::params;
use sha2::{Digest,Sha256};
use std::path::{Path,PathBuf};
use uuid::Uuid;

pub struct PhotoRepository<'a>{pub db:&'a Database,pub root:&'a Path}
impl PhotoRepository<'_>{
 pub fn import(&self,student_id:&str,file_name:&str,mime:&str,bytes:&[u8])->AppResult<String>{
  if bytes.is_empty()||bytes.len()>10*1024*1024{return Err(AppError::Validation("ছবির আকার ১ বাইট থেকে ১০ MB-এর মধ্যে হতে হবে।".into()));}
  let ext=match mime{"image/jpeg"=>"jpg","image/png"=>"png","image/webp"=>"webp",_=>return Err(AppError::Validation("শুধু JPG, PNG বা WEBP ছবি গ্রহণযোগ্য।".into()))};
  let magic_ok=match ext{"jpg"=>bytes.starts_with(&[0xff,0xd8,0xff]),"png"=>bytes.starts_with(b"\x89PNG\r\n\x1a\n"),"webp"=>bytes.len()>12&&&bytes[0..4]==b"RIFF"&&&bytes[8..12]==b"WEBP",_=>false};if !magic_ok{return Err(AppError::Validation("ছবি ফাইলটি সঠিক নয়।".into()));}
  let photo_id=Uuid::new_v4().to_string();let dir=self.root.join("students").join(student_id);std::fs::create_dir_all(&dir)?;let final_path=dir.join(format!("{}.{}",photo_id,ext));let temp=dir.join(format!(".{}.tmp",photo_id));std::fs::write(&temp,bytes)?;let hash=format!("{:x}",Sha256::digest(bytes));let relative=PathBuf::from("photos").join("students").join(student_id).join(format!("{}.{}",photo_id,ext)).to_string_lossy().replace('\\',"/");std::fs::rename(&temp,&final_path)?;
  let result=self.db.transaction(|tx|{let exists:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM students WHERE id=?1 AND deleted_at IS NULL)",[student_id],|r|r.get(0))?;if !exists{return Err(AppError::Validation("শিক্ষার্থী পাওয়া যায়নি।".into()));}tx.execute("UPDATE student_photos SET is_primary=0,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE student_id=?1 AND deleted_at IS NULL",[student_id])?;tx.execute("INSERT INTO student_photos(id,student_id,file_name,relative_path,mime_type,byte_size,sha256,is_primary,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now'))",params![photo_id,student_id,file_name,relative,mime,bytes.len() as i64,hash])?;Ok(())});if let Err(e)=result{let _=std::fs::remove_file(&final_path);return Err(e)}Ok(final_path.to_string_lossy().into_owned())
 }
}
