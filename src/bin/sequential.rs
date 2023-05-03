use std::{env, fs::File, io::{Read, BufReader}};
use std::io::Write;
use log::error;


fn main() ->std::io::Result<()>{
    let args :Vec<String> = env::args().collect();
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    if args.len() < 3 {
        println!("Usage: sequential <app> <filename> ..." );
        error!("Argument Wrong");
        return Ok(());
    }
    let  mapf ;
    let  reducef;
    match args[1].trim() {
        "wc" => {
            mapf = mr::apps::wc_m;
            reducef = mr::apps::wc_r;
        }
        _ => {
             error!("App Wrong");
             return Ok(());
        }
    }
    let mut intermediate = vec![];
    for filename in args[2..].iter() {
        let file  = File::open(filename)? ;      
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut  contents)?;
        let mut kva = mapf(filename,&contents);
        intermediate.append(&mut kva);
    }
    
    intermediate.sort_by(|a,b| a.key.partial_cmp(&b.key).unwrap());

    let oname = "mr-out-0";
    let mut ofile = File::create(oname)?;

    let mut i = 0;
    while i < intermediate.len() {
        let mut j = i+1;
        while j < intermediate.len() && intermediate[j].key == intermediate[i].key {
            j +=1;
        }
        let mut values  = vec![];
        for k in i..j {
            values.push(intermediate[k].value.clone());
        }
       let  output = reducef(intermediate[i].key.clone(),values);
       write!(ofile,"{} {}\n",intermediate[i].key,output)?;
       i =j;
    }

    Ok(())
}