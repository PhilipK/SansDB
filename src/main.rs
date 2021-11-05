use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard};

use std::thread;
use std::sync::mpsc::sync_channel;

use std::sync::mpsc::SyncSender;

#[derive(Deserialize,Serialize,Clone)]
enum TestCommand{
    Increment,
    Deceremnt
}


fn main() {
    let db = SansDB::<TestCommand,i32>::new(0,|command,data|{
        match command{
            TestCommand::Increment => *data += 1,
            TestCommand::Deceremnt => *data -= 1,
        }
    });

    let arc_db = Arc::new(db);
    let capacity = 10000;   
    let mut handles = Vec::with_capacity(capacity);
    let before = std::time::Instant::now();
    for _i in 0..capacity{
        let arc_db_here = arc_db.clone();
        handles.push(std::thread::spawn(move ||{
            arc_db_here.write(TestCommand::Increment);
            arc_db_here.write(TestCommand::Deceremnt);
            arc_db_here.write(TestCommand::Increment);
            let data = arc_db_here.read().unwrap();    
            if *data % 100 == 0 {println!("{}",data);}
        }));

    }
    for handle in handles {
        handle.join().unwrap();
    }
    //thread::sleep(Duration::from_millis(1));
    let data = arc_db.read().unwrap();
    println!("{}",data);
    println!("{:?}", before.elapsed());
}

struct SansDB<Command,Data> where
Command : Sync + Send + Clone + Serialize  ,
Data: Sync + Send + Serialize + DeserializeOwned {
   data: Arc<RwLock<Data>>,
   sender: SyncSender<Command>,
}

impl<Command,Data> SansDB<Command,Data> 
where Command : 'static + Sync + Send + Clone + Serialize + DeserializeOwned,
Data: 'static + Sync + Send + Serialize +  DeserializeOwned {

    pub fn read(&self) -> LockResult<RwLockReadGuard<'_,Data>>{
        self.data.read()
    }

    pub fn write(&self, command:Command) { 
        //remove unwrap
        self.sender.send(command).unwrap();

    }

    pub fn new(data:Data, command_handler: fn(&Command, &mut Data) -> ()) -> Self{
        // 0 means, lock on writes untill a handler is ready
        let (sender,receiver) = sync_channel::<Command>(1000);
        let data = Arc::new(RwLock::new(data)); 
        let inner_data = data.clone();
        thread::spawn(move || {
            loop {
                let command = receiver.recv();
                if command.is_err(){
                    return;
                }
                let command = command.unwrap();
                let write_lock = inner_data.write();
                let mut write_guard = write_lock.unwrap();
                (command_handler)(&command,&mut write_guard);
            }
        });
        Self{
            data,
            sender,
        }
    }
}

