use serde::{Deserialize, Serialize,};
use std::{path::Path, sync::Arc};

mod sans_db; 
use sans_db::*;
#[derive(Deserialize,Serialize,Clone)]
enum TestCommand{
    Increment,
    Deceremnt
}

#[derive(Deserialize,Serialize,Clone)]
enum SmsCommand{
    Add{message:String},
    Yeet
}
#[derive(Deserialize,Serialize,Clone,Default)]
struct OurData {
    pub sms:Vec<String>
}
fn main() {
    let before = std::time::Instant::now();
    let storage = Path::new("storage");
    let config = Configuration{
        sync_writes:false,
        block_writes:false,
        storage_path:None
    };
    let sms_len = {
        std::fs::create_dir_all(storage).unwrap();
        let sms_db = SansDB::<SmsCommand,OurData>::restore(OurData::default(),|command,data|{
            match command {
                SmsCommand::Add { message } => data.sms.push(message.clone()),
                SmsCommand::Yeet =>{ data.sms.pop();},
            }
        },config);
        let arc_db = Arc::new(sms_db);
        let capacity = 10000;   
        let mut handles = Vec::with_capacity(capacity);
        for _i in 0..capacity{
            let arc_db_here = arc_db.clone();
            handles.push(std::thread::spawn(move ||{
                arc_db_here.write(SmsCommand::Add{message:"hello".to_string()});
            }));
        }
        for handle in handles {
            handle.join().unwrap();
        }
        //thread::sleep(Duration::from_millis(1));
        let data = arc_db.read().unwrap();
        data.sms.len()
    };

    println!("{:?}", sms_len);
    println!("{:?}", before.elapsed());
}

