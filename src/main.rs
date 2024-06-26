use std::error::Error;
use battle_ship_game::invaders::Invaders;
use rusty_audio::Audio;
use std::io;
use crossterm::{terminal, ExecutableCommand};
use crossterm::terminal::{EnterAlternateScreen,LeaveAlternateScreen};
use crossterm::cursor::{Hide,Show};
use crossterm::event;
use crossterm::event::{Event,KeyCode};
use std::time::{Duration, Instant};
use battle_ship_game::frame;
use battle_ship_game::frame::Drawable;
use battle_ship_game::render;
use battle_ship_game::player::Player;
use std::sync::mpsc;
use std::thread;


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
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
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
    });


    //Game loop
    let mut player= Player::new();
    let mut instant = Instant::now();
    let mut invaders: Invaders = Invaders::new();

        'gameloop: loop{
            //per - frame initialization
            let mut crr_frame= frame::new_frame();
            let delta = instant.elapsed();
            instant= Instant::now();
              
            //input
            while event::poll(Duration::default())?{
                if let Event::Key(key_event) = event::read()?{
                    match key_event.code{
                        KeyCode::Left => player.move_left(),
                        KeyCode::Right=> player.move_right(),
                        KeyCode::Char(' ') | KeyCode::Enter =>{
                            if player.shoot(){
                                audio.play("pew");
                            }
                        }
                        KeyCode::Esc | KeyCode::Char('q') =>{
                            audio.play("lose");
                            break 'gameloop;
                        },
                        _=>{}
                    }
                }
            }

            //updates
            player.update(delta);
           if invaders.update(delta){
            audio.play("move");
           }

           if player.detect_hits(&mut invaders){
            audio.play("explode")
           }

            //draw and render 
            //the drawing lines are being replaved with the generic drawable line
            // player.draw(&mut crr_frame);
            // invaders.draw(&mut crr_frame);
            let drawables: Vec<&dyn Drawable> = vec![&player, &invaders];
            for drawable in drawables{
                drawable.draw(&mut crr_frame)
            }
            let _ = render_tx.send(crr_frame);
            thread::sleep(Duration::from_millis(1));

            // win or lose?

            if invaders.all_killed(){
                audio.play("win");
                break 'gameloop;
            }
            if invaders.reached_bottom(){
                audio.play("lose");
                break 'gameloop;
            }

        }



    //Clean up here
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())

}
