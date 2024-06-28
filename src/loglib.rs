use std::io::Write;

pub struct Logger{
    pub file:std::fs::File,
}
impl Logger{
    pub fn log(&mut self,log_string:String){
        let _=self.file.write(log_string.as_bytes());
        
    }
}