use crate::action::{Action, TalkAction};
use crate::asset::GameAssets;
use crate::command;
use crate::command::CommandResult;
use crate::command::parse::{Parse, first_match, first_match_or};
use crate::core::behavior::{Character, GivesHuntRewardData, Waiting};
use crate::core::name::{Name, NameData};
use crate::core::position::Pos;
use crate::core::store::Shopkeeper;
use crate::core::{CrewMember, area, status};
use crate::dialogue::TalkTopic;
use crate::game_loop::GameState;
use hecs::Entity;

pub fn commands(
    parse: &Parse,
    state: &GameState,
    assets: &GameAssets,
) -> Option<Result<CommandResult, String>> {
    first_match!(
        parse.literal("talk", |parse| {
            first_match_or!(
                parse.literal("to", |parse| {
                    parse.match_against(
                        talk_targets(state, assets),
                        |parse, target| parse.done_or_err(|| talk_to(state, target, assets)),
                        |input| Err(format!("\"{input}\" is not a valid target.")),
                    )
                });
                parse.default_err()
            )
        }),
        parse.literal("tell", |parse| {
            parse.match_against(
                super::crew_character_targets(&state.world, assets),
                |parse, target| {
                    first_match_or!(
                        parse.literal("to", |parse|
                            subcommands_for_tell(parse, state, target, assets));
                        parse.default_err()
                    )
                },
                |input| Err(format!("\"{input}\" is not a valid target.")),
            )
        }),
        parse.literal("ask", |parse| {
            parse.match_against(
                talk_targets(state, assets),
                |parse, target| {
                    first_match_or!(
                        parse.literal("to join", |parse|
                            parse.done_or_err(|| ask_to_join(state, target, assets))),
                        parse.literal("for their name", |parse|
                            parse.done_or_err(|| ask_for_name(state, target, assets)));
                        parse.default_err()
                    )
                },
                |input| Err(format!("\"{input}\" is not a valid target.")),
            )
        }),
    )
}

fn subcommands_for_tell(
    parse: Parse,
    state: &GameState,
    target: Entity,
    assets: &GameAssets,
) -> Result<CommandResult, String> {
    first_match_or!(
        parse.literal("wait", |parse|
            first_match_or!(
                parse.literal("at ship", |parse|
                    parse.done_or_err(|| tell_to_wait_at_ship(state, target, assets)));
                parse.done_or_err(|| tell_to_wait(state, target, assets))
            )
        ),
        parse.literal("follow", |parse|
            parse.done_or_err(|| tell_to_follow(state, target, assets)));
        parse.default_err()
    )
}

fn talk_targets(state: &GameState, assets: &GameAssets) -> Vec<(String, Entity)> {
    let character_pos = *state.world.get::<&Pos>(state.controlled).unwrap();
    state
        .world
        .query::<&Pos>()
        .with::<&Character>()
        .iter()
        .filter(|(_, pos)| pos.is_in(character_pos.get_area()))
        .flat_map(|(entity, _)| {
            command::entity_names(state.world.entity(entity).unwrap(), assets)
                .into_iter()
                .map(move |name| (name, entity))
        })
        .collect::<Vec<_>>()
}

fn talk_to(
    state: &GameState,
    target: Entity,
    assets: &GameAssets,
) -> Result<CommandResult, String> {
    check_is_valid_talk_target(target, state, assets)?;

    super::check_adjacent_accessible_with_message(target, state.controlled, &state.world, assets)?;

    if state.world.satisfies::<&Shopkeeper>(target).unwrap() {
        return command::action_result(Action::Trade(target));
    }

    let Some(topic) = TalkTopic::pick(target, &state.world) else {
        return Err(
            if let Ok(gives_hunt_reward) = state.world.get::<&GivesHuntRewardData>(target) {
                format!(
                    "{the_target} is still waiting for {the_hunt_target} to be gone.",
                    the_target = NameData::find(&state.world, target, assets,).definite(),
                    the_hunt_target = gives_hunt_reward.target_label,
                )
            } else {
                format!(
                    "{the_speaker} has nothing to say to {the_target}.",
                    the_speaker =
                        NameData::find(&state.world, state.controlled, assets,).definite(),
                    the_target = NameData::find(&state.world, target, assets,).definite(),
                )
            },
        );
    };

    command::action_result(TalkAction { target, topic })
}

