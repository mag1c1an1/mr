use crate::KeyValue;
pub fn wc_m(_filename: &str, contents: &str) -> Vec<KeyValue> {
    let words: Vec<&str> = contents.split(|r|!char::is_alphabetic(r)).filter(|&s| s != "").collect();
    let mut kva: Vec<KeyValue> = vec![];
    for w in words {
        kva.push(KeyValue{
           key: w.into(),
            value:"1".into(),
        })
    }
    return kva;
}


pub fn wc_r(_key:String,values:Vec<String>) -> String {
   values.len().to_string() 
}
