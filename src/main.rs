use std::{env, path::PathBuf};

use app_service::apps::entry;
use serde_json::to_string;
use xdgkit::{icon_finder};

fn main() {
   let args = env::args().collect::<Vec<String>>();
   let args = args.iter().map(|x| x.as_ref()).collect::<Vec<&str>>();
   // println!("参数列表: {:?}", args);

   match args[1] {
     "app" => { 
       let entry = entry::transform_entry();
       let mut layer = Vec::new();
       let mut app = Vec::new();
       for (i, ele) in entry.iter().enumerate() {
          app.push(ele);
          if (i+1) % 8 == 0 {
              layer.push(app);
              app  = Vec::new();
          } 
       }
       if app.len() > 0 {
           layer.push(app);
       }
       let json = to_string(&layer).unwrap();
       println!("{}", json);
     },
     "application" => {
        let icons = match icon_finder::find_icon("idea".to_string(), 48, 0) {
            Some(icon) => icon,
            None => PathBuf::new(),
        };
        println!("{:?}", icons);
     },
      _ => println!("{}", "[]"),
   };
}
