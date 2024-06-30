use crossterm::{cursor::{self, DisableBlinking, EnableBlinking, Hide, MoveTo, MoveToColumn, MoveToRow, RestorePosition, SavePosition, Show}, event::{read, Event, KeyCode,KeyEventKind, KeyModifiers}, execute, style::{self, style, Attribute, Color, Print, ResetColor, SetBackgroundColor, Stylize}, terminal::{self,DisableLineWrap, EnterAlternateScreen}, ExecutableCommand, QueueableCommand
};
use displaylib::{Action, Change, ChangeStack, ChangeType, Line, Span, StyledCharacter};
use loglib::Logger;
use std::{  io::{self, Write}};
mod displaylib;
mod loglib;
fn print_events(stdout: &mut io::Stdout) -> std::io::Result<()> {
    terminal::enable_raw_mode()?;
    loop {
        // `read()` blocks until an `Event` is available
        match read()? {
            Event::FocusGained => println!("FocusGained"),
            Event::FocusLost => println!("FocusLost"),
            Event::Key(event) => println!("{:?}", event),
            Event::Mouse(event) => println!("{:?}", event),
            Event::Paste(data) => println!("{:?}", data),
            Event::Resize(width, height) => println!("New size {}x{}", width, height),
            _ => {}
        }
        stdout.flush()?;
    }
    //for cleanup
    //terminal::disable_raw_mode()?;
}
fn refresh_line(stdout: &mut io::Stdout,current_pos: &[u16; 2],displayText:&mut Vec<Line>,system_logger:&mut Logger)->io::Result<()>{
    system_logger.log("redrawing line\n".to_string());
    stdout.queue(Hide)?;
    stdout.queue(MoveTo(0,current_pos[1]))?;
    stdout.queue(Show)?;
    stdout.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
    system_logger.log("is the error here\n".to_string());
    system_logger.log(format!("attempting to refresh line ={}",usize::from(current_pos[1])));
    displayText[usize::from(current_pos[1])].queue_line(stdout,system_logger);
    system_logger.log(displayText[usize::from(current_pos[1])].log_line());
    Ok(())
}
fn refresh_screen(stdout: &mut io::Stdout,displayText:&mut Vec<Line>,system_logger:&mut Logger)->io::Result<()>{
    
    stdout.queue(terminal::Clear(terminal::ClearType::All))?;
    stdout.queue(terminal::Clear(terminal::ClearType::Purge))?;

    for line in displayText{
        
        line.queue_line(stdout, system_logger);
        
    } 
    return Ok(()); 

}
fn main() -> io::Result<()> {
   
    
    //stdout.queue(Print("this is some text".to_string()))?;
    //execute!(
    //    std::io::stdout(),
    //    SetBackgroundColor(Color::Blue),
    //   Print("temp test"),
    //    ResetColor,
    //    SetBackgroundColor(Color::White),
    //    Print("test highlight text")
    //)?;
    
    main_loop();
    Ok(())
}