fn ask_for_name(
    state: &GameState,
    target: Entity,
    assets: &GameAssets,
) -> Result<CommandResult, String> {
    check_is_valid_talk_target(target, state, assets)?;

    super::check_adjacent_accessible_with_message(target, state.controlled, &state.world, assets)?;

    if state
        .world
        .get::<&Name>(target)
        .is_ok_and(|name| name.is_known)
    {
        return Err(format!(
            "{the_performer} already knows {the_target}'s name.",
            the_performer = NameData::find(&state.world, state.controlled, assets).definite(),
            the_target = NameData::find(&state.world, target, assets).definite(),
        ));
    }

    command::action_result(TalkAction {
        target,
        topic: TalkTopic::AskName,
    })
}

fn ask_to_join(
    state: &GameState,
    target: Entity,
    assets: &GameAssets,
) -> Result<CommandResult, String> {
    check_is_valid_talk_target(target, state, assets)?;

    if state.world.satisfies::<&CrewMember>(target).unwrap() {
        return Err(format!(
            "{the_target} is already part of the crew.",
            the_target = NameData::find(&state.world, target, assets).definite(),
        ));
    }
    super::check_adjacent_accessible_with_message(target, state.controlled, &state.world, assets)?;

    command::action_result(Action::Recruit(target))
}

fn check_is_valid_talk_target(
    target: Entity,
    state: &GameState,
    assets: &GameAssets,
) -> Result<(), String> {
    if target == state.controlled {
        return Err(format!(
            "{} does not want to talk to themselves.",
            NameData::find(&state.world, state.controlled, assets).definite(),
        ));
    }
    if !status::is_alive(target, &state.world) {
        return Err(format!(
            "{} cannot talk to the dead.",
            NameData::find(&state.world, state.controlled, assets).definite(),
        ));
    }
    Ok(())
}

fn tell_to_wait(
    state: &GameState,
    target: Entity,
    assets: &GameAssets,
) -> Result<CommandResult, String> {
    if state.controlled == target {
        return Err(format!(
            "{} can't give an order to themselves.",
            NameData::find(&state.world, state.controlled, assets).definite()
        ));
    }
    let controlled_pos = state.world.get::<&Pos>(state.controlled).unwrap();
    let target_pos = state.world.get::<&Pos>(target).unwrap();
    if !controlled_pos.is_in(target_pos.get_area()) {
        return Err(format!(
            "{} can't tell {} to do things from here.",
            NameData::find(&state.world, state.controlled, assets).definite(),
            NameData::find(&state.world, target, assets).definite()
        ));
    }

    if state.world.satisfies::<&Waiting>(target).unwrap_or(false) {
        return Err(format!(
            "{} is already waiting.",
            NameData::find(&state.world, target, assets).definite()
        ));
    }

    command::action_result(Action::TellToWait(target))
}

fn tell_to_wait_at_ship(
    state: &GameState,
    target: Entity,
    assets: &GameAssets,
) -> Result<CommandResult, String> {
    if state.controlled == target {
        return Err(format!(
            "{} can't give an order to themselves.",
            NameData::find(&state.world, state.controlled, assets).definite()
        ));
    }
    let controlled_pos = *state.world.get::<&Pos>(state.controlled).unwrap();
    let target_pos = *state.world.get::<&Pos>(target).unwrap();
    if !controlled_pos.is_in(target_pos.get_area()) {
        return Err(format!(
            "{} can't tell {} to do things from here.",
            NameData::find(&state.world, state.controlled, assets).definite(),
            NameData::find(&state.world, target, assets).definite()
        ));
    }
    if area::is_in_ship(target_pos, &state.world)
        && state.world.satisfies::<&Waiting>(target).unwrap_or(false)
    {
        return Err(format!(
            "{} is already waiting at the ship.",
            NameData::find(&state.world, target, assets).definite()
        ));
    }
    if state
        .world
        .get::<&Waiting>(target)
        .is_ok_and(|waiting| waiting.at_ship)
    {
        return Err(format!(
            "{} is already on their way to the ship.",
            NameData::find(&state.world, target, assets).definite()
        ));
    }

    command::action_result(Action::TellToWaitAtShip(target))
}

fn tell_to_follow(
    state: &GameState,
    target: Entity,
    assets: &GameAssets,
) -> Result<CommandResult, String> {
    if state.controlled == target {
        return Err(format!(
            "{} can't give an order to themselves.",
            NameData::find(&state.world, state.controlled, assets).definite()
        ));
    }
    let controlled_pos = state.world.get::<&Pos>(state.controlled).unwrap();
    let target_pos = state.world.get::<&Pos>(target).unwrap();
    if !controlled_pos.is_in(target_pos.get_area()) {
        return Err(format!(
            "{} can't tell {} to do things from here.",
            NameData::find(&state.world, state.controlled, assets).definite(),
            NameData::find(&state.world, target, assets).definite()
        ));
    }

    command::action_result(Action::TellToFollow(target))
}
