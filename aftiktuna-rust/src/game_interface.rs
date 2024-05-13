use crate::action::{Action, Waiting};
use crate::command::{CommandResult, Target};
use crate::core::position::Pos;
use crate::core::{self, CrewMember, GameState, StopType};
use crate::game_loop::{self, Step};
use crate::location::LocationTracker;
use crate::serialization::LoadError;
use crate::view::{Frame, Messages};
use crate::{command, location, serialization};
use hecs::Satisfies;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fs::File;
use std::mem::swap;

pub fn new_or_load() -> Result<Game, LoadError> {
    match File::open(serialization::SAVE_FILE_NAME) {
        Ok(file) => serialization::load_game(file),
        Err(_) => Ok(setup_new()),
    }
}

pub fn load() -> Result<Game, LoadError> {
    let file = File::open(serialization::SAVE_FILE_NAME)?;
    serialization::load_game(file)
}

pub fn setup_new() -> Game {
    setup_new_with(LocationTracker::new(3))
}

pub fn setup_new_with(locations: LocationTracker) -> Game {
    let mut state = core::setup(locations);
    let mut frame_cache = FrameCache::new(vec![Frame::Introduction]);
    let (phase, frames) = game_loop::run(Step::PrepareNextLocation, &mut state);
    frame_cache.add_new_frames(frames);
    Game {
        phase,
        state,
        frame_cache,
    }
}

pub enum GameResult<'a> {
    Frame(FrameGetter<'a>),
    Input,
    Stop,
}

impl<'a> GameResult<'a> {
    pub fn has_frame(&self) -> bool {
        matches!(self, GameResult::Frame(_))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    phase: Phase,
    state: GameState,
    frame_cache: FrameCache,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Phase {
    ChooseLocation(location::Choice),
    CommandInput,
    Stopped(StopType),
}

impl Game {
    pub fn next_result(&mut self) -> GameResult {
        if self.frame_cache.has_more_frames() {
            GameResult::Frame(FrameGetter(&mut self.frame_cache))
        } else {
            match &self.phase {
                Phase::ChooseLocation(_) | Phase::CommandInput => GameResult::Input,
                Phase::Stopped(_) => GameResult::Stop,
            }
        }
    }

    pub fn ready_to_take_input(&self) -> bool {
        if self.frame_cache.has_more_frames() {
            false
        } else {
            match &self.phase {
                Phase::ChooseLocation(_) | Phase::CommandInput => true,
                Phase::Stopped(_) => false,
            }
        }
    }

    pub fn handle_input(&mut self, input: &str) -> Result<(), Messages> {
        match &self.phase {
            Phase::ChooseLocation(choice) => {
                let location =
                    self.state
                        .locations
                        .try_make_choice(choice, input, &mut self.state.rng)?;
                let (phase, frames) = game_loop::run(Step::LoadLocation(location), &mut self.state);
                self.phase = phase;
                self.frame_cache.add_new_frames(frames);
            }
            Phase::CommandInput => match command::try_parse_input(input, &self.state)? {
                CommandResult::Action(action, target) => {
                    insert_action(&mut self.state, action, target);
                    let (phase, frames) = game_loop::run(Step::Tick, &mut self.state);
                    self.phase = phase;
                    self.frame_cache.add_new_frames(frames);
                }
                CommandResult::ChangeControlled(character) => {
                    let (phase, frames) =
                        game_loop::run(Step::ChangeControlled(character), &mut self.state);
                    self.phase = phase;
                    self.frame_cache.add_new_frames(frames);
                }
                CommandResult::Info(messages) => return Err(messages),
            },
            state => panic!("Handling input in unexpected state {state:?}"),
        }
        Ok(())
    }
}

fn insert_action(state: &mut GameState, action: Action, target: Target) {
    let world = &mut state.world;
    let controlled = state.controlled;
    match target {
        Target::Controlled => {
            world.insert_one(controlled, action).unwrap();
        }
        Target::Crew => {
            let area = world.get::<&Pos>(controlled).unwrap().get_area();
            let aftiks = world
                .query::<(&Pos, Satisfies<&Waiting>)>()
                .with::<&CrewMember>()
                .iter()
                .filter(|&(entity, (pos, is_waiting))| {
                    pos.is_in(area) && (entity == controlled || !is_waiting)
                })
                .map(|(aftik, _)| aftik)
                .collect::<Vec<_>>();
            for aftik in aftiks {
                world.insert_one(aftik, action.clone()).unwrap();
            }
        }
    }
}

#[derive(Debug, Default)]
struct FrameCache {
    last_frame: Option<Frame>,
    remaining_frames: Vec<Frame>,
}

impl FrameCache {
    fn new(mut frames: Vec<Frame>) -> Self {
        frames.reverse();
        Self {
            last_frame: None,
            remaining_frames: frames,
        }
    }

    fn add_new_frames(&mut self, mut frames: Vec<Frame>) {
        frames.reverse();
        swap(&mut self.remaining_frames, &mut frames);
        self.remaining_frames.extend(frames);
    }

    fn has_more_frames(&self) -> bool {
        !self.remaining_frames.is_empty()
    }

    fn take_next_frame(&mut self) -> Option<Frame> {
        let frame = self.remaining_frames.pop();
        if let Some(frame) = &frame {
            self.last_frame = Some(frame.clone());
        }
        frame
    }
}

impl Serialize for FrameCache {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut frames: Vec<&Frame> = self.remaining_frames.iter().collect();
        if let Some(frame) = &self.last_frame {
            frames.push(frame);
        }
        frames.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for FrameCache {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let frames = Vec::<Frame>::deserialize(deserializer)?;
        Ok(Self {
            last_frame: None,
            remaining_frames: frames,
        })
    }
}

// A FrameGetter should only be created under the assumption
// that there is at least one frame available.
pub struct FrameGetter<'a>(&'a mut FrameCache);

impl<'a> FrameGetter<'a> {
    pub fn get(self) -> Frame {
        self.0.take_next_frame().unwrap()
    }
}
