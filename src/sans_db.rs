
use serde::{Serialize, de::DeserializeOwned};
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard};
use std::thread;
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::SyncSender;
use std::time::SystemTime;


pub struct SansDB<Command,Data> where
Command : Sync + Send + Clone + Serialize  ,
Data: Sync + Send + Serialize + DeserializeOwned {
    data: Arc<RwLock<Data>>,
    sender: SyncSender<InnerCommand<Command>>,
    write_handle : Option<thread::JoinHandle<()>>,
}

impl <Command,Data> Drop for SansDB<Command,Data> where
Command : Sync + Send + Clone + Serialize  ,
Data: Sync + Send + Serialize + DeserializeOwned {
    fn drop(&mut self) {
        self.sender.send(InnerCommand::Stop).unwrap();
        self.write_handle.take().unwrap().join().unwrap();
    }
}


enum InnerCommand<Command> {
    Stop,
    Actual(Command)
}


impl<Command,Data> SansDB<Command,Data> 
where Command : 'static + Sync + Send + Clone + Serialize + DeserializeOwned,
      Data: 'static + Sync + Send + Serialize +  DeserializeOwned {


          pub fn restore(mut data:Data, storage_location:&Path, command_handler: fn(&Command, &mut Data) -> ()) -> Self{
              //todo get rid of unwrap
              for entry in std::fs::read_dir(storage_location).unwrap() {
                  let entry = entry.unwrap();
                  let path = entry.path();
                  if path.is_file(){
                      let bytes = std::fs::read(path).unwrap();
                      let command: Command = bincode::deserialize(&bytes).unwrap();
                      (command_handler)(&command,&mut data);
                  }
              }
              Self::new(data,storage_location,command_handler)
          }

          pub fn read(&self) -> LockResult<RwLockReadGuard<'_,Data>>{
              self.data.read()
          }

          pub fn write(&self, command:Command) { 
              //remove unwrap
              let command = InnerCommand::Actual(command);
              self.sender.send(command).unwrap();
          }


          pub fn new(data:Data, storage_location:&Path, command_handler: fn(&Command, &mut Data) -> ())  -> Self{
              // 0 means, lock on writes untill a handler is ready
              let (sender,receiver) = sync_channel::<InnerCommand<Command>>(1000);
              let data = Arc::new(RwLock::new(data)); 
              let inner_data = data.clone();

              let storage_path_buff = storage_location.to_path_buf();
              let write_handle = thread::spawn(move || {
                  loop {
                     let command = receiver.recv();
                      if command.is_err(){
                          // this happens if all senders are gone
                          return;
                      }
                      match command.unwrap(){
                        InnerCommand::Stop => return,
                        InnerCommand::Actual(command) => {
                          let serialize_command = command.clone();

                          let storage_path_buff = storage_path_buff.to_path_buf();
                          let ser_handle = thread::spawn(move || {
                              let data: Vec<u8> = bincode::serialize(&serialize_command).unwrap();
                              let now = SystemTime::now();
                              use std::time:: UNIX_EPOCH;
                              let since_the_epoch = now
                                  .duration_since(UNIX_EPOCH)
                                  .expect("Time went backwards");
                              let file_name = format!("{:?}", since_the_epoch.as_nanos());
                              use std::io::prelude::*;
                              let mut file = File::create(storage_path_buff.join(file_name)).unwrap();
                              //get rid of this unwrap
                              file.write_all(&data).unwrap();
                          });
                          {
                              let write_lock = inner_data.write();
                              let mut write_guard = write_lock.unwrap();
                              (command_handler)(&command,&mut write_guard);
                          }
                          ser_handle.join().unwrap();

                      },
                    }
                  }
              });
              Self{
                  data,
                  sender,
                  write_handle:Some(write_handle)
              }

          }
      }
