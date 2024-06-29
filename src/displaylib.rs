use std::{ fmt::format, thread::current};

use crossterm::style::{Attribute, Attributes, Color};

use crate::loglib::{self, Logger};
#[derive(Clone,Debug)]
pub enum ChangeType {
    Add(StyledCharacter),
    Delete(StyledCharacter),
    ChangeAttribute(StyledCharacter), //for when i add in multi select
    AddLine,
    RemoveLine,
}
#[derive(Clone,Debug)]
pub struct StyledCharacter{
    pub ch:char,
    pub background_color:Color,
    pub foreground_color:Color,
    pub attributes:Attributes

}
#[derive(Clone,Debug)]
pub struct Action {
    pub all_changes: Vec<Change>,
}
#[derive(Clone,Debug)]
pub struct Change {
    pub row: u16,
    pub column: u16,
    pub change_type: ChangeType,
}
#[derive(Clone,Debug)]
pub struct ChangeStack {
    pub actions: Vec<Action>,
}

impl ChangeStack {
    pub fn revert_action(&mut self) -> Option<Action> {
        return self.actions.pop();
    }
    pub fn add_action(&mut self, action: Action) {
        self.actions.push(action);
    }
    pub fn add_change(&mut self, change_type: ChangeType, current_pos: &mut [u16; 2]) {
        self.add_action(Action {
            all_changes: vec![Change {
                change_type,
                row: current_pos[1],
                column: current_pos[0],
            }],
        })
    }
    pub fn add_paste(&mut self, paste_string: Vec<char>, current_pos: &mut [u16; 2],paste_attributes:Attributes) {
        let mut changes = Vec::<Change>::new();
        for ch in paste_string {
            match ch {
                '\n' => current_pos[1] += 1,
                _ => {
                    let new_change = Change {
                        row: current_pos[1],
                        column: current_pos[0],
                        change_type: ChangeType::Add(StyledCharacter{
                            ch,
                            attributes:paste_attributes,
                            foreground_color:Color::White,
                            background_color:Color::Black
                        }),
                    };
                    changes.push(new_change)
                }
            };
        }
        self.add_action(Action {
            all_changes: changes,
        });
    }
    pub fn add_removed_selection(&mut self, paste_string: Vec<char>, current_pos: &mut [u16; 2],paste_attributes:Attributes) {
        let mut changes = Vec::<Change>::new();
        for ch in paste_string {
            match ch {
                '\n' => current_pos[1] += 1,
                _ => {
                    let new_change = Change {
                        row: current_pos[1],
                        column: current_pos[0],
                        change_type: ChangeType::Delete(StyledCharacter{
                            ch,
                            attributes:paste_attributes,
                            foreground_color:Color::White,
                            background_color:Color::Black
                        }),
                    };
                    changes.push(new_change)
                }
            };
        }
        self.add_action(Action {
            all_changes: changes,
        });
    }
    pub fn undo_change(&mut self,other_stack:&mut ChangeStack,displayText:&mut Vec<Line>,system_logger:&mut Logger,current_pos: &mut [u16; 2]) {
        //remove most recent change and move it to the other stack
        //also loop through changes and edit displayText
        //TODO fix to work when undoing a change moves to a different line
      
        let current_action = match  self.actions.pop(){
            Some(a)=>a,
            None=>return,
        };
        other_stack.add_action(current_action.clone());
        for change in current_action.all_changes{
            match change.change_type{
                ChangeType::Add(styled_character)=>{
                    //character was added so remove it.
                   
                    displayText[usize::from(change.row)].remove_character(change.column-1, system_logger);
                    //update current_position
                    current_pos[0] = change.column-1;
                    current_pos[1] = change.row;
                },
                ChangeType::Delete(styled_character)=>{
                    //character was removed so add it
                    displayText[usize::from(change.row)].add_character(styled_character.ch, change.column, styled_character.attributes, system_logger);
                    current_pos[0] = change.column+1;
                    current_pos[1] = change.row;
                },
                ChangeType::ChangeAttribute(styled_character)=>{
                    //change styling on character in that pos
                    //TODO add method in line to do this
                },
                ChangeType::AddLine=>{
                    displayText.remove(usize::from(change.row));
                    current_pos[0] = change.column;
                    current_pos[1] = change.row-1;
                },
                ChangeType::RemoveLine=>{
                    let new_line = Line{text:Vec::new(),len:0};
                    displayText.insert(usize::from(change.row), new_line);
                    current_pos[0] = change.column;
                    current_pos[1] = change.row+1;
                },
                
            };
            
        }
        

    }
    pub fn redo_change(&mut self,other_stack:&mut ChangeStack,displayText:&mut Vec<Line>,system_logger:&mut Logger,current_pos: &mut [u16; 2]){
        //remove from cache stack and put back in main stack
        //also loop through changes and edit displayText
        //TODO fix to work when redo when a change moves to a different line
        
        let current_action = match  self.actions.pop(){
            Some(a)=>a,
            None=>return,
        };
        other_stack.add_action(current_action.clone());
        for change in current_action.all_changes{
            match change.change_type{
                ChangeType::Add(styled_character)=>{
                    //in cache stack so to redo it we need to add it back
                    
                    displayText[usize::from(change.row)].add_character( styled_character.ch,change.column-1,styled_character.attributes,system_logger);
                    //update current_position
                    current_pos[0] = change.column;
                    current_pos[1] = change.row;
                },
                ChangeType::Delete(styled_character)=>{
                    //ned to remove it
                    displayText[usize::from(change.row)].remove_character(change.column, system_logger);
                    current_pos[0] = change.column;
                    current_pos[1] = change.row;
                },
                ChangeType::ChangeAttribute(styled_character)=>{
                    //change styling on character in that pos
                    //TODO add method in line to do this
                },
                ChangeType::RemoveLine=>{
                    displayText.remove(usize::from(change.row));
                    current_pos[0] = change.column;
                    current_pos[1] = change.row-1;
                },
                ChangeType::AddLine=>{
                    let new_line = Line{text:Vec::new(),len:0};
                    displayText.insert(usize::from(change.row), new_line);
                    current_pos[0] = change.column;
                    current_pos[1] = change.row+1;
                },
            }
        }
    }
    pub fn add_owned_line(&mut self,new_line:&mut Line,current_pos: &mut [u16; 2]){
        let mut changes = Vec::<Change>::new();
        changes.push(Change{row:current_pos[1],column:current_pos[0],change_type:ChangeType::AddLine});
        let mut curr_x = current_pos[0].clone();
        for span_i in 0..new_line.text.len(){
            //loop through each Span
            for ch_i in 0..new_line.text[span_i].text.len(){
                let new_ch = StyledCharacter{ch:new_line.text[span_i].text[ch_i].clone(),background_color:new_line.text[span_i].BackgroundColor,foreground_color:new_line.text[span_i].Color,attributes:new_line.text[span_i].Attributes};
                changes.push(Change{row:current_pos[1],column:curr_x,change_type:ChangeType::Delete(new_ch)});
                curr_x += 1
            }
        }
        //changes.reverse();
        self.add_action(Action{all_changes:changes});
    }
    pub fn clear_stack(&mut self){
        self.actions.clear();
    }

}
#[derive(Clone,Debug,PartialEq)]
pub struct Span{
    pub text:Vec<char>,
    pub BackgroundColor:crossterm::style::Color,
    pub Color:crossterm::style::Color,
    pub Attributes: crossterm::style::Attributes,
}
#[derive(Clone,Debug)]
pub struct Line{
    pub text:Vec<Span>,
    pub len:u16,
}