fn main_loop() -> io::Result<()> {
    let mut main_stack = ChangeStack{actions:Vec::<Action>::new()};
    let mut cache_stack = ChangeStack{actions:Vec::<Action>::new()};


    let mut stdout = io::stdout();
    let mut displayText = Vec::<Line>::new();
    let mut file = match std::fs::File::create(r"C:\programming\rust\Projects\TextEditorCLITest\src\log.txt"){
        Ok(v)=>v,
        Err(_)=>panic!("failed to open log file"),
    };
    let mut system_logger = loglib::Logger{file};
    let mut active_attributes = style::Attributes::default();

    terminal::enable_raw_mode()?;
    let _screen = EnterAlternateScreen;
    
    let start_string = "this is some temporary text!";
    let start_string2 = "this is some temporary text22323242!";
    let mut line_vec: Vec<char> = start_string.chars().collect();
    let mut line_vec2: Vec<char> = start_string2.chars().collect();
    let u16_value1: Option<u16> = line_vec.len().try_into().ok();
    let u16_value2: Option<u16> = line_vec2.len().try_into().ok();
    displayText.push(
        Line{
            text:vec![Span{
                text:line_vec2,
                BackgroundColor:Color::Black,
                Color:Color::White,
                Attributes:active_attributes.clone()}],
            len:u16_value2.unwrap()});
    displayText.push(
        Line{
            text:vec![Span{
                text:line_vec.clone(),
                BackgroundColor:Color::Black,
                Color:Color::White,
                Attributes:active_attributes.clone()}],
            len:u16_value1.unwrap()});
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    stdout.execute(DisableLineWrap)?;

    //execute!(
//            std::io::stdout(),
//            Print(start_string.to_string())
    //    )?;
    //stdout.execute(Print(start_string.to_string()))?;
    for line in &mut displayText{
        line.queue_line(&mut stdout,&mut system_logger);
    }
    stdout.flush();
    let mut current_pos = [0, 0];
    stdout.execute(cursor::MoveTo(current_pos[0], current_pos[1]))?;
    //stdout.execute(Show)?;
    
    loop {
        //stdout.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
        
      
        match read()? {
            Event::Key(event) => {
                
                //println!("{:?}",event)
                if event.kind ==  KeyEventKind::Press {
                    match event.code {
                        KeyCode::Right => {
                            
                            //move cursor Right
                            //TODO fix to add to end of line properly
                            if current_pos[0] < displayText[usize::from(current_pos[1])].len{
                                current_pos[0] += 1;
                                
                            }
                            
                        },
                        KeyCode::Left => {
                            
                            //move cursor Left
                            if current_pos[0] > 0{
                                current_pos[0] -= 1;
                                
                            };

                            
                        },
                        KeyCode::Up => {
                           
                            //move up
                            if current_pos[1] > 0{
                                current_pos[1] -= 1;
                                //if line above is shorter move to end of line
                                //if line above is longer and on end of current line move to end
                                if current_pos[0] > displayText[usize::from(current_pos[1])].len ||(current_pos[0] < displayText[usize::from(current_pos[1])].len && current_pos[0]== displayText[usize::from(current_pos[1]+1)].len){
                                    current_pos[0] = displayText[usize::from(current_pos[1])].len;
                                }
                            }
                          
                        },
                        KeyCode::Down => {
                            //move down
                            if usize::from(current_pos[1]) < displayText.len()-1{
                                current_pos[1] += 1;
                                //if line below is shorter move to end of line
                                if current_pos[0] > displayText[usize::from(current_pos[1])].len{
                                     current_pos[0] = displayText[usize::from(current_pos[1])].len;
                                } 
                            }
                          
                        },
                        KeyCode::Backspace=>{
                            //TODO make work with new system
                            //TODO breaks when line ends, edit so move to next line
                            if displayText.len() != 0{
                                if current_pos[0] == 0{
                                    if displayText[usize::from(current_pos[1])].len == 0 {
                                        displayText.remove(usize::from(current_pos[1]));
                                        
                                    
                                        stdout.queue(Hide)?;
                                        stdout.queue(MoveTo(0,0))?;
                                        refresh_screen(&mut stdout,&mut displayText,&mut system_logger)?;
                                        stdout.queue(MoveTo(0,current_pos[1]))?;
                                        stdout.queue(Show)?;

                                        if current_pos[1] != 0{
                                            current_pos[1] -= 1;
                                        }

                                    }else{
                                        if current_pos[1] >0{
                                            //merge current line with line above and refresh screen
                                            
                                            let mut current_line = displayText.remove(usize::from(current_pos[1]));
                                           
                                            current_pos[1] -= 1;
                                            current_pos[0] = displayText[usize::from(current_pos[1])].len;
                                            stdout.queue(Hide)?;
                                            stdout.queue(MoveTo(0,0))?;

                                            displayText[usize::from(current_pos[1])].merge_line(&mut current_line,&mut system_logger);
                                           
                                            refresh_screen(&mut stdout, &mut displayText, &mut system_logger)?;
                                            stdout.queue(Show)?;
                                            stdout.queue(MoveTo(current_pos[0],current_pos[1]))?;
                                           
                                        }
                                    }
                                }else{
                                    if current_pos[0] > 0{
                                        current_pos[0] -= 1;
                                        
                                    };
                                    let ch = match displayText[usize::from(current_pos[1])].get_char(usize::from(current_pos[0])){
                                        Some(c)=>c,
                                        None=>panic!("line should have been deleted")
                                        
                                    };
                                    main_stack.add_action(Action{
                                        all_changes:vec![Change{
                                            row:current_pos[1],
                                            column:current_pos[0],
                                            change_type:ChangeType::Delete(StyledCharacter{
                                                ch:ch,
                                                attributes:active_attributes,
                                                background_color:Color::Black,
                                                foreground_color:Color::White})
                                        }]
                                    }); 
                                   
                                    displayText[usize::from(current_pos[1])].remove_character(current_pos[0],&mut system_logger);
                                    
                                
                                    refresh_line(&mut stdout,&current_pos,&mut displayText,&mut system_logger)?;
                                }
                            }
                                
                        }
                        KeyCode::Char(ch)=>{
                            //TODO Current task. add ctrl B and ctrl U
                            match event.modifiers{
                                KeyModifiers::CONTROL=>{
                                    match ch{
                                        'b'=>active_attributes.toggle(Attribute::Bold),
                                        'u'=>active_attributes.toggle(Attribute::Underlined),
                                        'z'=>{
                                            main_stack.undo_change(&mut cache_stack, &mut displayText, &mut system_logger, &mut current_pos);
                                            
                                            stdout.queue(Hide)?;
                                            stdout.queue(MoveTo(0,0))?;
                                            refresh_screen(&mut stdout, &mut displayText, &mut system_logger)?;
                                            stdout.queue(MoveTo(current_pos[0],current_pos[1]))?;
                                            stdout.queue(Show)?;
                                        },
                                        'y'=>{
                                            cache_stack.redo_change(&mut main_stack, &mut displayText, &mut system_logger, &mut current_pos);
                                           
                                            stdout.queue(Hide)?;
                                            stdout.queue(MoveTo(0,0))?;
                                            refresh_screen(&mut stdout, &mut displayText, &mut system_logger)?;
                                            stdout.queue(MoveTo(current_pos[0],current_pos[1]))?;
                                            stdout.queue(Show)?;
                                            
                                        },
                                        _=>{}
                                    }
                                },
                                KeyModifiers::NONE | KeyModifiers::SHIFT=>{
                                    displayText[usize::from(current_pos[1])].add_character(ch,current_pos[0],active_attributes.clone(),&mut system_logger);
                                    current_pos[0] += 1;
                                    refresh_line(&mut stdout,&current_pos,&mut displayText,&mut system_logger)?;
                                    cache_stack.clear_stack();
                                    main_stack.add_action(Action{
                                        all_changes:vec![Change{
                                            row:current_pos[1],
                                            column:current_pos[0],
                                            change_type:ChangeType::Add(StyledCharacter{
                                                ch:ch,attributes:active_attributes,
                                                background_color:Color::Black,
                                                foreground_color:Color::White})
                                        }]
                                    });
                                }
                                _=>{}
                            }
                     
                        },
                        KeyCode::Enter if event.modifiers==KeyModifiers::NONE=>{
                            //3 scenarios
                            //1 at end of line - create new line below
                            //2 at start of line - create new line above
                            //3 in between - split the line
                            let mut new_line:Option<Line> = Some(Line{text:Vec::new(),len:0});
                            let mut new_line_pos:u16 = 0;
                            if current_pos[0] == 0{
                                //create new line at index 0
                                //fix a bit. since if i am on an empty line this breaks a bit//if currPos ==0 and len=new line below
                                if(displayText[usize::from(current_pos[1])].len == 0){
                                    current_pos[1] +=1;
                                    new_line_pos = current_pos[1];
                                    main_stack.add_change(ChangeType::AddLine, &mut current_pos);
                                }else{
                                    new_line_pos = current_pos[1];
                                    main_stack.add_change(ChangeType::AddLine, &mut current_pos);
                                    current_pos[1] +=1;
                                }
                               
                            }else if current_pos[0] == displayText[usize::from(current_pos[1])].len{
                                
                                new_line_pos = current_pos[1]+1;
                                current_pos[1] +=1;
                                
                                main_stack.add_change(ChangeType::AddLine, &mut current_pos);
                            }else{
                                new_line = displayText[usize::from(current_pos[1])].split_line(usize::from(current_pos[0]));
                                new_line_pos = current_pos[1]+1;
                                main_stack.add_owned_line(new_line.as_mut().unwrap(), &mut current_pos);
                                current_pos[1] +=1;
                            }
                            let new_line = match new_line{
                                Some(line)=>line,
                                None=>panic!("there is supposed to be something here")
                            };
                           
                            current_pos[0] = 0;
                            
                            displayText.insert(usize::from(new_line_pos), new_line);
                            stdout.queue(Hide)?;
                            stdout.queue(MoveTo(0,0))?;                                
                            refresh_screen(&mut stdout, &mut displayText, &mut system_logger)?;
                            stdout.queue(Show)?;
                            stdout.queue(MoveTo(current_pos[0],current_pos[1]))?;

                        },
                        _ => {}
                    }
                }
            },
            _ => {}
        }
        //stdout.queue(Hide)?;
        //stdout.queue(MoveTo(0,0))?;
        //stdout.queue(Print(line_vec.iter().collect::<String>()))?;
        //for line in &mut displayText{
        //    line.queue_line(&mut stdout);
        //}
        
        stdout.queue(MoveTo(current_pos[0],current_pos[1]))?;
        //stdout.queue(Show)?;
        stdout.flush()?;
    }
}
