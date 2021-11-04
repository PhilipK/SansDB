use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};


fn main() {

    println!("Hello, world!");
}


struct SansDB<Command,Data> where
Command : Sync + Serialize + DeserializeOwned,
Data: Sync + Serialize +  DeserializeOwned {
    data: Arc<RwLock<Data>>,
    que: VecDeque<Command>
}