//TODO add remove_character
impl Line{
    pub fn queue_line(&mut self,stdout: &mut std::io::Stdout,system_logger:&mut loglib::Logger) {
        
        for span in &mut self.text{
            match crossterm::queue!(
                stdout,
                crossterm::style::SetBackgroundColor(span.BackgroundColor),
                crossterm::style::SetForegroundColor(span.Color),
                crossterm::style::SetAttributes(span.Attributes),
                crossterm::style::Print(span.text.iter().collect::<String>()),
                crossterm::style::ResetColor
            ) {
                Ok(_)=>{},
                Err(e)=>{system_logger.log(format!("error:{}\n",e));}
            }
        }
       
        //remember to print a newline character
        match crossterm::queue!(
            stdout,
            crossterm::style::Print("\n".to_string())
        ){
            Ok(_)=>{},
            Err(e)=>{system_logger.log(format!("error:{}\n",e));}
        }
    }
    pub fn add_character(&mut self,ch:char,position:u16,ch_attributes:crossterm::style::Attributes,system_logger:&mut loglib::Logger) {
        //TODO fix to use styledCharacter as input for wider usage
        //TODO re-evaluate/refactor this whole function to be a bit more modular
        //TODO get Bold text working(doesnt work on windows cmd)
        //TODO get to split span when char of different styling inserted
        //go through the vec's until a valid vec is reached
        let mut current_distance = 0;
        if self.text.len() == 0{
            //no span exists create new one
            let new_span = Span{text:vec![ch],BackgroundColor:Color::Black,Color:Color::White,Attributes:ch_attributes};
            self.text.push(new_span);
            self.len = 1;
            return;
        }
        for span_i in 0..self.text.len(){
            let as_u16:Option<u16> = self.text[span_i].text.len().try_into().ok();
            match as_u16{
                Some(v)=>{
                    if v+current_distance >= position{
                       
                        //character is within this vector
                        //check if it has the same attributes
                        let relative_pos = position-current_distance;
                        if(self.text[span_i].Attributes == ch_attributes){
                            
                            self.text[span_i].text.insert(usize::from(relative_pos), ch);
                            self.len += 1;
                            return;//quick exit
                        }
                        //the Attributes of new char and current span do not match

                        if relative_pos == 0 && span_i != 0{
                            //at start of vec check previous vec type
                            if self.text[span_i-1].Attributes == ch_attributes{
                                
                                //create new ch in here
                                let vec_len = self.text[span_i-1].text.len();
                                self.text[span_i-1].text.insert(vec_len, ch);
                                self.len += 1;
                                return;
                            }

                        }
                        //inserting different attribute char at start of line
                        let new_span = Span{
                            text:vec![ch],
                            BackgroundColor:crossterm::style::Color::Black,
                            Color:crossterm::style::Color::White,
                            Attributes:ch_attributes
                        };
                        let mut new_span_location:usize=span_i;
                        if position != 0{
                            let split_span = Span{
                                text:self.text[span_i].text.split_off(usize::from(relative_pos)),
                                BackgroundColor:self.text[span_i].BackgroundColor,
                                Color:self.text[span_i].Color,
                                Attributes:self.text[span_i].Attributes.clone()
                        
                            };   
                            self.text.insert(span_i+1, split_span);
                            new_span_location += 1;
                        }
                    

                        self.text.insert(new_span_location, new_span);
                        self.len += 1;
                      
                        return;
                    }
                    current_distance+=v;
                    
                },
                None=>panic!("bro smth happend i dunno what tho")
            };
            
        }                    
        
    
    }
    pub fn remove_character(&mut self,position:u16,system_logger:&mut loglib::Logger) {
        // if my pos_index is 4 i want to remove at index 3
        let mut current_distance:u16 = 0;
       
        for span_i in 0..self.text.len(){
            let as_u16:Option<u16> = self.text[span_i].text.len().try_into().ok();
            match as_u16{
                Some(v)=>{
                    if v+current_distance > position{
                        //found the span that the current span exists in
                        
                        //generate relative position
                        let relative_pos = position-current_distance;
                        self.text[span_i].text.remove(usize::from(relative_pos));
                        if self.text[span_i].text.len() == 0{
                            //span is now empty. delete it
                            //TODO improve to merge span on either side if similar
                            self.text.remove(span_i);
                        }
                       
                        //quick escape
                        self.len -= 1;
                        return;
                    }
                    current_distance+=v;
                    
                },
                None=>panic!("bro smth happend i dunno what tho")  
            }
        }
    }
    pub fn merge_line(&mut self,other_line: &mut Line,system_logger:&mut loglib::Logger){
        self.len += other_line.len;
        if self.text[self.text.len()-1].Attributes == other_line.text[0].Attributes{
            //merge the 2 spans
            let end = self.text.len()-1;
           
            let mut new_span = other_line.text.remove(0);
            
            self.text[end].text.append(&mut new_span.text);
            
          
        }
        self.text.append(&mut other_line.text);
    }
    pub fn split_line(&mut self,index: usize)->Option<Line>{
        //first i need to find the Span that contains the split segment
        let mut current_distance: usize= 0;
        for span_i in 0..self.text.len(){
            let length = self.text[span_i].text.len();
            if current_distance<=index && index<(current_distance+length){
                //index is within this range of valid index for this span
                let mut edit_span = self.text.remove(span_i);
                let new_span_char_vec = edit_span.text.split_off(index-current_distance);
                let mut new_line_span_vec = self.text.split_off(span_i);
                let mut new_span = edit_span.clone();
                self.text.push(edit_span);
                new_span.text = new_span_char_vec;

                new_line_span_vec.insert(0,new_span);

                let new_line = Line{
                    text:new_line_span_vec,
                    len: (self.len-((index) as u16))
                };
                self.len = (index) as u16;
                
                return Some(new_line);
            }
            current_distance += length
        }
        return None;
    }   
    pub fn log_line(&mut self) ->String{
        let mut new_string = String::new();
        for span_i in 0..self.text.len(){
            let to_add = self.text[span_i].text.iter().collect::<String>();
            new_string += &to_add;
        }
        new_string += "\n";
        new_string
    }
    pub fn get_char(&mut self,index:usize)->Option<char> {
        let mut current_distance = 0;
        for span_i in 0..self.text.len(){
            if self.text[span_i].text.len()+current_distance > index{
                let relative_pos = index-current_distance;
                if self.text[span_i].text.len() == 0{
                    return None;
                }
                return Some(self.text[span_i].text[relative_pos]);
            }
            current_distance+=self.text[span_i].text.len();
        }
        return None;
    }
    
}