use crate::command::CommandResult;
use crate::game_loop::{self, GameState, Step, StopType};
use crate::location::GenerationState;
use crate::serialization::LoadError;
use crate::view::Frame;
use crate::{command, location, serialization};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fs::File;
use std::mem;

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
    setup_new_with(GenerationState::load_new(3).unwrap_or_else(|message| panic!("{message}")))
}

pub fn setup_new_with(locations: GenerationState) -> Game {
    let mut game = Game {
        phase: Phase::Invalid,
        state: game_loop::setup(locations),
        frame_cache: FrameCache::new(vec![Frame::Introduction]),
        is_in_error_state: false,
    };
    game.run_from_step(Step::PrepareNextLocation);
    game
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
    #[serde(skip)]
    is_in_error_state: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Phase {
    Invalid,
    ChooseLocation(location::Choice),
    CommandInput,
    Stopped(StopType),
    LoadLocation(String),
}

impl Phase {
    pub fn with_error(self, message: String) -> PhaseResult {
        PhaseResult {
            next_phase: self,
            load_error: Some(message),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PhaseResult {
    next_phase: Phase,
    load_error: Option<String>,
}

impl From<Phase> for PhaseResult {
    fn from(value: Phase) -> Self {
        Self {
            next_phase: value,
            load_error: None,
        }
    }
}

impl Game {
    pub fn next_result(&mut self) -> GameResult {
        if self.frame_cache.has_more_frames() {
            GameResult::Frame(FrameGetter(&mut self.frame_cache))
        } else if self.is_in_error_state {
            GameResult::Stop
        } else {
            match &self.phase {
                Phase::ChooseLocation(_) | Phase::CommandInput => GameResult::Input,
                Phase::Stopped(_) => GameResult::Stop,
                Phase::LoadLocation(location) => {
                    self.run_from_step(Step::LoadLocation(location.to_owned()));
                    self.next_result()
                }
                Phase::Invalid => panic!("Invalid state!"),
            }
        }
    }

    pub fn ready_to_take_input(&self) -> bool {
        if self.frame_cache.has_more_frames() {
            false
        } else {
            match &self.phase {
                Phase::ChooseLocation(_) | Phase::CommandInput => true,
                Phase::Stopped(_) | Phase::LoadLocation(_) => false,
                Phase::Invalid => panic!("Invalid state!"),
            }
        }
    }

    pub fn handle_input(&mut self, input: &str) -> Result<(), Vec<String>> {
        match &self.phase {
            Phase::ChooseLocation(choice) => {
                let location = self
                    .state
                    .generation_state
                    .try_make_choice(choice, input, &mut self.state.rng)
                    .map_err(|error| vec![error])?;
                self.run_from_step(Step::LoadLocation(location));
            }
            Phase::CommandInput => {
                match command::try_parse_input(input, &self.state).map_err(|error| vec![error])? {
                    CommandResult::Action(action, target) => {
                        self.run_from_step(Step::Tick(Some((action, target))));
                    }
                    CommandResult::ChangeControlled(character) => {
                        self.run_from_step(Step::ChangeControlled(character));
                    }
                    CommandResult::Info(text_lines) => return Err(text_lines),
                }
            }
            state => panic!("Handling input in unexpected state {state:?}"),
        }
        Ok(())
    }

    fn run_from_step(&mut self, step: Step) {
        let (phase_result, frames) = game_loop::run(step, &mut self.state);
        self.phase = phase_result.next_phase;
        self.frame_cache.add_new_frames(frames);
        if let Some(message) = phase_result.load_error {
            self.is_in_error_state = true;
            self.frame_cache.error_frame = Some(Frame::Error(message));
        }
    }
}

#[derive(Debug, Default)]
struct FrameCache {
    last_frame: Option<Frame>,
    remaining_frames: Vec<Frame>,
    error_frame: Option<Frame>,
}

impl FrameCache {
    fn new(mut frames: Vec<Frame>) -> Self {
        frames.reverse();
        Self {
            last_frame: None,
            remaining_frames: frames,
            error_frame: None,
        }
    }

    fn add_new_frames(&mut self, mut frames: Vec<Frame>) {
        frames.reverse();
        mem::swap(&mut self.remaining_frames, &mut frames);
        self.remaining_frames.extend(frames);
        self.error_frame = None;
    }

    fn has_more_frames(&self) -> bool {
        !self.remaining_frames.is_empty() || self.error_frame.is_some()
    }

    fn take_next_frame(&mut self) -> Option<Frame> {
        let frame = self.remaining_frames.pop();
        if let Some(frame) = &frame {
            self.last_frame = Some(frame.clone());
        }
        frame.or_else(|| mem::take(&mut self.error_frame))
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
            error_frame: None,
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
