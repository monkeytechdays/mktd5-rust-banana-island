use futures::{Future, Stream};
use tokio_core::reactor::Core;
use hyper::{Client, Method, Request, StatusCode};
use hyper::header::{ContentLength, ContentType};
use serde_json;

use super::model::{GameStateMutex, Move, PlayerMove};

pub struct Mediator {
    base_uri: String,
}

impl Mediator {
    pub fn new(base_uri: &str) -> Mediator {
        Mediator { base_uri: String::from(base_uri) }
    }

    pub fn register(&self, name: String, endpoint: String, game_mutex: GameStateMutex) {
        let base_uri = self.base_uri.clone();
        ::std::thread::spawn(move || {
            Self::register_sync(base_uri, name, endpoint, game_mutex);
        });
    }

    fn register_sync(base_uri: String, name: String, endpoint: String, game_mutex: GameStateMutex) {
        let uri = format!("{}/player", base_uri).parse().unwrap();
        let json = format!(r#"{{ "name": {:?}, "endpoint": {:?} }}"#, name, endpoint);
        let mut req = Request::new(Method::Post, uri);
        req.headers_mut().set(ContentType::json());
        req.headers_mut().set(ContentLength(json.len() as u64));
        req.set_body(json);

        let mut core = Core::new().unwrap();
        let client = Client::new(&core.handle());
        let player = client.request(req)
                           .and_then(|res| {
                               res.body().concat2().and_then(|body| {
                                  let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

                                  let mut player_id = None;
                                  if let serde_json::Value::Number(ref number) = body["id"] {
                                      player_id = number.as_i64();
                                  }

                                  if player_id.is_some() {
                                      let mut game = game_mutex.lock().unwrap();
                                      *game.player_mut() = player_id;
                                      println!("Registred as {}", player_id.unwrap());
                                  } else {
                                      //TODO Stop process if can't read player_id
                                      eprintln!("Unable to register player");
                                  }
                                  Ok(())
                               })
                           });
        if let Err(err) = core.run(player) {
            eprintln!("Error registring player: {:?}", err);
        }
    }

    pub fn play(&self, uuid: String, direction: Move, game_mutex: GameStateMutex) {
        let base_uri = self.base_uri.clone();
        ::std::thread::spawn(move || {
            Self::play_sync(base_uri, uuid, direction, game_mutex);
        });
    }

    fn play_sync(base_uri: String, uuid: String, direction: Move, game_mutex: GameStateMutex) {
        #[derive(Serialize)]
        struct PlayResource {
            #[serde(rename = "move")]
            direction: Move,
        }

        let uri = format!("{}/map", base_uri).parse().unwrap();
        let json = serde_json::to_string(&PlayResource { direction }).unwrap();
        let mut req = Request::new(Method::Post, uri);
        req.headers_mut().set(ContentType::json());
        req.headers_mut().set(ContentLength(json.len() as u64));
        req.headers_mut().set_raw("uuid", uuid);
        req.set_body(json);

        let mut core = Core::new().unwrap();
        let client = Client::new(&core.handle());
        let player = client.request(req)
                           .and_then(|res| {
                               match res.status() {
                                   StatusCode::NoContent => {
                                       let mut game = game_mutex.lock().unwrap();
                                       let id = game.player();
                                       game.map_mut().apply_move(&PlayerMove { id, direction });
                                       println!("Send move: {:?}", direction);
                                   }
                                   _ => {
                                       eprintln!("Invalid play (status={:?})", res.status());
                                   }
                               }

                               Ok(())
                           });
        if let Err(err) = core.run(player) {
            eprintln!("Error submitting player move: {:?}", err);
        }
    }
}
