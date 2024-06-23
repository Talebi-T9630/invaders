use std::error::Error;
use rusty_audio::Audio;
use std::io;
use crossterm::{terminal, ExecutableCommand};
use crossterm::terminal::{EnterAlternateScreen,LeaveAlternateScreen};
use crossterm::cursor::{Hide,Show};
use crossterm::event;
use crossterm::event::{Event,KeyCode};
use std::time::Duration;
use crate::frame::Frame;
use create::render;


fn main() -> Result <(), Box<dyn Error>> {
    let mut audio= Audio::new();
    audio.add("explode","explode.wav");
    audio.add("lose","lose.wav");
    audio.add("move","move.wav");
    audio.add("pew","pew.wav");
    audio.add("startup","startup.wav");
    audio.add("win","win.wav");
    audio.play("startup");


    //Terminal
    let mut stdout= io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    //Render Loop in a seperate thread
    //setting up channel
    let(render_tx,render_rx)=mpsc::channel();
    let render_handle = threat::spawn(move || {
        let mut last_frame = Frame::new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);

        loop{
            let crr_frame = match render_rx.recv(){
                Ok(x)=>x,
                Err(_)=> break,
            };
            render::render(&mut stdout, &last_frame, &crr_frame, false);
            last_frame = crr_frame;
        }
    })


    //Game loop
        'gameloop: loop{
            //per - frame init
            
            //input
            while event::poll(Duration::default())?{
                if let Event::Key(key_event) = event::read()?{
                    match key_event.code{
                        KeyCode::Esc | KeyCode::Char('q') =>{
                            audio.play("lose");
                            break 'gameloop;
                        },
                        _=>{}
                    }
                }
            }
        }



    //Clean up here
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())

}
