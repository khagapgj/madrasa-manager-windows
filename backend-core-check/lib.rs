#[path = "../src-tauri/src/error.rs"] pub mod error;
#[path = "../src-tauri/src/db/mod.rs"] pub mod db;
#[path = "../src-tauri/src/domain/mod.rs"] pub mod domain;
#[path = "../src-tauri/src/repositories/mod.rs"] pub mod repositories;

#[cfg(test)]
mod integration_tests {
    use super::{db::Database,domain::{class::ClassInput,student::{StudentInput,StudentListRequest}},repositories::{class::ClassRepository,photo::PhotoRepository,student::StudentRepository}};
    use std::collections::BTreeMap;

    fn student(name:&str,class_id:String)->StudentInput{StudentInput{student_code:Some("001".into()),admission_id:None,roll_no:Some("01".into()),name:name.into(),father_name:Some("পিতা".into()),mother_name:None,mobile:Some("+8801711111111".into()),class_id:Some(class_id),class_name:None,student_type_id:None,student_type_name:Some("আবাসিক".into()),notes:None,status:Some("active".into()),custom_fields:BTreeMap::from([("রক্তের গ্রুপ".into(),"A+".into())])}}

    #[test]
    fn student_class_crud_search_pagination_and_soft_delete(){
        let dir=tempfile::tempdir().unwrap();let db=Database::open(dir.path().join("test.sqlite")).unwrap();
        let classes=ClassRepository{db:&db};let class=classes.create(ClassInput{name:"হিফজ".into()}).unwrap();
        let students=StudentRepository{db:&db};let created=students.create(student("আব্দুল্লাহ",class.id.clone())).unwrap();assert_eq!(created.class,"হিফজ");let codes=students.next_codes(Some(&class.id),None,3).unwrap();assert_eq!(codes,("02".into(),"002".into()));assert_eq!(created.custom_fields.get("রক্তের গ্রুপ").unwrap(),"A+");
        let page=students.list(&StudentListRequest{query:Some("আব্দুল্লাহ".into()),class_id:Some(class.id.clone()),class_name:None,status:None,sort_key:Some("name".into()),sort_direction:Some("asc".into()),page:1,page_size:50}).unwrap();assert_eq!(page.total,1);assert_eq!(page.items[0].uid,created.uid);
        let mut changed=student("আব্দুল্লাহ আপডেট",class.id);changed.student_code=Some("002".into());let updated=students.update(&created.uid,changed,Some(&created.updated_at)).unwrap();assert_eq!(updated.name,"আব্দুল্লাহ আপডেট");
        students.change_status(&created.uid,"inactive").unwrap();
        let photo_root=dir.path().join("photos");let path=PhotoRepository{db:&db,root:&photo_root}.import(&created.uid,"student.png","image/png",b"\x89PNG\r\n\x1a\nminimal").unwrap();assert!(std::path::Path::new(&path).exists());assert!(students.get(&created.uid).unwrap().unwrap().photo_path.is_some());
        students.soft_delete(&created.uid).unwrap();assert!(students.get(&created.uid).unwrap().is_none());students.restore(&created.uid).unwrap();assert!(students.get(&created.uid).unwrap().is_some());
    }
}
