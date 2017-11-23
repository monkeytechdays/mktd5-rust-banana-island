use std::sync::{Arc, Mutex};

use rocket::{self, Rocket, State};
use rocket::fairing::{Info, Fairing, Kind};
use rocket_contrib::Json;

use super::model::{GameStateMutex, GameTick, Map};
use super::mediator;
use super::ai::AI;

pub struct InvokeMediatorOnStart {
    game: GameStateMutex,
}
impl InvokeMediatorOnStart {
    fn new() -> InvokeMediatorOnStart {
        InvokeMediatorOnStart {
            game: Arc::new(Mutex::default())
        }
    }

    fn new_mediator(config: &rocket::Config) -> mediator::Mediator {
        let mediator_uri = config.get_str("mediator_uri").expect("Missing param <mediator_uri>");
        mediator::Mediator::new(mediator_uri)
    }
}
impl Fairing for InvokeMediatorOnStart {
    fn info(&self) -> Info {
        Info {
            name: "Invoke mediator on start",
            kind: Kind::Attach | Kind::Launch,
        }
    }

    fn on_attach(&self, rocket: Rocket) -> Result<Rocket, Rocket> {
        let mediator = Arc::new(Self::new_mediator(rocket.config()));
        Ok(rocket
            .manage(self.game.clone())
            .manage(mediator)
            .manage(Arc::new(Mutex::new(super::ai::AI::new())))
        )
    }

    fn on_launch(&self, rocket: &Rocket) {
        let config = rocket.config();
        let player_name = String::from(config.get_str("player_name").expect("Missing param <player_name>"));
        let address = format!("{}:{}", config.address, config.port);
        Self::new_mediator(config).register(player_name, address, self.game.clone());
    }
}

#[post("/map", data="<map>")]
fn game_start(map: Json<Map>, _game_mutex: State<GameStateMutex>) {
    println!("Game start");
    let mut game = _game_mutex.lock().unwrap();
    game.set_map(map.into_inner());
}

#[post("/map/<uuid>", data="<body>")]
fn game_tick(uuid: String, body: Json<GameTick>,
             game_mutex: State<GameStateMutex>,
             mediator: State<Arc<mediator::Mediator>>,
             ai: State<Arc<Mutex<AI>>>) {
    let tick = body.into_inner();
    println!("Game tick[{}] {:?}", uuid, tick.moves());

    if let Some(moves) = tick.moves().as_ref() {
        let mut game = game_mutex.lock().unwrap();
        for player_move in moves {
            game.map_mut().apply_move(&player_move);
        }
    }

    let game_mutex = game_mutex.clone();
    let mediator = mediator.clone();
    let ai = ai.clone();
    ::std::thread::spawn(move || {
        let direction = {
            let mut game = game_mutex.lock().unwrap();
            let mut ai = ai.lock().unwrap();
            ai.play(&mut game)
        };
        mediator.play(uuid, direction, game_mutex);
    });
}

#[get("/")]
fn index(game_mutex: State<GameStateMutex>) -> String {
    let game = game_mutex.lock().unwrap();
    format!("{:#?}", game)
}

pub fn start() {
    rocket::ignite().mount("/", routes![index, game_start, game_tick])
                    .attach(InvokeMediatorOnStart::new())
                    .launch();
}
